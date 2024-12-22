use statrs::{
    distribution::{ContinuousCDF, StudentsT},
    statistics::Statistics,
};

use crate::{
    chart::{Histogram, Linear},
    EstimationConfig, Log, Results, Simulation,
};

#[derive(Clone, serde::Serialize)]
pub struct ExperimentResult {
    pub runs: Results,
    pub warmup_gap: Option<Gap>,
}

#[derive(Clone, serde::Serialize)]
pub enum Gap {
    Absolute(usize),
    MeanVariance(usize),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaseParameter {
    FreeWorkers,
    BusyTables,
    WaitingTime,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExperimentConfig {
    /// total count of runs
    pub total: usize,
    /// use continous experiment (warmed up state)
    pub continous: bool,
    pub parameter: BaseParameter,
}

pub fn run(config: EstimationConfig) {
    let mut total_results = Results::zeros();
    let mut total_logs = Log::empty();
    let mut results = Vec::<Results>::new();

    for _ in 0..config.experiment.total {
        let sim = Simulation::with_config(config.simulation.clone());
        let (run_result, run_log) = sim.run();

        total_results.add_mut(run_result.clone());
        total_logs.add_mut(run_log);

        results.push(run_result);
    }

    total_results.norm_mut(config.experiment.total);
    total_logs.norm_mut(config.experiment.total);

    assert!(config.experiment.total > 2, "At least 3 run must be set");

    let mut min_gap = None;

    let mut ts_values = vec![];
    let mut td_values = vec![];

    let logs_entries = total_logs
        .iter()
        .map(|(_tick, entry)| entry.clone())
        .collect::<Vec<_>>();

    let sim_duration = config.simulation.max_time as usize;
    assert_eq!(sim_duration, logs_entries.len());

    for gap_size in 1..(sim_duration / 2) {
        let mut big_ones = vec![]; //1 if higher all, 0 otherwise
        let mut small_ones = vec![]; //1 if lower all, 0 otherwisse

        let mut last_min = f32::MAX;
        let mut last_max = f32::MIN;

        (gap_size..sim_duration).for_each(|i| {
            let v = match config.experiment.parameter {
                BaseParameter::FreeWorkers => {
                    logs_entries[i].average_free_workers
                }
                BaseParameter::BusyTables => {
                    logs_entries[i].average_busy_tables
                }
                BaseParameter::WaitingTime => {
                    logs_entries[i].average_worker_waiting_time
                }
            };

            if v < last_min {
                last_min = v;
                small_ones.push(1.0f32);
            } else {
                small_ones.push(0.0f32);
            }

            if v > last_max {
                last_max = v;
                big_ones.push(1.0f32);
            } else {
                big_ones.push(0.0f32);
            }
        });

        let rest_sim_duration = (sim_duration - gap_size) as f64;

        let s = big_ones
            .iter()
            .zip(small_ones.iter())
            .map(|(k, l)| (k + l) as f64)
            .sum::<f64>();

        let d = big_ones
            .iter()
            .zip(small_ones.iter())
            .map(|(k, l)| (k - l) as f64)
            .sum::<f64>();

        let t_s = {
            let mean = (20.0 * rest_sim_duration.ln() - 32.0).sqrt();
            let std = (2.0 * rest_sim_duration.ln() - 3.4253).sqrt();
            (s - mean).abs() / std
        };

        let t_d = {
            let std = (2.0 * rest_sim_duration.ln() - 0.8456).sqrt();
            d.abs() / std
        };

        let t_critical = StudentsT::new(0.0, 1.0, rest_sim_duration - 1.0)
            .unwrap()
            .inverse_cdf(1.0 - config.stats.alpha / 2.0);

        if t_s < t_critical && t_d < t_critical {
            min_gap = Some(Gap::Absolute(gap_size));
            break;
        }

        if t_d < t_critical && min_gap.is_none() {
            min_gap = Some(Gap::MeanVariance(gap_size));
        }

        ts_values.push(t_s);
        td_values.push(t_d);
    }

    if min_gap.is_none() {
        log::warn!("Failed to find min gap. Try more total runs");
    }

    log::info!("Min ts = {}", (&ts_values).min());
    log::info!("Min td = {}", (&td_values).min());

    let experiment_results = ExperimentResult {
        runs: total_results,
        warmup_gap: min_gap,
    };

    std::fs::write(
        "results.toml",
        toml::to_string(&experiment_results).unwrap(),
    )
    .unwrap();

    log::info!("Single run is finished");

    Linear::from_data(
        "BusyTables over Time",
        (0..(ts_values.len())).map(|v| v as f32).collect(),
        ts_values.into_iter().map(|v| v as f32).collect(),
    )
    .use_approximation(false)
    .save("stats/logs/TS")
    .unwrap();

    Linear::from_data(
        "BusyTables over Time",
        (0..(td_values.len())).map(|v| v as f32).collect(),
        td_values.into_iter().map(|v| v as f32).collect(),
    )
    .use_approximation(false)
    .save("stats/logs/TD")
    .unwrap();

    Linear::from_data(
        "BusyTables over Time",
        total_logs.iter().map(|(tick, _)| tick as f32).collect(),
        total_logs
            .iter()
            .map(|(_, entry)| entry.average_busy_tables)
            .collect(),
    )
    .use_approximation(false)
    .save("stats/logs/BusyTables")
    .unwrap();

    Linear::from_data(
        "FreeWorkers Over Time",
        total_logs.iter().map(|(tick, _)| tick as f32).collect(),
        total_logs
            .iter()
            .map(|(_, entry)| entry.average_free_workers)
            .collect(),
    )
    .use_approximation(false)
    .save("stats/logs/FreeWorkers")
    .unwrap();

    Linear::from_data(
        "WorkerWaitingTime Over Time",
        total_logs.iter().map(|(tick, _)| tick as f32).collect(),
        total_logs
            .iter()
            .map(|(_, entry)| entry.average_worker_waiting_time)
            .collect(),
    )
    .use_approximation(false)
    .save("stats/logs/WaitingTime")
    .unwrap();

    log::info!("Logs visualization is finished");

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

    log::info!("Single run vizualization is finished");
}
