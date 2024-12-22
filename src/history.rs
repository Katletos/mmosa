use std::collections::HashMap;

use crate::{Results, SimulationTick};

pub struct Log {
    pub entries: HashMap<SimulationTick, Results>,
}

impl Log {
    pub fn empty() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn append(&mut self, tick: SimulationTick, entry: Results) {
        let _ = self.entries.insert(tick, entry);
    }

    ///ordered by simulation time
    pub fn iter(&self) -> impl Iterator<Item = (SimulationTick, &Results)> {
        let mut ticks = self.entries.keys().copied().collect::<Vec<_>>();
        ticks.sort_unstable();

        ticks.into_iter().map(|tick| {
            let entry = self.entries.get(&tick).unwrap();
            (tick, entry)
        })
    }

    pub fn add_mut(&mut self, other: Self) {
        other.entries.into_iter().for_each(|(tick, entry)| {
            self.entries
                .entry(tick)
                .and_modify(|e| e.add_mut(entry.clone()))
                .or_insert(entry);
        });
    }

    pub fn norm_mut(&mut self, count: usize) {
        self.entries
            .values_mut()
            .for_each(|entry| entry.norm_mut(count));
    }
}
