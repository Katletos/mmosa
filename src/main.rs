mod chart;
mod config;
mod event;
mod results;
mod scenario;
mod simulation;
mod statistic;

use chart::Histogram;
pub use config::{EstimationConfig, SimulationConfig};
pub use event::Event;
pub use results::Results;
pub use simulation::Simulation;
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

    let mut total_results = Results::zeros();
    let mut results = Vec::<Results>::new();

    for _ in 0..config.total {
        let sim = Simulation::with_config(config.simulation.clone());
        let run_result = sim.run();

        total_results.add_mut(run_result.clone());
        results.push(run_result);
    }

    total_results.norm_mut(config.total);

    std::fs::write("results.toml", toml::to_string(&total_results).unwrap())
        .unwrap();

    Histogram::from_y_data(
        "Average Waiting time",
        results
            .iter()
            .map(|r| r.average_worker_waiting_time)
            .collect(),
    )
    .save("stats/single_run/WaitingTime", &config.stats)
    .unwrap();

    Histogram::from_y_data(
        "Average busy tables",
        results.iter().map(|r| r.average_busy_tables).collect(),
    )
    .save("stats/single_run/BusyTables", &config.stats)
    .unwrap();

    Histogram::from_y_data(
        "Average free workers",
        results.iter().map(|r| r.average_free_workers).collect(),
    )
    .save("stats/single_run/FreeWorkers", &config.stats)
    .unwrap();

    Histogram::from_y_data(
        "Immediate left client",
        results
            .iter()
            .map(|r| r.immediately_left_clients_count)
            .collect(),
    )
    .save("stats/single_run/ImmediateClients", &config.stats)
    .unwrap();

    Histogram::from_y_data(
        "Average order time",
        results.iter().map(|r| r.average_order_time).collect(),
    )
    .save("stats/single_run/OrderTime", &config.stats)
    .unwrap();

    Histogram::from_y_data(
        "Average consumption time",
        results.iter().map(|r| r.average_consumption_time).collect(),
    )
    .save("stats/single_run/ConsumptionTime", &config.stats)
    .unwrap();

    if config.scenario.is_some() {
        scenario::run(config);
    }
}
