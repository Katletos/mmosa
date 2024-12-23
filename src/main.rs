mod chart;
mod config;
mod event;
mod experiment;
mod history;
mod results;
mod scenario;
mod simulation;
mod statistic;

use std::fs;

pub use config::{EstimationConfig, SimulationConfig};
pub use event::Event;
pub use experiment::ExperimentConfig;
pub use history::Log;
pub use results::Results;
pub use simulation::{Simulation, SimulationTick};
pub use statistic::Stats;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let config = {
        let raw_config = std::fs::read_to_string("config.toml")
            .expect("Failed to read config");

        toml::from_str::<EstimationConfig>(&raw_config)
            .expect("Failed to parse config")
    };

    let directories = vec![
        "stats/logs",
        "stats/multi",
        "stats/scenario",
        "stats/single_run",
    ];

    for dir_path in directories.into_iter() {
        if fs::metadata(dir_path).is_ok() {
            fs::remove_dir_all(dir_path)?;
        }
        fs::create_dir_all(dir_path)?;
    }

    experiment::run(config.clone());

    log::info!("Experiment is finished");

    if config.scenario.is_some() {
        scenario::run(config);
        log::info!("Scenario is finished");
    }

    Ok(())
}
