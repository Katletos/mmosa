use rayon::iter::{IntoParallelIterator, ParallelIterator};
use statrs::{
    distribution::{ContinuousCDF, StudentsT},
    statistics::Statistics,
};

use crate::{
    chart::Linear,
    statistic::{f_test, t_test, FisherTest, StudentTest},
    EstimationConfig, Log, Results, Simulation,
};

#[derive(serde::Serialize)]
pub struct Test {
    pub name: &'static str,
    pub t_test: StudentTest,
    pub f_test: FisherTest,
}

#[derive(serde::Serialize)]
pub struct ExperimentResult {
    pub runs: Results,
    pub tests: Vec<Test>,
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
    pub min_total: usize,
    /// use continous experiment (warmed up state)
    pub continous: bool,
    pub gap_size: usize,
    pub parameter: BaseParameter,
}

pub fn run(config: EstimationConfig, base_path: &str) -> Results {
    let mut total_results = Results::zeros();
    let mut total_logs = Log::empty();
    let mut results = Vec::<Results>::new();

    if config.experiment.continous {
        let mut sim = Simulation::with_config(config.simulation.clone());
        for _ in 0..config.experiment.total {
            sim.reset_metrics();
            let (run_result, run_log) = sim.run();
            total_results.add_mut(run_result.clone());
            total_logs.add_mut(run_log);
            results.push(run_result);
        }
    } else {
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
    }

    total_results.norm_mut(config.experiment.total);
    total_logs.norm_mut(config.experiment.total);

    assert!(config.experiment.total > 2, "At least 3 run must be set");

    let logs_entries = total_logs
        .iter()
        .map(|(_tick, entry)| entry.clone())
        .collect::<Vec<_>>();

    let sim_duration = config.simulation.max_time as usize;
    assert_eq!(sim_duration, logs_entries.len());

    Linear::from_data(
        "BusyTables over Time",
        total_logs.iter().map(|(tick, _)| tick as f32).collect(),
        total_logs
            .iter()
            .map(|(_, entry)| entry.average_busy_tables)
            .collect(),
    )
    .save(&format!("{base_path}/BusyTables"))
    .unwrap();

    Linear::from_data(
        "FreeWorkers Over Time",
        total_logs.iter().map(|(tick, _)| tick as f32).collect(),
        total_logs
            .iter()
            .map(|(_, entry)| entry.average_free_workers)
            .collect(),
    )
    .save(&format!("{base_path}/FreeWorkers"))
    .unwrap();

    Linear::from_data(
        "WaitingTime Over Time",
        total_logs.iter().map(|(tick, _)| tick as f32).collect(),
        total_logs
            .iter()
            .map(|(_, entry)| entry.average_worker_waiting_time)
            .collect(),
    )
    .save(&format!("{base_path}/WaitingTime"))
    .unwrap();

    Linear::from_data(
        "Dispatched Clients",
        total_logs.iter().map(|(tick, _)| tick as f32).collect(),
        total_logs
            .iter()
            .map(|(_, entry)| entry.dispatched_clients)
            .collect(),
    )
    .save(&format!("{base_path}/DispatchedClients"))
    .unwrap();

    let long_data = results
        .iter()
        .skip(config.experiment.gap_size)
        .cloned()
        .collect::<Vec<_>>();

    let short_data = results
        .iter()
        .skip(config.experiment.gap_size)
        .take(config.experiment.min_total)
        .cloned()
        .collect::<Vec<_>>();

    let experiment_results = ExperimentResult {
        runs: total_results.clone(),
        tests: vec![
            Test {
                name: "busy_tables",
                t_test: t_test(
                    &long_data
                        .iter()
                        .map(|r| r.average_busy_tables as f64)
                        .collect::<Vec<_>>(),
                    &short_data
                        .iter()
                        .map(|r| r.average_busy_tables as f64)
                        .collect::<Vec<_>>(),
                ),
                f_test: f_test(
                    &long_data
                        .iter()
                        .map(|r| r.average_busy_tables as f64)
                        .collect::<Vec<_>>(),
                    &short_data
                        .iter()
                        .map(|r| r.average_busy_tables as f64)
                        .collect::<Vec<_>>(),
                ),
            },
            Test {
                name: "free_workers",
                t_test: t_test(
                    &long_data
                        .iter()
                        .map(|r| r.average_free_workers as f64)
                        .collect::<Vec<_>>(),
                    &short_data
                        .iter()
                        .map(|r| r.average_free_workers as f64)
                        .collect::<Vec<_>>(),
                ),
                f_test: f_test(
                    &long_data
                        .iter()
                        .map(|r| r.average_free_workers as f64)
                        .collect::<Vec<_>>(),
                    &short_data
                        .iter()
                        .map(|r| r.average_free_workers as f64)
                        .collect::<Vec<_>>(),
                ),
            },
            Test {
                name: "waiting_time",
                t_test: t_test(
                    &long_data
                        .iter()
                        .map(|r| r.average_worker_waiting_time as f64)
                        .collect::<Vec<_>>(),
                    &short_data
                        .iter()
                        .map(|r| r.average_worker_waiting_time as f64)
                        .collect::<Vec<_>>(),
                ),
                f_test: f_test(
                    &long_data
                        .iter()
                        .map(|r| r.average_worker_waiting_time as f64)
                        .collect::<Vec<_>>(),
                    &short_data
                        .iter()
                        .map(|r| r.average_worker_waiting_time as f64)
                        .collect::<Vec<_>>(),
                ),
            },
        ],
    };

    std::fs::write(
        format!("{base_path}/results.toml"),
        toml::to_string(&experiment_results).unwrap(),
    )
    .unwrap();

    total_results
}
