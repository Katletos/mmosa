use std::{
    fmt::{Display, Formatter},
    ops::Range,
};

use crate::{scenario::ScenarioConfig, statistic::StatsConfig, ExperimentConfig};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimulationConfig {
    pub workers: u32,
    pub tables: u32,
    pub max_time: u32,
    /// the probability that client will appereaed in simulation tick
    pub client_ratio: f64,
    pub production_time: Range<u32>,
    pub dancing_time: Range<u32>,
    pub consumption_time: Range<u32>,
    pub use_logs: bool,
}

impl Display for SimulationConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EstimationConfig {
    pub simulation: SimulationConfig,
    pub stats: StatsConfig,
    pub scenario: Option<ScenarioConfig>,
    pub experiment: ExperimentConfig,
}

impl Display for EstimationConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
