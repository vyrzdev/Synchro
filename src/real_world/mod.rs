use chrono::{DateTime, Utc};
use tokio::task::JoinSet;
use tokio::sync::mpsc::{channel, Sender, Receiver};
use crate::observations::Observation;
use crate::real_world::config::{PlatformConfig, RealWorldConfig};
use crate::real_world::square::polling::{SquarePollingConfig, SquarePollingInterface};
use crate::real_world::square::record::{SquareRecordConfig, SquareRecordInterface};
use crate::value::Value;

pub mod square;
pub mod interpreter;
pub mod config;

pub async fn real_world_main(cfg: RealWorldConfig) {
    let (interpreter_tx, interpreter_rx) = channel(10);
    let (value_tx, value_rx) = channel(10);
    let futures = JoinSet::new();

    for (name,platform_cfg) in cfg.platforms {
        match platform_cfg {
            PlatformConfig::Polling(square_cfg) => build_polling_platform(name, square_cfg, value_rx.clone(), interpreter_tx.clone(), cfg.initial_value),
            PlatformConfig::Records(square_cfg) => build_record_platform(name, square_cfg, value_rx.clone(), interpreter_tx.clone(), cfg.initial_value),
        }
    }
}

pub async fn build_record_platform(
    name: String,
    cfg: SquareRecordConfig,
    to_write: Receiver<Value>,
    observation_out: Sender<Observation<DateTime<Utc>>>,
    initial_value: Value,
) -> () {
    let mut new_interface = SquareRecordInterface::new(name, cfg);
    // TODO: Offset Worker.
    new_interface.write(initial_value).await;
    return new_interface.record_worker(observation_out).await;
}
pub async fn build_polling_platform(
    name: String,
    cfg: SquarePollingConfig,
    to_write: Receiver<Value>,
    observation_out: Sender<Observation<DateTime<Utc>>>,
    initial_value: Value
) -> () {
    let mut new_interface = SquarePollingInterface::new(name, cfg);
    new_interface.write(initial_value).await;
    return new_interface.poll_worker(to_write, observation_out, (Utc::now(), initial_value))
}