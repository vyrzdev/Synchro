use serde::{Deserialize, Serialize};
use crate::real_world::config::RealWorldConfig;
use crate::simulation::config::SimulationConfig;

/// Root Config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Config {
    /// Real World Config
    RealWorld(RealWorldConfig),
    /// Simulation Config
    Simulation(Vec<SimulationConfig>),
}