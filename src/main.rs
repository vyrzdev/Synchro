#![feature(linked_list_cursors)]
#![feature(future_join)]

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

mod value;
mod observations;
mod intervals;
mod interpreter;
mod real_world;
mod simulation;
mod predicates;
mod ordering;
mod config;
mod simulations;

fn output_path(output_folder: &PathBuf, input_path: &PathBuf) -> PathBuf {
    let candidate = output_folder.join(input_path.file_stem().unwrap()).with_extension("log");
    return resolve_unique_path(&candidate);
}


pub fn resolve_unique_path(original: &Path) -> PathBuf {
    if !original.exists() {
        return original.to_path_buf();
    }

    let stem = original.file_stem().unwrap_or_default().to_string_lossy();
    let ext = original.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let parent = original.parent().unwrap_or_else(|| Path::new(""));

    for i in 1.. {
        let candidate = parent.join(format!("{stem}_{i}{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!() // Compile please! (try taking it away)
}

// Testing Macro to make traces quickly...
macro_rules! make_observation {
    ($start:expr, $end:expr) => {
        Observation {
            interval: (
                MonotonicTime::new($start, 0).unwrap(),
                MonotonicTime::new($end, 0).unwrap(),
            ),
            definition_predicate: DefinitionPredicate::Unknown,
            source: "".to_string(),
            platform_metadata: PlatformMetadata::Polling { poll_count: 0 },
        }
    };
}
// Usage:
// let observations = vec![
//     make_observation!(0, 10),
//     ...
//     make_observation!(25, 65)
// ];



/// Simple program to greet a person
async fn simulate(input_path: PathBuf, log_handle: &mut Option<Handle>, iterations: u64) {
    // Generate output path (+1 each time)
    let output_path = output_path(&PathBuf::from("output"), &input_path);
    info!("Loading Simulation from: {input_path:?}");
    info!("Will output to {output_path:?}");


    // Initialise output logfile
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(output_path).unwrap();

    let logconfig = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
            .appender("logfile")
            .build(LevelFilter::Info)).unwrap();

    // Can only initialise a logger once per execution.
    if let Some(handle) = log_handle.as_ref() {
        handle.set_config(logconfig);
    } else {
        log_handle.replace(log4rs::init_config(logconfig).unwrap());
    }

    // Read and parse
    let contents = fs::read_to_string(&input_path).unwrap();
    info!("Reading: {input_path:?}");
    let config: Config = serde_json::from_str(&contents)
        .expect(&*("Failed to parse config ".to_owned() + input_path.display().to_string().as_str()));

    // Only run simulation if config matches
    if let Config::Simulation(cfg) = config {
        if let Some(first_cfg) = cfg.first() {
            let results = simulation::driver::driver(first_cfg.clone(), iterations);
            log::info!("Ended With Results {results:#?}");
            log::info!("Finished processing {}", input_path.display());
        } else {
            log::warn!("Simulation config in {} was empty", input_path.display());
        }
    } else {
        log::warn!("Config in {} was not a Simulation variant", input_path.display());
    }

}

async fn command_simulate(input_path: PathBuf, iterations: u64) {
    fs::create_dir_all("output").unwrap();
    let mut log_handle: Option<Handle> = None;
    if input_path.is_dir() {
        for entry in fs::read_dir(input_path.clone()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            simulate(path, &mut log_handle, iterations).await;
        }
    } else {
        simulate(input_path, &mut log_handle, iterations).await;
    }

    let input_dirs = vec!["scenarios/todo"];
    let output_dir = Path::new("results");

    fs::create_dir_all(output_dir).unwrap();

    let mut log_handle: Option<Handle> = None;

    for dir in input_dirs {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Build corresponding .log output path
                let filename = path.file_stem().unwrap(); // no extension
                // if filename.eq("SAFE_1000_BOTH_ALLMUT") {
                //     println!("FLAG");
                // }
                let log_path = output_dir.join(filename).with_extension("log");

                let logfile = FileAppender::builder()
                    .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
                    .build(log_path).unwrap();

                let logconfig = LogConfig::builder()
                    .appender(Appender::builder().build("logfile", Box::new(logfile)))
                    .build(Root::builder()
                        .appender("logfile")
                        .build(LevelFilter::Info)).unwrap();

                if let Some(handle) = log_handle.as_ref() {
                    handle.set_config(logconfig);
                } else {
                    log_handle = Some(log4rs::init_config(logconfig).unwrap());
                }


            }
        }
    }
}

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


#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Simulate { input_path, iterations } => {
            // Call your simulation logic here
            command_simulate(input_path.to_owned(), iterations.to_owned()).await;
        }

        Commands::Run { config_file } => {
            // Call your runtime logic here
            // run(config_file);
            colog::init(); // Prefer colog to log4rs in general- but log4rs supports files.
            info!("Loading real-world config at: {:?}", config_file);
            let contents = fs::read_to_string(&config_file).unwrap();
            let config: Config = serde_json::from_str(&contents)
                .expect(&*("Failed to parse config ".to_owned() + config_file.display().to_string().as_str()));

            if let Config::RealWorld(cfg) = config {
                real_world_main(cfg).await;
            } else {
                log::warn!("Config in {} was not a RealWorld variant", config_file.display());
            }
        }
    }

    // real_world_main();
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



    // File::create("sim_config.json").expect("Failed TO OPEN").write((&serde_json::to_string(&Config::Simulation(vec![make_demo_sim()])).expect("TODO: panic message")).as_ref());

}
