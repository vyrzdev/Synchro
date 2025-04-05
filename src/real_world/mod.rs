use std::future::join;
use chrono::{Utc};
use log::info;
use tokio::task::JoinSet;
use tokio::sync::mpsc::{channel};
use tokio::sync::watch;
use crate::real_world::config::{PlatformConfig, RealWorldConfig};
use crate::real_world::interpreter::interpreter_worker;
use crate::real_world::square::polling::{SquarePollingInterface};
use crate::real_world::square::record::{SquareRecordInterface};

pub mod square;
pub mod interpreter;
pub mod config;


pub async fn real_world_main(cfg: RealWorldConfig) {
    // Initialise interpreter channels
    let (interpreter_tx, mut interpreter_rx) = channel(10);
    let (value_tx, value_rx) = watch::channel(None);

    // Start thread pools
    let mut polling_futures = JoinSet::new();
    let mut record_futures = JoinSet::new();

    // for each platform configured- initialise thread...
    for (name,platform_cfg) in cfg.platforms {
        info!("Discovered {}!", name);
        match platform_cfg {
            PlatformConfig::Polling(square_cfg) => {
                let mut new_interface = SquarePollingInterface::new(name, square_cfg);
                new_interface.write(cfg.initial_value.clone()).await;

                let local_rx = value_rx.clone();
                let local_tx = interpreter_tx.clone();
                let local_initial_value = cfg.initial_value.clone();

                polling_futures.spawn((async move || { // Move local copies into future.
                    new_interface.poll_worker(local_rx, local_tx, (Utc::now(), 0)).await;
                })());
            },
            PlatformConfig::Records(square_cfg) => {
                let mut new_interface = SquareRecordInterface::new(name, square_cfg).await;
                // TODO: Offset Worker.
                new_interface.write(cfg.initial_value.clone()).await;
                let local_rx = value_rx.clone();
                let local_tx = interpreter_tx.clone();

                record_futures.spawn((async move || { // Move local copies into future.
                    new_interface.record_worker(local_rx, local_tx).await;
                })());
            },
        }
    }

    info!("Initialising Interpreter");
    // Initialise interpreter
    let interpreter_future = interpreter_worker(interpreter_rx, cfg.initial_value, value_tx);

    info!("Starting!");
    // Join all threads - run until termination.
    join!(polling_futures.join_all(), interpreter_future).await;
}