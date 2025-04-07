use std::fs;
use std::path::{Path, PathBuf};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Handle;
use log4rs::Config as LogConfig;
use log::{info, LevelFilter};
use crate::config::Config;
use crate::simulation;
use crate::utils::output_path;

/// Runs a simulation based on the provided input path and number of iterations.
///
/// - If the `input_path` points to a directory, the function iterates through each
///   file in the directory and runs simulations for them.
/// - If the `input_path` points to a file, a single simulation is executed for that file.
/// - The results and log files are generated in the `output` and `results` directories.
///
/// # Parameters
/// - `input_path`: A `PathBuf` representing the path to the input file or directory of files.
/// - `iterations`: The number of iterations to run the simulation for.
///
/// # Behavior
/// - Initializes directories for output and results.
/// - For `.json` files in the specified input directories, configures logging
///   and processes each file using the `run_simulation` function.
/// - Handles invalid or empty configurations and logs appropriate warnings.
///
/// # Panics
/// This function will panic if:
/// - It fails to create the output or results directories.
/// - It encounters errors while reading the input path or configuring logging.
pub async fn simulate(input_path: PathBuf, iterations: u64) {
    // Create output Directory
    fs::create_dir_all("output").unwrap();

    let mut log_handle: Option<Handle> = None;
    if input_path.is_dir() {
        for entry in fs::read_dir(input_path.clone()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            run_simulation(path, &mut log_handle, iterations).await;
        }
    } else {
        run_simulation(input_path, &mut log_handle, iterations).await;
    }
}


/// Runs a single simulation based on the given input path and number of iterations.
///
/// # Parameters
/// - `input_path`: A `PathBuf` representing the path to the input file.
/// - `log_handle`: A mutable reference to an `log4rs::Handle` for managing log configurations.
/// - `iterations`: A `u64` indicating the number of iterations to run the simulation.
///
/// # Behavior
/// - Generates an output path where logs will be stored.
/// - Reads and parses the input `.json` file into a simulation configuration.
/// - Configures logging for the simulation process to the specified output log file.
/// - Runs the simulation only if the configuration is valid
///
/// # Panics
/// This function will panic if:
/// - The input file cannot be read.
/// - The input file contains an invalid or improperly formatted configuration.
/// - Errors occur during log configuration setup.
pub async fn run_simulation(input_path: PathBuf, log_handle: &mut Option<Handle>, iterations: u64) {
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

    // Read and parse the contents of the input file
    let contents = fs::read_to_string(&input_path).unwrap(); // This will panic if the file cannot be read
    info!("Reading: {input_path:?}");

    // Deserialize JSON into Config
    let config: Config = serde_json::from_str(&contents)
        .expect(&*("Failed to parse config ".to_owned() + input_path.display().to_string().as_str())); // Panic if the JSON is improperly formatted or invalid

    let config: Config = serde_json::from_str(&contents)
        .expect(&*("Failed to parse config ".to_owned() + input_path.display().to_string().as_str()));

    // unpack config and start sim.
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
