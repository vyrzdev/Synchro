use std::collections::HashSet;
use std::env;
use std::str::FromStr;
use tokio::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use chrono::{DateTime, Utc};
use log::{error, warn};
use squareup::api::{CatalogApi, InventoryApi};
use squareup::config::{BaseUri, Configuration, Environment};
use squareup::http::client::HttpClientConfiguration;
use squareup::models::enums::InventoryState::InStock;
use squareup::models::{BatchChangeInventoryRequest, InventoryChange, InventoryPhysicalCount, RetrieveInventoryCountParams};
use squareup::models::enums::InventoryChangeType;
use squareup::SquareClient;
use tokio::time::sleep;
use uuid::Uuid;
use crate::intervals::Interval;
use crate::observations::Observation;
use crate::ordering::PlatformMetadata;
use crate::predicates::DefinitionPredicate;
use squareup::models::DateTime as SquareDateTime;
use tokio::sync::mpsc::error::TryRecvError;
use crate::real_world::square::{SquareMetadata, Target, IGNORE};
use crate::simulation::record::RecordConfig;
use crate::value::Value;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PollingInterpretation {
    Transition,
    Mutation,
    Assignment
}

pub struct SquarePollingConfig {
    pub(crate) token: String,
    backoff: Duration,
    target: Target,
    interpretation: PollingInterpretation,
}

pub struct SquarePollingInterface {
    pub(crate) name: String,
    pub(crate) catalog_api: CatalogApi,
    pub(crate) inventory_api: InventoryApi,
    pub(crate) config: SquarePollingConfig,
}

impl SquarePollingInterface {
    pub fn new(name: String, config: SquarePollingConfig) -> SquarePollingInterface {
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

        return SquarePollingInterface { name, catalog_api, inventory_api, config};
    }
    pub async fn poll_worker(&mut self, mut to_write: Receiver<Value>, observation_out: Sender<Observation<DateTime<Utc>>>, initial_poll: (DateTime<Utc>, Value)) -> ! {
        let (mut last_sent, mut last_value) = initial_poll;

        loop {
            let (value, sent, replied) = self.request(self.config.target.clone()).await;
            if (value != last_value) {
                // Generate Observation!
                observation_out.send(Observation {
                    interval: Interval(last_sent, replied),
                    definition_predicate: match &self.config.interpretation {
                        PollingInterpretation::Transition => DefinitionPredicate::Transition(last_value, value),
                        PollingInterpretation::Mutation => DefinitionPredicate::AllMut((value - last_value)),
                        PollingInterpretation::Assignment => DefinitionPredicate::LastAssn(value),
                    },
                    source: self.name.clone(),
                    platform_metadata: PlatformMetadata::Square(SquareMetadata {
                        timestamp: sent // Use poll sent times as logical ordering.
                    }),
                }).unwrap();
            }

            match to_write.try_recv() {
                // If some value waiting to write.
                Ok(to_write) => {
                    // Write it - NOTE: UNSAFE!
                    let sent_at = Utc::now();
                    self.write(to_write).await;

                    (last_sent, last_value) = (sent_at, to_write);
                    // Write completed- schedule next poll.
                    sleep(self.config.backoff).await;
                },
                Err(TryRecvError::Disconnected) => panic!("SquarePolling :: Interpreter Output Channel hung up!"),
                // No value waiting to write- schedule next poll.
                _ => sleep(self.config.backoff).await
            }
        }
    }

    pub async fn request(&self, target: Target) -> (Value, DateTime<Utc>, DateTime<Utc>) {
        let params = RetrieveInventoryCountParams {
            location_ids: Some(vec![self.config.target.0.clone()]),
            cursor: None
        };

        loop {
            let sent = chrono::Utc::now();
            match self.inventory_api.retrieve_inventory_count(self.config.target.1.clone(), params.clone()).await {
                Ok(response) => {
                    let replied = chrono::Utc::now();
                    let counts = response.counts.unwrap();
                    let value = counts.iter()
                        .filter(|c| c.state == InStock)
                        .map(|c| Value::from_str(&c.quantity).unwrap())
                        .sum::<Value>();

                    return (value, sent, replied);
                },
                Err(e) => {
                    error!("SquareInterface - {}:: API Error: {e:#?}", self.name);
                    warn!("Sandboxing, waiting 200ms and trying again!");
                    sleep(Duration::from_millis(200)).await;
                }
            }
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