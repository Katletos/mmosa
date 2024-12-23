mod chart;
mod config;
mod event;
mod experiment;
mod history;
mod results;
mod scenario;
mod simulation;
mod statistic;

pub use config::{EstimationConfig, SimulationConfig};
pub use event::Event;
pub use experiment::ExperimentConfig;
pub use history::Log;
pub use results::Results;
pub use simulation::{Simulation, SimulationTick};
pub use statistic::Stats;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let config = {
        let raw_config = std::fs::read_to_string("config.toml")
            .expect("Failed to read config");

        toml::from_str::<EstimationConfig>(&raw_config)
            .expect("Failed to parse config")
    };

    chart::HyperPlane::from_data(vec![-1.0, 1.0], vec![-1.0, 1.0], vec![-1.0, 1.0], "HyperPlane")
        .save("3d.png")
        .unwrap();

    experiment::run(config.clone());

    log::info!("Experiment is finished");

    if config.scenario.is_some() {
        scenario::run(config);
        log::info!("Scenario is finished");
    }
}
