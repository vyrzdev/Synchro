#![feature(linked_list_cursors)] // Used for history automata
#![feature(future_join)] // Join all futures.
use std::{fs};
use std::path::{Path, PathBuf};
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Root};
use log4rs::config::Config as LogConfig;
use log4rs::Handle;
use log::info;
use crate::config::Config;
use clap::{command, Parser, Subcommand};
use crate::real_world::config::RealWorldConfig;
use crate::real_world::real_world_main;
use crate::simulate::simulate;

mod value;
mod real_world;
mod simulation;
mod config;
mod simulate;
mod utils;
mod core;

#[derive(Parser)]
#[command(name = "synchro")]
#[command(about = "Simulation and Demonstration of Synchro", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a simulation using a config or directory of config files
    Simulate {
        /// Path to the config or directory of config files
        input_path: PathBuf,
        /// Number of iterations to run the simulator for each config.
        iterations: u64
    },

    /// Run the system using a specific configuration file
    Run {
        /// Path to the config file
        config_file: PathBuf,
    },
}

/// The main entry point for the application.
///
/// This function parses CLI arguments and invokes the appropriate command.
/// The application supports the following commands:
/// - `simulate`: Runs a simulation using a configuration file or directory of configuration files.
/// - `run`: Executes the system using a specific configuration file.
///
/// # Examples
/// ```bash
/// # To run a simulation:
/// ./synchro simulate /path/to/config(s)  100
///
/// # To run the system:
/// ./synchro run /path/to/config.json
/// ```
#[tokio::main]
async fn main() {
    let cli = Cli::parse(); // parse cli args

    match &cli.command {
        // If simulate command
        Commands::Simulate { input_path, iterations } => {
            simulate(input_path.to_owned(), iterations.to_owned()).await; // Run simulator
        }
        // If real-world command
        Commands::Run { config_file } => {
            colog::init(); // Prefer colog to log4rs in general- but log4rs supports files.
            info!("Loading real-world config at: {:?}", config_file);
            let contents = fs::read_to_string(&config_file).unwrap(); // read cfg
            let config: Config = serde_json::from_str(&contents) // parse cfg
                .expect(&*("Failed to parse config ".to_owned() + config_file.display().to_string().as_str()));

            if let Config::RealWorld(cfg) = config {
                real_world_main(cfg).await; // run realworld
            } else {
                log::warn!("Config in {} was not a RealWorld variant", config_file.display());
            }
        }
    }

    // HISTORY DEBUG PATH
    // =============================
    // colog::init();
    //
    // let mut history = History {
    //     list: LinkedList::from([Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(2, 723000000).unwrap(), MonotonicTime::new(5, 42000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling2".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 0 }) }], cached_definition: Some(AllMut(-1)) }, Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(3, 996000000).unwrap(), MonotonicTime::new(6, 108000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling2".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 1 }) }], cached_definition: Some(AllMut(-1)) }])
    // };
    //
    // let obs = Observation { interval: Interval(MonotonicTime::new(5, 42000000).unwrap(), MonotonicTime::new(7, 191000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling1".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 0 }) };
    //
    // // let obs = Observation {
    // //     interval: Interval(MonotonicTime::new(8, 395000000).unwrap(), MonotonicTime::new(8, 796000000).unwrap()),
    // //     definition_predicate: AllMut(-1), source: "Polling2".to_string().to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 2 }) };
    // // let mut history = History {
    // //     list: LinkedList::from([Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(2, 524000000).unwrap(), MonotonicTime::new(2, 822000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling1".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 0 }) }], cached_definition: Some(AllMut(-1)) }, Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(4, 245000000).unwrap(), MonotonicTime::new(4, 626000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling1".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 1 }) }], cached_definition: Some(AllMut(-1)) }, Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(4, 626000000).unwrap(), MonotonicTime::new(4, 992000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling1".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 2 }) }], cached_definition: Some(AllMut(-1)) }, Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(5, 802000000).unwrap(), MonotonicTime::new(6, 152000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling2".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 0 }) }], cached_definition: Some(AllMut(-1)) }, Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(6, 152000000).unwrap(), MonotonicTime::new(6, 486000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling2".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 1 }) }], cached_definition: Some(AllMut(-1)) }, Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(6, 498000000).unwrap(), MonotonicTime::new(6, 931000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling1".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 3 }) }], cached_definition: Some(AllMut(-1)) }, Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(6, 931000000).unwrap(), MonotonicTime::new(7, 294000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling1".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 4 }) }], cached_definition: Some(AllMut(-1)) }, Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(8, 97000000).unwrap(), MonotonicTime::new(8, 395000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling1".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 5 }) }], cached_definition: Some(AllMut(-1)) }, Region { observations: vec![Observation { interval: Interval(MonotonicTime::new(8, 395000000).unwrap(), MonotonicTime::new(8, 724000000).unwrap()), definition_predicate: AllMut(-1), source: "Polling1".to_string(), platform_metadata: Simulation(SimulationMetaData { monotonic: 6 }) }], cached_definition: Some(AllMut(-1)) }])
    // // };
    //
    // println!("---------- History Before ------------");
    // let mut i = 0;
    // for region in &history.list {
    //     println!("Region {i}");
    //     for observation in &region.observations {
    //         print!("{:?}", observation.definition_predicate);
    //         print!("{:?}.{:?}, {:?}.{:?} |", observation.interval.0.as_secs(), observation.interval.0.subsec_nanos(), observation.interval.1.as_secs(), observation.interval.1.subsec_nanos());
    //     }
    //     println!("");
    //     i += 1;
    // }
    //
    // // info!("History Before: {:#?}", history);
    // history.insert(obs, MonotonicTime::new(8, 796000000).unwrap());
    // // info!("History After: {:#?}", history);
    //
    // println!("---------- History After  ------------");
    // let mut i = 0;
    // for region in &history.list {
    //     println!("Region {i}");
    //     for observation in &region.observations {
    //         print!("{:?}", observation.definition_predicate);
    //         print!("{:?}.{:?}, {:?}.{:?} |", observation.interval.0.as_secs(), observation.interval.0.subsec_nanos(), observation.interval.1.as_secs(), observation.interval.1.subsec_nanos());
    //     }
    //     println!("");
    //     i += 1;
    // }
    //
    // panic!();
}
