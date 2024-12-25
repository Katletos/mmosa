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
use rayon::iter::{IntoParallelIterator, ParallelIterator};
pub use results::Results;
use scenario::{ScenarioConfig, ScenarioParameter};
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
        "stats/3_1",
        "stats/3_2",
        "stats/3_3",
        "stats/3_4",
        "stats/3_5",
        "stats/3_6",
        "stats/4_1",
        "stats/4_2",
        "stats/4_3",
    ];

    for dir_path in directories.into_iter() {
        if fs::metadata(dir_path).is_ok() {
            fs::remove_dir_all(dir_path)?;
        }
        fs::create_dir_all(dir_path)?;
    }

    // experiment::run(config.clone());

    log::info!("Experiment is finished");

    // if config.scenario.is_some() {
    //     scenario::run(config);
    //     log::info!("Scenario is finished");
    // }

    task_3_1(&config);
    task_3_2(&config);
    task_3_3(&config);
    task_3_4(&config);
    task_3_5(&config);
    // task_3_6(&config);

    Ok(())
}

fn task_3_1(config: &EstimationConfig) {
    let mut total_results = Results::zeros();
    let mut total_logs = Log::empty();
    let mut results = Vec::<Results>::new();

    let tmp = (0..config.experiment.total)
        .into_par_iter()
        .map(|_| {
            let mut sim = Simulation::with_config(config.simulation.clone());
            sim.run()
        })
        .collect::<Vec<_>>();

    tmp.into_iter().for_each(|(run_result, run_log)| {
        total_results.add_mut(run_result.clone());
        total_logs.add_mut(run_log);

        results.push(run_result);
    });

    total_results.norm_mut(config.experiment.total);
    total_logs.norm_mut(config.experiment.total);

    chart::Histogram::from_y_data(
        "Average busy tables",
        results.iter().map(|r| r.average_busy_tables).collect(),
    )
    .save("stats/3_1/BusyTables", &config.stats)
    .unwrap();

    chart::Histogram::from_y_data(
        "Average free workers",
        results.iter().map(|r| r.average_free_workers).collect(),
    )
    .save("stats/3_1/FreeWorkers", &config.stats)
    .unwrap();

    chart::Histogram::from_y_data(
        "Immediate left client",
        results.iter().map(|r| r.dispatched_clients).collect(),
    )
    .save("stats/3_1/DispatchedClients", &config.stats)
    .unwrap();
}

// Интервальная оценка двух непрерывных и одного дискретного откликов
fn task_3_2(config: &EstimationConfig) {
    let total = 10;
    let mut results = vec![];

    for _ in 0..total {
        let mut sim = Simulation::with_config(config.simulation.clone());
        let (run_result, _run_log) = sim.run();
        results.push(run_result);
    }

    chart::Linear::from_data(
        "BusyTables over Time",
        (0..total).map(|v| v as f32).collect(),
        results.iter().map(|r| r.average_busy_tables).collect(),
    )
    .use_approximation(false)
    .set_config(&config.stats)
    .save("stats/3_2/BusyTables")
    .unwrap();

    chart::Linear::from_data(
        "Free Workers over Time",
        (0..total).map(|v| v as f32).collect(),
        results.iter().map(|r| r.average_free_workers).collect(),
    )
    .use_approximation(false)
    .set_config(&config.stats)
    .save("stats/3_2/FreeWorkers")
    .unwrap();

    chart::Linear::from_data(
        "Dispatched client over Time",
        (0..total).map(|v| v as f32).collect(),
        results.iter().map(|r| r.dispatched_clients).collect(),
    )
    .use_approximation(false)
    .set_config(&config.stats)
    .save("stats/3_2/DispatchedClients")
    .unwrap();
}

fn task_3_3(config: &EstimationConfig) {
    let total = 100;
    let window_size = 2;
    let mut results = vec![];

    for _ in 0..(total + window_size) {
        let mut sim = Simulation::with_config(config.simulation.clone());
        let (run_result, _run_log) = sim.run();
        results.push(run_result);
    }

    let mut values = vec![];

    for i in window_size..(total + window_size) {
        let window = results[0..i].to_vec();
        values.push(window);
    }

    chart::Linear::from_data(
        "BusyTables over Time",
        (0..total).map(|v| v as f32).collect(),
        values
            .iter()
            .map(|data| {
                let processed = data
                    .iter()
                    .map(|r| r.average_busy_tables as f64)
                    .collect::<Vec<_>>();

                let stats = Stats::new(&processed, &config.stats);
                (2.0 * stats.std_dev * stats.t_stat) as f32
                    / (data.len() as f32).sqrt()
            })
            .collect(),
    )
    .save("stats/3_3/BusyTables")
    .unwrap();

    chart::Linear::from_data(
        "FreeWorkers over Time",
        (0..total).map(|v| v as f32).collect(),
        values
            .iter()
            .map(|data| {
                let processed = data
                    .iter()
                    .map(|r| r.average_free_workers as f64)
                    .collect::<Vec<_>>();

                let stats = Stats::new(&processed, &config.stats);
                (2.0 * stats.std_dev * stats.t_stat) as f32
                    / (data.len() as f32).sqrt()
            })
            .collect(),
    )
    .save("stats/3_3/FreeWorkers")
    .unwrap();

    chart::Linear::from_data(
        "DispatchedClients over Time",
        (0..total).map(|v| v as f32).collect(),
        values
            .iter()
            .map(|data| {
                let processed = data
                    .iter()
                    .map(|r| r.dispatched_clients as f64)
                    .collect::<Vec<_>>();

                let stats = Stats::new(&processed, &config.stats);
                (2.0 * stats.std_dev * stats.t_stat) as f32
                    / (data.len() as f32).sqrt()
            })
            .collect(),
    )
    .save("stats/3_3/DispatchedClients")
    .unwrap();
}

fn task_3_4(config: &EstimationConfig) {
    let mut config = config.clone();
    config.scenario = Some(ScenarioConfig {
        parameters: vec![ScenarioParameter {
            kind: scenario::ParameterKind::Clients,
            values: 30..80,
            step: 5,
        }],
    });

    scenario::run(config);
}

fn task_3_5(config: &EstimationConfig) {
    let mut config = config.clone();
    config.experiment.continous = false;
    config.experiment.total = 10_000;

    experiment::run(config);
}

fn task_3_6(config: &EstimationConfig) {
    let mut config = config.clone();
    config.experiment.continous = true;
    config.experiment.total = 10_000;

    experiment::run(config);
}
