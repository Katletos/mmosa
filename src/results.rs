use std::fmt::{Display, Formatter};

#[derive(Default, Clone)]
pub struct Results {
    pub average_worker_waiting_time: f32,
    pub average_order_time: f32,
    pub average_busy_tables: f32,
    pub average_free_workers: f32,
    pub average_consumption_time: f32,
    pub dispatched_clients: f32,
    pub not_dispatched_clients: f32,
    pub immediately_left_clients_count: f32,
}

impl Results {
    pub fn zeros() -> Self {
        Self::default()
    }

    pub fn add_mut(&mut self, other: Self) {
        self.average_worker_waiting_time += other.average_worker_waiting_time;
        self.average_consumption_time += other.average_consumption_time;
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
        self.average_consumption_time /= count as f32;
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
