use std::ops::Range;

use crate::{
    chart::Linear, EstimationConfig, Results, Simulation, SimulationConfig,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioParameter {
    Workers,
    Tables,
    Clients,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScenarioConfig {
    pub parameter: ScenarioParameter,
    pub values: Range<u32>,
    pub step: u32,
}

pub fn run(config: EstimationConfig) {
    let scenario = config.scenario.clone().unwrap();
    let build_config = config_builder(&config);

    let mut scenario_results = vec![];
    for v in scenario.values.clone().step_by(scenario.step as usize) {
        let simulation_config = build_config(v);

        let mut total_results = Results::zeros();
        let mut results = Vec::<Results>::new();
        for _ in 0..config.total {
            let sim = Simulation::with_config(simulation_config.clone());
            let (run_result, _run_log) = sim.run();

            total_results.add_mut(run_result.clone());
            results.push(run_result);
        }

        total_results.norm_mut(config.total);

        scenario_results.push(total_results);
    }

    let parameters = scenario
        .values
        .step_by(scenario.step as usize)
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
    .save("stats/scenario/DuspatchedClients")
    .unwrap();

    Linear::from_data(
        "Not Dispatched clients",
        parameters.clone(),
        scenario_results
            .iter()
            .map(|r| r.not_dispatched_clients)
            .collect(),
    )
    .save("stats/scenario/NotDuspatchedClients")
    .unwrap();

    Linear::from_data(
        "Busy Tables",
        parameters.clone(),
        scenario_results
            .iter()
            .map(|r| r.average_busy_tables)
            .collect(),
    )
    .save("stats/scenario/BusyTables")
    .unwrap();

    Linear::from_data(
        "Free Worker",
        parameters.clone(),
        scenario_results
            .iter()
            .map(|r| r.average_free_workers)
            .collect(),
    )
    .save("stats/scenario/FreeWorker")
    .unwrap();
}

fn config_builder(
    config: &EstimationConfig,
) -> impl Fn(u32) -> SimulationConfig {
    let scenario = config.scenario.clone().unwrap();
    let kind = scenario.parameter.clone();
    let mut base_config = config.simulation.clone();
    base_config.use_logs = false;

    move |v| {
        let mut config = base_config.clone();

        match kind {
            ScenarioParameter::Workers => {
                config.workers = v;
            }
            ScenarioParameter::Tables => {
                config.tables = v;
            }
            ScenarioParameter::Clients => {
                config.client_ratio = v as f64 / 100.0
            }
        }

        config
    }
}
