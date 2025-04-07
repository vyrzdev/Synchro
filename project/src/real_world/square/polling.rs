use std::env;
use std::str::FromStr;
use tokio::sync::mpsc::{Sender};
use std::time::Duration;
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use squareup::api::{CatalogApi, InventoryApi};
use squareup::config::{BaseUri, Configuration, Environment};
use squareup::http::client::HttpClientConfiguration;
use squareup::models::enums::InventoryState::InStock;
use squareup::models::{BatchChangeInventoryRequest, InventoryChange, InventoryPhysicalCount, RetrieveInventoryCountParams};
use squareup::models::enums::InventoryChangeType;
use squareup::SquareClient;
use tokio::time::sleep;
use uuid::Uuid;
use squareup::models::DateTime as SquareDateTime;
use tokio::sync::watch;
use crate::core::intervals::Interval;
use crate::core::observations::Observation;
use crate::core::ordering::PlatformMetadata;
use crate::core::predicates::DefinitionPredicate;
use crate::real_world::square::{SquareMetadata, Target, IGNORE};
use crate::value::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PollingInterpretation {
    Transition,
    Mutation,
    Assignment
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// Creates a new `SquarePollingInterface`
    ///
    /// # Arguments
    /// * `name` - Identifier for the polling interface.
    /// * `config` - Configuration for polling, including token, backoff, target, and interpretation.
    ///
    /// # Returns
    /// A fully initialized `SquarePollingInterface`.
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

    /// Polling worker thread.
    /// # Arguments
    /// * `to_write` - A `watch::Receiver` instance that listens for new values to write to
    ///    Square API.
    /// * `observation_out` - An `mpsc::Sender` instance used to send `Observation`s,
    ///    containing data about inventory changes
    /// * `initial_poll` - A tuple containing the initial timestamp and value of the replica
    ///
    /// # Behavior
    ///
    /// - Polls the Square API for inventory values at regular intervals, specified by
    ///   `config.backoff`.
    /// - Compares the polled value to the last retrieved value:
    ///   - If there is a change, generates an `Observation` which describes the change.
    /// - Listens for changes to the `to_write` receiver channel:
    ///   - If a value is detected, writes the value to Square's API and updates the state.
    pub async fn poll_worker(&mut self, mut to_write: watch::Receiver<Option<Value>>, observation_out: Sender<Observation<DateTime<Utc>>>, initial_poll: (DateTime<Utc>, Value)) -> ! {
        let (mut last_sent, mut last_value) = initial_poll;

        loop {
            let (value, sent, replied) = self.request().await;
            if value != last_value {
                // Generate Observation!
                observation_out.send(Observation {
                    interval: Interval(last_sent, replied),
                    definition_predicate: match &self.config.interpretation {
                        PollingInterpretation::Transition => DefinitionPredicate::Transition(last_value, value),
                        PollingInterpretation::Mutation => DefinitionPredicate::AllMut(value - last_value),
                        PollingInterpretation::Assignment => DefinitionPredicate::LastAssn(value),
                    },
                    source: self.name.clone(),
                    platform_metadata: PlatformMetadata::Square(SquareMetadata {
                        timestamp: sent // Use poll sent times as logical ordering.
                    }),
                }).await.unwrap();
            }

            if to_write.has_changed().unwrap() {
                to_write.mark_unchanged();
                // If some value waiting to write.
                // Write it - NOTE: UNSAFE!
                let sent_at = Utc::now();
                let to_value = to_write.borrow().unwrap().clone();

                self.write(to_value).await;
                (last_sent, last_value) = (sent_at, to_value);
            } else {
                (last_sent, last_value) = (sent, value);
            }
            // Schedule next poll.
            sleep(self.config.backoff).await;
        }
    }

    /// Processes the next polling request from the Square API.
    ///
    /// # Behavior
    ///
    /// - Issues a request to retrieve inventory count from the Square API
    /// - If the request is successful:
    ///   - Extracts the inventory counts for the `InStock` state.
    ///   - Aggregates these counts into a single `Value` and records the timestamps when the request
    ///     was sent and when the response was received.
    ///   - Returns a tuple containing the aggregated `Value`, the sent timestamp, and the reply
    ///     timestamp.
    /// - If the request encounters an error:
    ///   - Logs the error details.
    ///   - Waits for 200ms and retries the request in a loop until success.
    pub async fn request(&self) -> (Value, DateTime<Utc>, DateTime<Utc>) {
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


    /// Writes the provided value to Square API.
    ///
    /// # Arguments
    /// * `value` - A `Value` representing the new inventory state to be written.
    ///
    /// # Behavior
    ///
    /// - Constructs a `BatchChangeInventoryRequest` to update the inventory count in Square's API.
    /// - Attempts to send the constructed request to the Square API:
    ///   - If the request is successful:
    ///     - Logs the success and returns immediately.
    ///   - If the request encounters an error:
    ///     - Logs the error details.
    ///     - Waits for 200ms and retries the request in a loop until success.
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
                    info!("{} - Wrote Value: {}!", self.name, value);
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