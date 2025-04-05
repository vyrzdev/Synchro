use serde::{Deserialize, Serialize};
use crate::real_world::config::RealWorldConfig;
use crate::simulation::config::SimulationConfig;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Config {
    RealWorld(RealWorldConfig),
    Simulation(Vec<SimulationConfig>),
}