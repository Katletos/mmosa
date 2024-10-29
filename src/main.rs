use rand::prelude::*;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

enum Event {
    Enter(u32),

    WaitingForWorker(u32, u32, bool),
    // LeavePlace,
    ConsumeFood(u32),
    WorkerWalkingDance(u32),
    WaitingForFood(u32), //random val
}

#[derive(Default)]
struct Results {
    average_worker_waiting_time: f32,
    average_order_time: f32,
    average_busy_tables: f32,
    average_free_workers: f32,
    dispatched_clients: f32,
    not_dispatched_clients: f32,
    immediately_left_clients_count: f32,
}

impl Results {
    pub fn zeros() -> Self {
        Self::default()
    }

    pub fn of(sim: Simulation) -> Self {
        let average_worker_waiting_time =
            sim.average_order_time.iter().sum::<u32>() as f32
                / sim.average_order_time.len() as f32;
        let average_order_time =
            sim.average_worker_waiting_time.iter().sum::<u32>() as f32
                / sim.average_worker_waiting_time.len() as f32;
        let average_busy_tables = sim.average_busy_tables.iter().sum::<u32>()
            as f32
            / sim.average_busy_tables.len() as f32;
        let average_free_workers = sim.average_free_workers.iter().sum::<u32>()
            as f32
            / sim.average_free_workers.len() as f32;

        Self {
            average_worker_waiting_time,
            average_order_time,
            average_busy_tables,
            average_free_workers,
            dispatched_clients: sim.dispatched_clients_count as f32,
            not_dispatched_clients: sim.not_dispatched_clients as f32,
            immediately_left_clients_count: sim.immediately_left_clients_count
                as f32,
        }
    }

    pub fn add_mut(&mut self, other: Self) {
        self.average_worker_waiting_time += other.average_worker_waiting_time;
        self.average_order_time += other.average_order_time;
        self.average_busy_tables += other.average_busy_tables;
        self.average_free_workers += other.average_free_workers;
        self.dispatched_clients += other.dispatched_clients;
        self.not_dispatched_clients += other.not_dispatched_clients;
        self.immediately_left_clients_count +=
            other.immediately_left_clients_count;
    }

    pub fn norm_mut(&mut self, count: usize) {
        self.average_worker_waiting_time /= count as f32;
        self.average_order_time /= count as f32;
        self.average_busy_tables /= count as f32;
        self.average_free_workers /= count as f32;
        self.dispatched_clients /= count as f32;
        self.not_dispatched_clients /= count as f32;
        self.immediately_left_clients_count /= count as f32;
    }
}

impl Display for Results {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
            Average order time {}\n
            Average worker waiting time {}\n
            Average busy tables {}\n
            Average free workers {}\n
            Dispatched clients' count {}\n
            Not dispatched clients' count {}\n
            Immediately left clients' count {}\n
            ",
            self.average_order_time,
            self.average_worker_waiting_time,
            self.average_busy_tables,
            self.average_free_workers,
            self.dispatched_clients,
            self.not_dispatched_clients,
            self.immediately_left_clients_count
        )
    }
}
struct Simulation {
    t_max_time: u32,

    available_tables: u32,
    available_workers: u32,

    events: VecDeque<Event>,

    average_worker_waiting_time: Vec<u32>,
    average_order_time: Vec<u32>,
    average_busy_tables: Vec<u32>,
    average_free_workers: Vec<u32>,
    not_dispatched_clients: usize,
    dispatched_clients_count: usize,
    immediately_left_clients_count: usize,
    config: SimulationConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct SimulationConfig {
    workers: u32,
    tables: u32,
    max_time: u32,
    client_ratio: f64,
}

impl Display for SimulationConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Simulation {
    pub fn with_config(config: SimulationConfig) -> Self {
        Self {
            config,
            t_max_time: config.max_time,

            available_tables: config.tables,
            available_workers: config.workers,

            events: VecDeque::with_capacity(150),
            average_worker_waiting_time: vec![],
            average_order_time: vec![],
            average_busy_tables: vec![],
            average_free_workers: vec![],
            not_dispatched_clients: 0,
            dispatched_clients_count: 0,
            immediately_left_clients_count: 0,
        }
    }

    pub fn run(mut self) -> Results {
        for time in 0..self.t_max_time {
            log::trace!("-----Tick #{time}-----");
            self.generate_new_events(time);
            self.process_tick(time);
        }

        Results::of(self)
    }

