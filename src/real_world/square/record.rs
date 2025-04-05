use std::cmp::{max, min};
use std::collections::HashSet;
use std::env;
use std::str::FromStr;
use std::time::Duration;
use chrono::{DateTime, TimeDelta, Utc};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use squareup::api::{CatalogApi, InventoryApi};
use squareup::config::{BaseUri, Configuration, Environment};
use squareup::models::{BatchChangeInventoryRequest, DateTime as SquareDateTime, InventoryAdjustment, InventoryPhysicalCount};
use squareup::http::client::HttpClientConfiguration;
use squareup::models::enums::InventoryState::InStock;
use squareup::models::{BatchRetrieveInventoryChangesRequest, InventoryChange};
use squareup::models::enums::{InventoryChangeType, InventoryState};
use squareup::SquareClient;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch;
use tokio::time::sleep;
use uuid::Uuid;
use crate::intervals::Interval;
use crate::observations::Observation;
use crate::ordering::PlatformMetadata;
use crate::predicates::DefinitionPredicate;
use crate::real_world::square::{SquareMetadata, Target, IGNORE};
use crate::value::Value;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SquareRecordConfig {
    token: String,
    backoff: Duration,
    target: Target,
    calibration_target: Target,
}

pub struct SquareRecordInterface {
    pub(crate) name: String,
    pub(crate) seen_change_ids: HashSet<String>,
    pub(crate) catalog_api: CatalogApi,
    pub(crate) inventory_api: InventoryApi,
    pub(crate) config: SquareRecordConfig,
    net_deviation_min: TimeDelta,
    net_deviation_max: TimeDelta,
}

impl SquareRecordInterface {
    pub async fn new(name: String, config: SquareRecordConfig) -> SquareRecordInterface {
        // Set Auth Token (in config)
        unsafe {
            env::set_var("SQUARE_API_TOKEN", config.token.clone());
        }

        // Initialise Catalog API
        let catalog_api = CatalogApi::new(SquareClient::try_new(Configuration {
            environment: Environment::Sandbox, // Testing in Sandbox Environment
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        }).unwrap());

        // Initialise Inventory API
        let inventory_api = InventoryApi::new(SquareClient::try_new(Configuration {
            environment: Environment::Sandbox, // Testing in Sandbox Environment
            http_client_config: HttpClientConfiguration::default(),
            base_uri: BaseUri::default(),
        }).unwrap());

        let mut net_deviation_max = TimeDelta::MIN;
        let mut net_deviation_min = TimeDelta::MAX;
        for i in 0..5 {
            info!("{} - Deviation Query, request {}/5", name, i);

            let request_key = Uuid::new_v4().to_string();
            let sent: DateTime<Utc> = Utc::now();
            let request = BatchChangeInventoryRequest {
                idempotency_key: request_key.clone(),
                changes: Some(vec![
                    InventoryChange {
                        r#type: Some(InventoryChangeType::Adjustment),
                        physical_count: None,
                        adjustment: Some(InventoryAdjustment {
                            id: None,
                            reference_id: Some(request_key.clone()),
                            from_state: Some(InventoryState::None),
                            to_state: Some(InventoryState::InStock),
                            location_id: Some(config.calibration_target.0.clone()),
                            catalog_object_id: Some(config.calibration_target.1.clone()),
                            catalog_object_type: None,
                            quantity: Some("1".to_string()),
                            total_price_money: None,
                            occurred_at: Some(Default::default()),
                            created_at: None,
                            source: None,
                            employee_id: None,
                            team_member_id: None,
                            transaction_id: None,
                            refund_id: None,
                            purchase_order_id: None,
                            goods_receipt_id: None,
                            adjustment_group: None,
                        }),
                        transfer: None,
                        measurement_unit: None,
                        measurement_unit_id: None,
                    }
                ]),
                ignore_unchanged_counts: None,
            };
            loop {
                match inventory_api.batch_change_inventory(&request).await {
                    Ok(resp) => {
                        let replied: DateTime<Utc> = Utc::now();
                        let change = resp.changes.as_ref().expect(format!("Failed to do Calibration: {}", name.clone()).as_str()).iter().nth(0).unwrap();
                        let created_at = change.adjustment.as_ref().unwrap().created_at.as_ref().unwrap();
                        let timestamp_dev: DateTime<Utc> = DateTime::from(created_at.clone());
                        let timestamp_min = timestamp_dev;
                        let timestamp_max = timestamp_dev + TimeDelta::milliseconds(1);

                        let deviation_max = timestamp_max - sent;
                        let deviation_min = timestamp_min - replied;
                        net_deviation_max = max(deviation_max, net_deviation_max);
                        net_deviation_min = min(deviation_min, net_deviation_min);
                        sleep(Duration::from_millis(400)).await; // Backoff to avoid rate limit spoiling calculations.
                        break
                    },
                    Err(e) => {
                        error!("SquareInterface - {}:: API Error: {e:#?}", name);
                        warn!("Sandboxing, waiting 200ms and trying again!");
                        sleep(Duration::from_millis(200)).await;
                    }
                }
            }

        }

        info!("{} Calibrated: {}, {}", name, net_deviation_min, net_deviation_max);

        return SquareRecordInterface { name, catalog_api, inventory_api, seen_change_ids:HashSet::new(), config, net_deviation_min, net_deviation_max };
    }

