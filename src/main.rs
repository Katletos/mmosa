mod config;
mod event;
mod results;
mod simulation;

pub use config::{EstimationConfig, SimulationConfig};
pub use event::Event;
pub use results::Results;
pub use simulation::Simulation;

fn main() {
    env_logger::builder().init();

    let mut results = Results::zeros();

    let config = {
        let raw_config = std::fs::read_to_string("config.toml")
            .expect("Failed to read config");

        toml::from_str::<EstimationConfig>(&raw_config)
            .expect("Failed to parse config")
    };

    for _ in 0..config.total {
        let sim = Simulation::with_config(config.simulation.clone());
        results.add_mut(sim.run());
    }

    results.norm_mut(config.total);

    println!("{results}");
    println!("Configuration: {config}");
}
