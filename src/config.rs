use crate::real_world::config::RealWorldConfig;
use crate::simulation::config::SimulationConfig;



pub enum Config {
    RealWorld(RealWorldConfig),
    Simulation(Vec<SimulationConfig>),
}