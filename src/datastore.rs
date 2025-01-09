use std::collections::{HashMap, VecDeque};

use crate::interval_stat_deque::{IntervalStatDeque, StatType};

#[derive(Clone, Debug)]
pub struct IntervalStats {
    pub min: f32,
    pub max: f32,
    pub sum: f32,
    pub sum_squares: f32,
    pub count: usize,
    pub last: f32,
}

pub struct IntervalStatsStore {
    pub data: VecDeque<f32>,
    pub interval: usize,
    pub deque_min: IntervalStatDeque,
    pub deque_max: IntervalStatDeque,
    pub sum: f32,
    pub sum_squares: f32,
    pub last: f32,
}

impl IntervalStatsStore {
    fn new(interval: usize) -> Self {
        IntervalStatsStore {
            data: VecDeque::new(),
            interval,
            deque_min: IntervalStatDeque::new(interval, StatType::Min),
            deque_max: IntervalStatDeque::new(interval, StatType::Max),
            sum: 0.0,
            sum_squares: 0.0,
            last: 0.0,
        }
    }

    fn add(&mut self, value: f32) {
        // adding new data
        self.data.push_back(value);
        // basic stats
        self.sum += value;
        self.sum_squares += value * value;
        self.last = value;

        // handling min and max
        self.deque_min.push(value);
        self.deque_max.push(value);

        // handling to big window
        if self.data.len() > self.interval {
            if let Some(to_remove) = self.data.pop_front() {
                self.sum -= to_remove;
                self.sum_squares -= to_remove * to_remove;
            }
        }

        println!("Count {}", self.data.len());
    }

    pub fn get_stats(&self) -> IntervalStats {
        IntervalStats {
            min: self.deque_min.stat(),
            max: self.deque_max.stat(),
            sum: self.sum,
            sum_squares: self.sum_squares,
            count: self.data.len(),
            last: self.last,
        }
    }
}

pub struct SymbolDataStore {
    intervals: HashMap<usize, IntervalStatsStore>,
}

impl SymbolDataStore {
    pub fn new(num_of_intervals: usize) -> Self {
        let mut intervals = HashMap::new();
        (1..=num_of_intervals).for_each(|i| {
            intervals.insert(i, IntervalStatsStore::new(10_usize.pow(i as u32)));
        });

        SymbolDataStore { intervals }
    }

    pub fn add_batch(&mut self, prices: &[f32]) {
        prices.iter().for_each(|price| {
            self.intervals.iter_mut().for_each(|(_, stats)| {
                stats.add(*price);
            });
        });
    }

    pub fn get_stats(&self, k: usize) -> Option<IntervalStats> {
        self.intervals.get(&k).map(|stats| stats.get_stats())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_batch() {
        let k = 8_usize;
        let mut store = SymbolDataStore::new(k);

        // adding 10000 prices
        store.add_batch(&vec![1.0; 10000]);
        assert_eq!(store.intervals.get(&4).unwrap().data.len(), 10000);
        store.intervals.iter().for_each(|(interval, stats)| {
            let count = 10000.min(10_usize.pow(*interval as u32));
            println!("Interval {} Count {}", interval, count);
            let stats = stats.get_stats();
            assert_eq!(stats.count, count);
            assert_eq!(stats.min, 1.0);
            assert_eq!(stats.max, 1.0);
            assert_eq!(stats.sum, count as f32);
            assert_eq!(stats.sum_squares, count as f32);
            assert_eq!(stats.last, 1.0);
        });

        // adding another 10000 prices
        store.add_batch(&vec![2.0; 10000]);

        let stats = store.get_stats(4).unwrap();
        assert_eq!(stats.count, 10000);
        assert_eq!(stats.min, 2.0);
        assert_eq!(stats.max, 2.0);
        assert_eq!(stats.sum, 20000.0);
        assert_eq!(stats.sum_squares, 40000.0);
        assert_eq!(stats.last, 2.0);

        let stats = store.get_stats(3).unwrap();
        assert_eq!(stats.count, 1000);
        assert_eq!(stats.min, 2.0);
        assert_eq!(stats.max, 2.0);
        assert_eq!(stats.sum, 2000.0);
        assert_eq!(stats.sum_squares, 4000.0);
        assert_eq!(stats.last, 2.0);

        let stats = store.get_stats(5).unwrap();
        assert_eq!(stats.count, 20000);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 2.0);
        assert_eq!(stats.sum, 30000.0);
        assert_eq!(stats.sum_squares, 50000.0);
        assert_eq!(stats.last, 2.0);
    }

    #[test]
    fn test_capacity_not_growing() {
        let num_of_intervals = 4;
        let mut store = SymbolDataStore::new(num_of_intervals);

        let data = vec![1.0; 10_usize.pow(num_of_intervals as u32)];

        for _ in 0..1000 {
            store.add_batch(&data);
        }
        store.intervals.iter().for_each(|(_, stats)| {
            assert_eq!(stats.data.len(), stats.interval);
        });
    }
}