    fn process_tick(&mut self, time: u32) {
        let mut new_events = VecDeque::with_capacity(self.events.len());

        while let Some(event) = self.events.pop_front() {
            match event {
                Event::Enter(arrival_time) => {
                    log::trace!("Client enter in {arrival_time}");

                    if self.available_tables > 0 {
                        self.available_tables -= 1;

                        let leave_time = thread_rng().gen_range(5..10) + time;
                        new_events.push_back(Event::WaitingForWorker(
                            time, leave_time, true,
                        ));
                    } else {
                        self.immediately_left_clients_count += 1;
                        log::trace!("Client leave immediately");
                    }
                }

                Event::WaitingForWorker(
                    start_time,
                    leave_time,
                    is_first_time,
                ) => {
                    log::trace!("Client is waiting for worker");
                    if leave_time <= time {
                        self.available_tables += 1;
                        self.average_worker_waiting_time
                            .push(leave_time - start_time);

                        if is_first_time {
                            self.not_dispatched_clients += 1;
                        }

                        log::trace!("Client exit without worker");
                        continue;
                    }

                    if self.available_workers > 0 {
                        log::trace!("Client communicates with worker");

                        self.available_workers -= 1;

                        let free_worker_time =
                            thread_rng().gen_range(1..2) + time;
                        // let free_worker_time = time;
                        new_events.push_back(Event::WorkerWalkingDance(
                            free_worker_time,
                        ));

                        self.average_worker_waiting_time
                            .push(time - start_time);
                    } else {
                        new_events.push_back(Event::WaitingForWorker(
                            time,
                            leave_time,
                            is_first_time,
                        ));
                    }
                }

                Event::WorkerWalkingDance(free_worker_time) => {
                    if time >= free_worker_time {
                        log::trace!("Worker becomes free");
                        self.available_workers += 1;
                        assert!(self.available_workers <= self.config.workers);

                        let producing_time = thread_rng().gen_range(1..2);
                        self.average_order_time.push(producing_time);

                        let finish_food_time = producing_time + time;
                        new_events
                            .push_back(Event::WaitingForFood(finish_food_time));
                    } else {
                        log::trace!("Worker is dancing");
                        new_events.push_front(Event::WorkerWalkingDance(
                            free_worker_time,
                        ));
                    }
                }

                Event::WaitingForFood(finish_food_time) => {
                    if time >= finish_food_time {
                        log::trace!("Client starts consuming");
                        let end_consume_time =
                            thread_rng().gen_range(1..5) + time;
                        new_events
                            .push_back(Event::ConsumeFood(end_consume_time));
                    } else {
                        log::trace!("Client is waiting for food");
                        new_events.push_front(Event::WaitingForFood(
                            finish_food_time,
                        ));
                    }
                }

                Event::ConsumeFood(end_consume_time) => {
                    if time >= end_consume_time {
                        log::trace!("Client consuming is finished");
                        let we_want_eat_more = thread_rng().gen_bool(0.2);
                        if we_want_eat_more {
                            log::trace!("Client wants mo-o-ore!!!");
                            let leave_time =
                                thread_rng().gen_range(1..3) + time;
                            new_events.push_back(Event::WaitingForWorker(
                                time, leave_time, false,
                            ));
                        } else {
                            self.dispatched_clients_count += 1;
                            self.available_tables += 1;
                            log::trace!("Client exit after consumption");
                            // new_events.push_back(Event::LeavePlace);
                        }
                    } else {
                        log::trace!("Client is consuming food");
                        new_events
                            .push_back(Event::ConsumeFood(end_consume_time));
                    }
                } // Event::LeavePlace => {
                  //     log::trace!("Client is leaving table");
                  //     self.available_tables += 1;
                  // }
            }
        }

        self.average_busy_tables
            .push(self.config.tables - self.available_tables);
        self.average_free_workers.push(self.available_workers);

        new_events
            .into_iter()
            .for_each(|e| self.events.push_back(e));
    }

    fn generate_new_events(&mut self, tick: u32) {
        let is_client_arrived = thread_rng().gen_bool(self.config.client_ratio);

        if is_client_arrived {
            self.events.push_back(Event::Enter(tick));
        }
    }
}

fn main() {
    env_logger::builder().init();

    let mut results = Results::zeros();
    let iter_count = std::env::var("ITER_COUNT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20_000);

    let config = SimulationConfig {
        workers: 1,
        tables: 7,
        max_time: 1000,
        client_ratio: 0.65,
    };

    for _ in 0..iter_count {
        let sim = Simulation::with_config(config);
        results.add_mut(sim.run());
    }

    results.norm_mut(iter_count);

    println!("{results}");
    println!("Configuration: {config}");
}
