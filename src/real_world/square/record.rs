use std::collections::HashSet;
use std::env;
use std::str::FromStr;
use std::time::Duration;
use chrono::{DateTime, TimeDelta, Utc};
use log::{error, warn};
use serde::{Deserialize, Serialize};
use squareup::api::{CatalogApi, InventoryApi};
use squareup::config::{BaseUri, Configuration, Environment};
use squareup::models::{BatchChangeInventoryRequest, DateTime as SquareDateTime, InventoryPhysicalCount};
use squareup::http::client::HttpClientConfiguration;
use squareup::models::enums::InventoryState::InStock;
use squareup::models::errors::SquareApiError;
use squareup::models::{BatchRetrieveInventoryChangesRequest, BatchRetrieveInventoryChangesResponse, InventoryChange, RetrieveInventoryCountParams};
use squareup::models::enums::InventoryChangeType;
use squareup::SquareClient;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
use uuid::Uuid;
use crate::intervals::Interval;
use crate::observations::Observation;
use crate::ordering::PlatformMetadata;
use crate::predicates::DefinitionPredicate;
use crate::real_world::square::{SquareMetadata, Target, IGNORE};
use crate::real_world::square::polling::SquarePollingConfig;
use crate::value::Value;


pub struct SquareRecordConfig {
    token: String,
    backoff: Duration,
    target: Target,
    calibration_target: String,
}

pub struct SquareRecordInterface {
    pub(crate) name: String,
    pub(crate) seen_change_ids: HashSet<String>,
    pub(crate) catalog_api: CatalogApi,
    pub(crate) inventory_api: InventoryApi,
    pub(crate) config: SquareRecordConfig,
}

impl SquareRecordInterface {
    pub fn new(name: String, config: SquareRecordConfig) -> SquareRecordInterface {
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

        return SquareRecordInterface { name, catalog_api, inventory_api, seen_change_ids:HashSet::new(), config};
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
                let (min, max) = (created_at.clone().into(), <SquareDateTime as Into<DateTime<Utc>>>::into(created_at.clone()) + TimeDelta::milliseconds(1));

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
                let (min, max) = (created_at.clone().into(), <SquareDateTime as Into<DateTime<Utc>>>::into(created_at.clone()) + TimeDelta::milliseconds(1));

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

    pub async fn record_worker(&mut self, output: Sender<Observation<DateTime<Utc>>>) -> ! {
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
                        output.send(observation).await.unwrap();
                        seen.insert(id);
                    }
                }
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