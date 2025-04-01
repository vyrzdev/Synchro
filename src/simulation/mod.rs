use std::time::Duration;
use nexosim::ports::{EventBuffer, EventSlot};
use nexosim::simulation::{Mailbox, SimInit, Simulation};
use tai_time::MonotonicTime;
use network::network_delay::NetworkConnection;
use polling::safe_interface::SafePollingInterface;
use polling::safe_platform::SafePollingPlatform;
use record::interface::RecordInterface;
use record::platform::RecordPlatform;
use crate::simulation::polling::PollingInterpretation;
use crate::TruthRecord;
use crate::value::Value;

pub mod data;
pub mod polling;
pub mod record;
pub mod network;
pub mod messages;
mod interpreter;

fn build_model(truth_sink: &EventBuffer<TruthRecord>, found_slot: &EventSlot<Option<Value>>) -> Simulation {
    let mut network1_connection = NetworkConnection::new(10.0, 4.0);
    let network1_in = Mailbox::new();
    let mut network2_connection = NetworkConnection::new(10.0, 4.0);
    let network2_in = Mailbox::new();
    let mut network3_connection = NetworkConnection::new(10.0, 4.0);
    let mut network3_in = Mailbox::new();

    let mut platform1 = SafePollingPlatform::new(10, 10.0, 10.0);
    let platform1_in = Mailbox::new();
    platform1.truth_output.connect_sink(truth_sink);
    platform1.output.connect(NetworkConnection::input_1, &network1_in);
    let mut platform2 = SafePollingPlatform::new(10, 10.0, 10.0);
    let platform2_in = Mailbox::new();
    platform2.output.connect(NetworkConnection::input_1, &network2_in);
    platform2.truth_output.connect_sink(truth_sink);
    let mut platform3 = RecordPlatform::new(100.0, 0.0, 10.0, 10.0);
    let platform3_in = Mailbox::new();
    platform3.truth_output.connect_sink(truth_sink);
    platform3.output.connect(NetworkConnection::input_1, &network3_in);



    let initial_state = CompletedPoll {
        send: MonotonicTime::EPOCH,
        receive: MonotonicTime::EPOCH,
        value: 10
    };
    let mut interface3 = RecordInterface::new("Record3".to_string(), Duration::from_millis(100));
    let mut interface3_platform_in = Mailbox::new();
    interface3.request_output.connect(NetworkConnection::input_2, &network3_in);

    let mut poller1 = SafePollingInterface::new("Poller1".to_string(), Duration::from_millis(100), PollingInterpretation::Transition, initial_state.clone());
    let mut poller1_platform_in = Mailbox::new();
    poller1.request_output.connect(NetworkConnection::input_2, &network1_in);

    let mut poller2 = SafePollingInterface::new("Poller2".to_string(), Duration::from_millis(100), PollingInterpretation::Transition, initial_state.clone());
    let mut poller2_platform_in = Mailbox::new();
    poller2.request_output.connect(NetworkConnection::input_2, &network2_in);

    // let mut interpreter = Interpreter::new(10);
    // let mut interpreter_in = Mailbox::new();
    // poller1.observation_output.connect(Interpreter::input, &interpreter_in);
    // poller2.observation_output.connect(Interpreter::input, &interpreter_in);
    // interface3.observation_output.connect(Interpreter::input, &interpreter_in);
    // interpreter.found_out.connect_sink(found_slot);
    // interpreter.found_out.connect(SafePollingInterface::write_input, &poller1_platform_in);
    // interpreter.found_out.connect(SafePollingInterface::write_input, &poller2_platform_in);

    network1_connection.output_1.connect(SafePollingInterface::reply_input, &poller1_platform_in);
    network1_connection.output_2.connect(SafePollingPlatform::input, &platform1_in);
    network2_connection.output_1.connect(SafePollingInterface::reply_input, &poller2_platform_in);
    network2_connection.output_2.connect(SafePollingPlatform::input, &platform2_in);
    network3_connection.output_1.connect(RecordInterface::reply_input, &interface3_platform_in);
    network3_connection.output_2.connect(RecordPlatform::input, &platform3_in);

    let t0 = MonotonicTime::EPOCH; // arbitrary start time
    // let mut simu = SimInit::new()
    //     .add_model(network1_connection, network1_in, "Network1")
    //     .add_model(platform1, platform1_in, "Platform1")
    //     .add_model(poller1, poller1_platform_in, "Poller1")
    //     .add_model(network2_connection, network2_in, "Network2")
    //     .add_model(platform2, platform2_in, "Platform2")
    //     .add_model(poller2, poller2_platform_in, "Poller2")
    //     .add_model(network3_connection, network3_in, "Network3")
    //     .add_model(interface3, interface3_platform_in, "Interface3")
    //     .add_model(platform3, platform3_in, "Platform3")
    //     .add_model(interpreter, interpreter_in, "Interpreter")
    //     .init(t0).unwrap()
    //     .0;
    let mut model = SimInit::new()
        .add_model(network1_connection, network1_in, "Network1")
        .add_model(platform1, platform1_in, "Platform1")
        .add_model(poller1, poller1_platform_in, "Poller1")
        .add_model(network2_connection, network2_in, "Network2")
        .add_model(platform2, platform2_in, "Platform2")
        .add_model(poller2, poller2_platform_in, "Poller2")
        .add_model(network3_connection, network3_in, "Network3")
        .add_model(interface3, interface3_platform_in, "Interface3")
        .add_model(platform3, platform3_in, "Platform3");
    // .add_model(interpreter, interpreter_in, "Interpreter");
    let simu =  model.init(t0).unwrap().0;
    return simu;
}