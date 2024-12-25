use std::{fmt::Display, ops::Range};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    chart::{HyperPlane, Linear},
    EstimationConfig, Results, Simulation, SimulationConfig,
};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterKind {
    Workers,
    Tables,
    Clients,
    Dancing,
    Production,
}

impl Display for ParameterKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScenarioParameter {
    pub kind: ParameterKind,
    pub values: Range<u32>,
    pub step: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScenarioConfig {
    pub parameters: Vec<ScenarioParameter>,
}

pub fn run(config: EstimationConfig, task_name: &str) {
    let scenario = config.scenario.clone().unwrap();

    assert!(scenario.parameters.len() <= 2);
    if scenario.parameters.len() == 1 {
        let mut scenario_results = vec![];
        let parameter = scenario.parameters[0].clone();
        let build_config =
            config_builder(config.simulation.clone(), &parameter);

        for v in parameter.values.clone().step_by(parameter.step as usize) {
            let simulation_config = build_config(v);

            let mut total_results = Results::zeros();
            let mut results = Vec::<Results>::new();
            let tmp = (0..config.experiment.total)
                .into_par_iter()
                .map(|_| {
                    let mut sim =
                        Simulation::with_config(simulation_config.clone());
                    sim.run()
                })
                .collect::<Vec<_>>();

            tmp.into_iter().for_each(|(run_result, _log)| {
                total_results.add_mut(run_result.clone());
                results.push(run_result);
            });

            total_results.norm_mut(config.experiment.total);

            scenario_results.push(total_results);
        }

        let parameters = parameter
            .values
            .step_by(parameter.step as usize)
            .map(|v| v as f32)
            .collect::<Vec<f32>>();

        Linear::from_data(
            "Dispatched clients",
            parameters.clone(),
            scenario_results
                .iter()
                .map(|r| r.dispatched_clients)
                .collect(),
        )
        .use_approximation(true)
        .save(&format!("stats/{task_name}/DispatchedClients"))
        .unwrap();

        Linear::from_data(
            "Not Dispatched clients",
            parameters.clone(),
            scenario_results
                .iter()
                .map(|r| r.not_dispatched_clients)
                .collect(),
        )
        .use_approximation(true)
        .save(&format!("stats/{task_name}/NotDispatchedClients"))
        .unwrap();

        Linear::from_data(
            "Busy Tables",
            parameters.clone(),
            scenario_results
                .iter()
                .map(|r| r.average_busy_tables)
                .collect(),
        )
        .use_approximation(true)
        .save(&format!("stats/{task_name}/BusyTables"))
        .unwrap();

        Linear::from_data(
            "Free Worker",
            parameters.clone(),
            scenario_results
                .iter()
                .map(|r| r.average_free_workers)
                .collect(),
        )
        .use_approximation(true)
        .save(&format!("stats/{task_name}/FreeWorkers"))
        .unwrap();

        Linear::from_data(
            "Waiting Time",
            parameters.clone(),
            scenario_results
                .iter()
                .map(|r| r.average_worker_waiting_time)
                .collect(),
        )
        .use_approximation(true)
        .save(&format!("stats/{task_name}/WaitingTime"))
        .unwrap();
    } else {
        assert!(scenario.parameters.len() == 2);

        let x_param = scenario.parameters[0].clone();
        let z_param = scenario.parameters[1].clone();

        let mut scenario_results = Vec::<Results>::new();

        let build_config_with_x =
            config_builder(config.simulation.clone(), &x_param);

        for x in x_param.values.clone().step_by(x_param.step as usize) {
            let base_config = build_config_with_x(x);
            let build_config_with_z = config_builder(base_config, &z_param);

            for z in z_param.values.clone().step_by(z_param.step as usize) {
                let simulation_config = build_config_with_z(z);
                let mut avg_sim_results = Results::zeros();

                let tmp = (0..config.experiment.total)
                    .into_par_iter()
                    .map(|_| {
                        let mut sim =
                            Simulation::with_config(simulation_config.clone());
                        sim.run()
                    })
                    .collect::<Vec<_>>();

                for (run_result, _log) in tmp.into_iter() {
                    avg_sim_results.add_mut(run_result.clone());
                }

                avg_sim_results.norm_mut(config.experiment.total);

                scenario_results.push(avg_sim_results);
            }
        }

        let x_values = x_param
            .values
            .clone()
            .step_by(x_param.step as usize)
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        let z_values = z_param
            .values
            .clone()
            .step_by(z_param.step as usize)
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        HyperPlane::from_data(
            x_values.clone(),
            z_values.clone(),
            scenario_results
                .iter()
                .map(|r| r.average_worker_waiting_time as f64)
                .collect(),
            &format!("WaitingTime over X={} Z={}", x_param.kind, z_param.kind),
        )
        .save("stats/multi/WaitingTime")
        .unwrap();

        HyperPlane::from_data(
            x_values.clone(),
            z_values.clone(),
            scenario_results
                .iter()
                .map(|r| r.average_free_workers as f64)
                .collect(),
            &format!("FreeWorkers over X={} Z={}", x_param.kind, z_param.kind),
        )
        .save("stats/multi/FreeWorkers")
        .unwrap();
    }
}

fn config_builder(
    mut base_config: SimulationConfig,
    parameter: &ScenarioParameter,
) -> impl Fn(u32) -> SimulationConfig {
    let kind = parameter.kind;
    base_config.use_logs = false;

    move |v| {
        let mut config = base_config.clone();

        match kind {
            ParameterKind::Workers => {
                config.workers = v;
            }
            ParameterKind::Tables => {
                config.tables = v;
            }
            ParameterKind::Clients => config.client_ratio = v as f64 / 100.0,
            ParameterKind::Dancing => {
                config.dancing_time = 1..v;
            }
            ParameterKind::Production => {
                config.production_time = 1..v;
            }
        }

        config
    }
}
