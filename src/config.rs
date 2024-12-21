use std::{
    fmt::{Display, Formatter},
    ops::Range,
};

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
}

impl Display for SimulationConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EstimationConfig {
    pub simulation: SimulationConfig,
    /// total count of runs
    pub total: usize,
    pub continues: bool,
}

impl Display for EstimationConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
