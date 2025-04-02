use crate::simulation::polling::r#unsafe::messages::UnsafePollQuery;
use crate::simulation::polling::safe::messages::SafePollQuery;
use crate::simulation::record::messages::RecordQuery;
use crate::value::Value;

#[derive(Debug, Clone)]
pub enum PlatformQuery {
    User(UserAction),
    Interface(InterfaceQuery)
}

#[derive(Clone, Copy, Debug)]
pub enum UserAction {
    Mutation(Value),
    Assignment(Value)
}

#[derive(Debug, Clone)]
pub enum InterfaceQuery {
    PollingSafe(SafePollQuery),
    PollingUnsafe(UnsafePollQuery),
    Record(RecordQuery)
}