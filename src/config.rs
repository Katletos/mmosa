use std::{
    fmt::{Display, Formatter},
    ops::Range,
};

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub workers: u32,
    pub tables: u32,
    pub max_time: u32,
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
