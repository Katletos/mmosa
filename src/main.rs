mod chart;
mod config;
mod event;
mod results;
mod simulation;

use chart::Histogram;
pub use config::{EstimationConfig, SimulationConfig};
pub use event::Event;
pub use results::Results;
pub use simulation::Simulation;

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

    let mut total_results = Results::zeros();
    let mut results = Vec::<Results>::new();

    for _ in 0..config.total {
        let sim = Simulation::with_config(config.simulation.clone());
        let run_result = sim.run();

        total_results.add_mut(run_result.clone());
        results.push(run_result);
    }

    Histogram::from_y_data(
        "Average Waiting time",
        results
            .iter()
            .map(|r| r.average_worker_waiting_time)
            .collect(),
    )
    .save("stats/WaitingTime.png")
    .unwrap();

    Histogram::from_y_data(
        "Average busy tables",
        results.iter().map(|r| r.average_busy_tables).collect(),
    )
    .save("stats/BusyTables.png")
    .unwrap();

    Histogram::from_y_data(
        "Average free workers",
        results.iter().map(|r| r.average_free_workers).collect(),
    )
    .save("stats/FreeWorkers.png")
    .unwrap();

    Histogram::from_y_data(
        "Immediate left client",
        results
            .iter()
            .map(|r| r.immediately_left_clients_count)
            .collect(),
    )
    .save("stats/ImmediateClients.png")
    .unwrap();

    Histogram::from_y_data(
        "Average order time",
        results
            .iter()
            .map(|r| r.average_order_time)
            .collect(),
    )
    .save("stats/OrderTime.png")
    .unwrap();

    Histogram::from_y_data(
        "Average consumption time",
        results
            .iter()
            .map(|r| r.average_consumption_time)
            .collect(),
    )
    .save("stats/ConsumptionTime.png")
    .unwrap();

    total_results.norm_mut(config.total);
}