    pub async fn request_events(&self, since: DateTime<Utc>) -> Vec<InventoryChange> {
        let mut request = BatchRetrieveInventoryChangesRequest {
            catalog_object_ids: Some(vec![self.config.target.1.clone()]),
            location_ids: Some(vec![self.config.target.0.clone()]),
            updated_after: Some((&since).into()),
            ..Default::default()
        };

        loop {
            match self.inventory_api.batch_retrieve_inventory_changes(&mut request).await {
                Ok(response) => {
                    return response.changes.unwrap_or_else(|| vec![])
                },
                Err(e) => {
                    error!("SquareInterface - {}:: API Error: {e:#?}", self.name);
                    warn!("Sandboxing, waiting 200ms and trying again!");
                    sleep(Duration::from_millis(200)).await;
                }
            }
        }
    }

    pub fn parse_change(&self, change: InventoryChange) -> (String, Option<Observation<DateTime<Utc>>>) {
        match change.r#type.as_ref().unwrap() {
            InventoryChangeType::PhysicalCount => {
                // Why did they structure it like this :sob:
                let physical_count = change.physical_count.unwrap();

                if physical_count.reference_id.is_some_and(|id| id.eq(IGNORE)) {
                    return (physical_count.id.unwrap(), None); // Ignore event.
                }
                let created_at = physical_count.created_at.unwrap();
                let min = DateTime::<Utc>::from(created_at.clone()) - self.net_deviation_max;
                let max = DateTime::<Utc>::from(created_at.clone())  - self.net_deviation_min;

                (physical_count.id.unwrap(), Some(Observation::<DateTime<Utc>> {
                    interval: Interval(min, max),
                    definition_predicate: DefinitionPredicate::LastAssn(Value::from_str(&physical_count.quantity.unwrap()).unwrap()),
                    source: self.name.clone(),
                    platform_metadata: PlatformMetadata::Square(SquareMetadata {
                        timestamp: created_at.into()
                    }),
                }))
            },
            InventoryChangeType::Adjustment => {
                let adjustment = change.adjustment.unwrap();

                if adjustment.reference_id.is_some_and(|id| id.eq(IGNORE)) {
                    return (adjustment.id.unwrap(), None); // Ignore event.
                }

                let created_at = adjustment.created_at.unwrap();
                let min = DateTime::<Utc>::from(created_at.clone())  - self.net_deviation_max;
                let max = DateTime::<Utc>::from(created_at.clone())  - self.net_deviation_min;
                let mut quantity = Value::from_str(&adjustment.quantity.unwrap()).unwrap();

                // Came FROM instock- must be a decrement.
                if matches!(adjustment.from_state, Some(InStock)) {
                    quantity = -quantity;
                }

                (adjustment.id.unwrap(), Some(Observation::<DateTime<Utc>> {
                    interval: Interval(min, max),
                    definition_predicate: DefinitionPredicate::AllMut(quantity),
                    source: self.name.clone(),
                    platform_metadata: PlatformMetadata::Square(SquareMetadata {
                        timestamp: created_at.into()
                    }),
                }))
            },
            _ => {
                // We do not test with transfers
                error!("We do not test across multiple locations!");
                panic!()
            }
        }
    }

    pub async fn record_worker(&mut self, mut to_write: watch::Receiver<Option<Value>>, observation_out: Sender<Observation<DateTime<Utc>>>) -> ! {
        let mut seen = HashSet::new();
        let mut last = Utc::now();
        loop {
            let events = self.request_events(last + TimeDelta::milliseconds(-5000));
            last = Utc::now();
            let results = events.await;

            for change in results {
                let (id, observation) = self.parse_change(change);
                if let Some(observation) = observation {
                    if !seen.contains(&id) {
                        info!("{} - Observed New: {:?}!", self.name, observation);
                        observation_out.send(observation).await.unwrap();
                        seen.insert(id);
                    }
                }
            }

            if to_write.has_changed().unwrap() {
                to_write.mark_unchanged();
                // If some value waiting to write.
                let value = to_write.borrow().unwrap().clone();
                self.write(value).await;
            }

            // Wait before next request.
            sleep(self.config.backoff).await;
        }
    }


    pub async fn write(&mut self, value: Value) {
        let params = BatchChangeInventoryRequest {
            idempotency_key: Uuid::new_v4().to_string(),
            changes: Some(vec![
                InventoryChange {
                    r#type: Some(InventoryChangeType::PhysicalCount),
                    physical_count: Some(
                        InventoryPhysicalCount {
                            id: None,
                            reference_id: Some(IGNORE.to_string()),
                            catalog_object_id: Some(self.config.target.1.clone()),
                            catalog_object_type: None,
                            state: Some(InStock),
                            location_id: Some(self.config.target.0.clone()),
                            quantity: Some(value.to_string()),
                            source: None,
                            employee_id: None,
                            team_member_id: None,
                            occurred_at: Some(SquareDateTime::now()),
                            created_at: None,
                        }
                    ),
                    adjustment: None,
                    transfer: None,
                    measurement_unit: None,
                    measurement_unit_id: None,
                }
            ]),
            ..Default::default()
        };

        loop {
            match self.inventory_api.batch_change_inventory(&params).await {
                Ok(_) => {
                    info!("{} - Wrote Value: {:?}!", self.name, value);
                    return;
                },
                Err(e) => {
                    error!("SquareInterface - {}:: API Error: {e:#?}", self.name);
                    warn!("Sandboxing, waiting 200ms and trying again!");
                    sleep(Duration::from_millis(200)).await;
                }
            }
        }

    }
}