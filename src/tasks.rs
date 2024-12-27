#![allow(unused)]
use crate::{chart::Linear, EstimationConfig, Log, Results, Simulation};

const AMOUNT_OF_RUNS: usize = 10;

pub fn task32(config: EstimationConfig) {
    let simulation_config = config.simulation;
    let mut total_logs = Log::empty();
    let gap_size = 180;

    for i in 0..config.experiment.total {
        let mut simulation = Simulation::with_config(simulation_config.clone());
        let (_result, run_log) = simulation.run();
        total_logs.add_mut(run_log);
    }

    total_logs.norm_mut(config.experiment.total);

    Linear::from_data(
        "Plot name",
        total_logs.iter()
            .skip(180)
            .map(|(tick, _)| tick as f32).collect(),

        total_logs
            .iter()
            .skip(180)
            .map(|(_, entry)| entry.average_busy_tables)
            .collect(),
    )
    .use_approximation(false)
    .set_config(&config.stats)
    .save("penis.")
    .unwrap();
}
