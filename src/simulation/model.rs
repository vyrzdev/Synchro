use nexosim::ports::{EventBuffer, EventSlot};
use nexosim::simulation::{Mailbox, SimInit, Simulation};
use tai_time::MonotonicTime;
use crate::core::interpreter::error::ConflictError;
use crate::simulation::config::{PlatformConfig, SimulationConfig};
use crate::simulation::driver::TruthRecord;
use crate::simulation::interpreter::interpreter::{Interpreter, InterpreterConfig};
use crate::simulation::polling::r#unsafe::{ProtoUnsafePollingModel, UnsafePollingModel};
use crate::simulation::polling::safe::{ProtoSafePollingModel, SafePollingModel};
use crate::simulation::record::ProtoRecordModel;
use crate::value::Value;

pub fn build_model(
    cfg: &SimulationConfig,
    truth_sink: &EventBuffer<TruthRecord>,
    found_slot: &EventSlot<Result<Value, ConflictError<MonotonicTime>>>
) -> Simulation {
    let t0 = MonotonicTime::EPOCH; // Start at EPOCH!
    let mut model = SimInit::new();

    let mut interpreter = Interpreter::new(InterpreterConfig {
        initial_value: cfg.initial_value
    });
    let interpreter_in = Mailbox::new();
    interpreter.found_out.connect_sink(found_slot);


    for (name, polling_cfg) in cfg.platforms.iter() {
        match polling_cfg {
            PlatformConfig::PollingSafe(safe_cfg) => {
                let mut polling_model = ProtoSafePollingModel::new(name.clone(), safe_cfg.clone());
                let polling_mbox = Mailbox::new();

                // Attach truth output.
                polling_model.truth_output.connect_sink(truth_sink);

                // Attach interface to interpreter.
                polling_model.observation_output.connect(Interpreter::input, &interpreter_in);

                // Attach interpreter to interface
                interpreter.found_out.connect(SafePollingModel::write_input, &polling_mbox);

                model = model.add_model(polling_model, polling_mbox, format!("SafePolling-{}", name))
            }
            PlatformConfig::PollingUnsafe(unsafe_cfg) => {
                let mut polling_model = ProtoUnsafePollingModel::new(name.clone(), unsafe_cfg.clone());
                let polling_mbox = Mailbox::new();

                // Attach truth output.
                polling_model.truth_output.connect_sink(truth_sink);

                // Attach interface to interpreter
                polling_model.observation_output.connect(Interpreter::input, &interpreter_in);

                // Attach interpreter to interface
                interpreter.found_out.connect(UnsafePollingModel::write_input, &polling_mbox);
                model = model.add_model(polling_model, polling_mbox, format!("UnsafePolling-{}", name))
            }
            PlatformConfig::Record(record_cfg) => {
                let mut record_model = ProtoRecordModel::new(name.clone(), record_cfg.clone());
                let record_mbox = Mailbox::new();

                // Attach truth output.
                record_model.truth_output.connect_sink(truth_sink);

                // Attach interface to interpreter
                record_model.observation_output.connect(Interpreter::input, &interpreter_in);

                // We do not model writes for record- as they do not effect visibility.
                model = model.add_model(record_model, record_mbox, format!("Record-{}", name))
            }
            _ => unreachable!()
        }
    }

    model = model.add_model(interpreter, interpreter_in, "Interpreter");

    // Construct Simulation
    let (simu, _) =  model.init(t0).unwrap();
    return simu;
}