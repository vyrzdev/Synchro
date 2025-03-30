pub mod unsafe_interface;
pub mod unsafe_platform;


pub enum PollingInterpretation {
    Transition,
    AllMut,
    LastAssn
}