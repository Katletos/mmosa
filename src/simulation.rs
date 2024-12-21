use std::collections::VecDeque;

use rand::prelude::*;

use crate::{Event, Results, SimulationConfig};

pub struct Simulation {
    t_max_time: u32,

    available_tables: u32,
    available_workers: u32,

    events: VecDeque<Event>,

    average_worker_waiting_time: Vec<u32>,
    average_order_time: Vec<u32>,
    average_consumption_time: Vec<u32>,
    average_busy_tables: Vec<u32>,
    average_free_workers: Vec<u32>,
    not_dispatched_clients: usize,
    dispatched_clients_count: usize,
    immediately_left_clients_count: usize,
    // average_time_in: Vec<u32>,
    config: SimulationConfig,
}

impl Simulation {
    pub fn with_config(config: SimulationConfig) -> Self {
        Self {
            t_max_time: config.max_time,

            available_tables: config.tables,
            available_workers: config.workers,

            config,
            events: VecDeque::with_capacity(150),
            average_worker_waiting_time: vec![],
            average_order_time: vec![],
            average_busy_tables: vec![],
            average_free_workers: vec![],
            average_consumption_time: vec![],
            not_dispatched_clients: 0,
            dispatched_clients_count: 0,
            immediately_left_clients_count: 0,
        }
    }

    // pub fn set_callback(&mut self, )

    pub fn run(mut self) -> Results {
        for time in 0..self.t_max_time {
            log::trace!("-----Tick #{time}-----");
            self.generate_new_events(time);
            self.process_tick(time);
        }

        result_of(self)
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

                        let free_worker_time = thread_rng()
                            .gen_range(self.config.dancing_time.clone())
                            + time;
                        // let free_worker_time = time;
                        new_events.push_back(Event::WorkerWalkingDance(
                            free_worker_time,
                        ));

                        self.average_worker_waiting_time
                            .push(time - start_time);
                    } else {
                        new_events.push_back(Event::WaitingForWorker(
                            start_time,
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
                        //todo: add correlation producing time on workload

                        let producing_time = thread_rng()
                            .gen_range(self.config.production_time.clone());

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

                        let consumption_time = thread_rng()
                            .gen_range(self.config.consumption_time.clone());

                        self.average_consumption_time.push(consumption_time);

                        let end_consume_time = consumption_time + time;

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
                }
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

fn result_of(sim: Simulation) -> Results {
    let average_worker_waiting_time =
        sim.average_worker_waiting_time.iter().sum::<u32>() as f32
            / sim.average_worker_waiting_time.len() as f32;

    let average_order_time = sim.average_order_time.iter().sum::<u32>() as f32
        / sim.average_order_time.len() as f32;

    let average_busy_tables = sim.average_busy_tables.iter().sum::<u32>()
        as f32
        / sim.average_busy_tables.len() as f32;

    let average_free_workers = sim.average_free_workers.iter().sum::<u32>()
        as f32
        / sim.average_free_workers.len() as f32;

    let average_consumption_time =
        sim.average_consumption_time.iter().sum::<u32>() as f32
            / sim.average_consumption_time.len() as f32;

    Results {
        average_consumption_time,
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
