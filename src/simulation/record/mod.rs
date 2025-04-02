// pub mod platform;
// pub mod interface;

pub mod messages;
pub mod platform;
pub mod interface;

use nexosim::model::{BuildContext, Model, ProtoModel};
use nexosim::ports::Output;
use nexosim::simulation::Mailbox;
use tai_time::MonotonicTime;
use crate::observations::Observation;
use crate::simulation::driver::TruthRecord;
use crate::simulation::messages::InterfaceQuery::Record;
use crate::simulation::network::network_delay::{NetworkConnection, NetworkParameters};
use crate::simulation::polling::r#unsafe::interface::UnsafePollingInterface;
use crate::simulation::polling::r#unsafe::platform::UnsafePollingPlatform;
use crate::simulation::record::interface::{RecordInterface, RecordInterfaceParameters};
use crate::simulation::record::platform::{RecordPlatform, RecordPlatformParameters};
use crate::simulation::user::user::User;
use crate::simulation::user::UserParameters;
use crate::value::Value;

pub struct RecordModel {}

#[derive(Clone, Debug)]
pub struct RecordConfig {
    pub(crate) network_params: NetworkParameters,
    pub(crate) interface_params: RecordInterfaceParameters,
    pub(crate) platform_params: RecordPlatformParameters,
    pub(crate) user_params: UserParameters,
}

impl Model for RecordModel {}


pub struct ProtoRecordModel {
    name: String,
    config: RecordConfig,
    pub observation_output: Output<Observation<MonotonicTime>>,
    pub truth_output: Output<TruthRecord>,
}

impl ProtoRecordModel {
    pub fn new(name: String, config: RecordConfig) -> ProtoRecordModel {
        ProtoRecordModel {
            name,
            config,
            observation_output: Default::default(),
            truth_output: Default::default(),
        }
    }
}

impl ProtoModel for ProtoRecordModel {
    type Model = RecordModel;

    fn build(self, cx: &mut BuildContext<Self>) -> Self::Model {
        let model = RecordModel {};

        // Initialise Platform Model
        let mut platform = RecordPlatform::new(self.name.clone(), self.config.platform_params);
        let platform_in = Mailbox::new();

        let mut interface = RecordInterface::new(self.name.clone(), self.config.interface_params);
        let interface_in = Mailbox::new();

        let mut network_connection = NetworkConnection::new(self.config.network_params);
        let network_in = Mailbox::new();

        let mut user = User::new(self.config.user_params);
        let user_in =Mailbox::new();

        // Connect user's output to platform's input.
        user.action_output.connect(RecordPlatform::input, &platform_in);

        // Connect platform reply out to interface input.
        platform.reply_output.connect(NetworkConnection::input_1, &network_in);
        network_connection.output_1.connect(RecordInterface::input, &interface_in);

        // Connect interface query out to platform input.
        interface.query_output.connect(NetworkConnection::input_2, &network_in);
        network_connection.output_2.connect(RecordPlatform::input, &platform_in);

        // Move External Truth Output into Submodel.
        platform.truth_output = self.truth_output;

        // Move External Observation Output into Submodel.
        interface.observation_output = self.observation_output;

        // Register Submodels.
        cx.add_submodel(platform, platform_in, format!("RECORDPLATFORM-{}", self.name.clone()).as_str());
        cx.add_submodel(network_connection, network_in, format!("NETWORK-{}", self.name.clone()).as_str());
        cx.add_submodel(interface, interface_in, format!("RECORDINTERFACE-{}", self.name.clone()).as_str());
        cx.add_submodel(user, user_in, format!("USER-{}", self.name).as_str());
        model
    }
}

//pub struct UnsafePollingModel {
//     internal_write_output: Output<Result<Value, ConflictError<MonotonicTime>>>,
// }
//
// #[derive(Clone, Debug)]
// pub struct UnsafePollingConfig {
//     pub(crate) initial_value: Value,
//     pub(crate) network_params: NetworkParameters,
//     pub(crate) interface_params: PollingInterfaceParameters,
//     pub(crate) user_params: UserParameters,
// }
//
// impl UnsafePollingModel {
//     pub fn new() -> Self {
//         UnsafePollingModel {
//             internal_write_output: Default::default(),
//         }
//     }
//
//     pub async fn write_input(&mut self, write: Result<Value, ConflictError<MonotonicTime>>) {
//         self.internal_write_output.send(write).await;
//     }
// }
// impl Model for UnsafePollingModel {}
//
// pub struct ProtoUnsafePollingModel {
//     name: String,
//     config: UnsafePollingConfig,
//     pub observation_output: Output<Observation<MonotonicTime>>,
//     pub truth_output: Output<TruthRecord>,
// }
// impl ProtoUnsafePollingModel {
//     pub fn new(name: String, initial_value: Value, config: UnsafePollingConfig) -> ProtoUnsafePollingModel {
//         ProtoUnsafePollingModel {
//             name,
//             observation_output: Default::default(),
//             truth_output: Default::default(),
//             config,
//         }
//     }
// }
//
// impl ProtoModel for ProtoUnsafePollingModel {
//     type Model = UnsafePollingModel;
//
//     fn build(mut self, cx: &mut BuildContext<Self>) -> Self::Model {
//         let mut model = UnsafePollingModel::new();
//         // Initialise Platform Model
//         let mut platform = UnsafePollingPlatform::new(self.name.clone(), self.config.initial_value);
//         let platform_in = Mailbox::new();
//
//         // Initialise Network Connection Model
//         let mut network_connection = NetworkConnection::new(self.config.network_params);
//         let network_in = Mailbox::new();
//
//         // Initialise Polling Interface Model
//         let mut interface = UnsafePollingInterface::new(self.name.clone(), self.config.interface_params, self.config.initial_value);
//         let interface_in = Mailbox::new();
//
//         // Initialise User
//         let mut user = User::new(self.config.user_params);
//         let user_in = Mailbox::new();
//
//         // Connect user's output to platform's input.
//         user.action_output.connect(UnsafePollingPlatform::input, &platform_in);
//
//         // Connect platform reply out to interface input.
//         platform.reply_output.connect(NetworkConnection::input_1, &network_in);
//         network_connection.output_1.connect(UnsafePollingInterface::platform_input, &interface_in);
//
//         // Connect interface query out to platform input.
//         interface.query_output.connect(NetworkConnection::input_2, &network_in);
//         network_connection.output_2.connect(UnsafePollingPlatform::input, &platform_in);
//
//         // Connect internal write output to interface input
//         model.internal_write_output.connect(UnsafePollingInterface::interpreter_input, &interface_in);
//
//         // Move External Truth Output into Submodel.
//         platform.truth_output = self.truth_output;
//
//         // Move External Observation Output into Submodel.
//         interface.observation_output = self.observation_output;
//         // interface = self.observation_output;
//
//         // Register Submodels.
//         cx.add_submodel(platform, platform_in, format!("UNSAFEPOLL-{}", self.name.clone()).as_str());
//         cx.add_submodel(network_connection, network_in, format!("NETWORK-{}", self.name.clone()).as_str());
//         cx.add_submodel(interface, interface_in, format!("UNSAFEINTERFACE-{}", self.name.clone()).as_str());
//         cx.add_submodel(user, user_in, format!("USER-{}", self.name).as_str());
//         model
//     }
// }