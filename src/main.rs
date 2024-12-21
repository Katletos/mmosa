mod config;
mod event;
mod results;
mod simulation;

pub use config::SimulationConfig;
pub use event::Event;
pub use results::Results;
pub use simulation::Simulation;

fn main() {
    env_logger::builder().init();

    let mut results = Results::zeros();
    let iter_count = std::env::var("ITER_COUNT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100);

    let config = SimulationConfig {
        workers: 1,
        tables: 10,
        max_time: 1000,
        client_ratio: 0.95,
        production_time: 1..2,
        dancing_time: 1..2,
        consumption_time: 1..2,
    };

    for _ in 0..iter_count {
        let sim = Simulation::with_config(config.clone());
        results.add_mut(sim.run());
    }

    results.norm_mut(iter_count);

    println!("{results}");
    println!("Configuration: {config}");
}
