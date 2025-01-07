use std::collections::{HashMap, VecDeque};

pub struct IntervalStats {
    pub min: f32,
    pub max: f32,
    pub sum: f32,
    pub sum_squares: f32,
    pub count: usize,
    pub last: f32,
}

impl Default for IntervalStats {
    fn default() -> Self {
        IntervalStats {
            min: f32::MAX,
            max: f32::MIN,
            sum: 0.0,
            sum_squares: 0.0,
            count: 0,
            last: 0.0,
        }
    }
}

impl IntervalStats {
    fn add(&mut self, value: f32) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum += value;
        self.sum_squares += value * value;
        self.count += 1;
    }
}

pub struct SymbolDataStore {
    capacity: usize,
    prices: VecDeque<f32>,
    interval_stats: HashMap<usize, IntervalStats>,
}

impl SymbolDataStore {
    pub fn new(capacity: usize) -> Self {
        let mut store = SymbolDataStore {
            capacity,
            prices: VecDeque::with_capacity(capacity),
            interval_stats: HashMap::new(),
        };

        store.init_stats();
        store
    }

    fn init_stats(&mut self) {
        self.interval_stats.clear();
        let last = self.prices.front().unwrap_or(&0.0);
        for i in 1..=self.prices.capacity().ilog10() {
            self.interval_stats.insert(
                10_usize.pow(i),
                IntervalStats {
                    last: *last,
                    ..IntervalStats::default()
                },
            );
        }
    }

    pub fn add_batch(&mut self, prices: &[f32]) {
        prices.iter().for_each(|price| {
            if self.prices.len() == self.capacity {
                self.prices.pop_back();
            }
            self.prices.push_front(*price);
        });
        self.init_stats();
        self.calculate_intervals();
    }

    pub fn get_stats(&self, k: u32) -> Option<&IntervalStats> {
        self.interval_stats.get(&10_usize.pow(k))
    }

    // calculte stast for intervals window of size 10 pow of (1-8)
    fn calculate_intervals(&mut self) {
        self.prices.iter().enumerate().for_each(|(i, price)| {
            self.interval_stats
                .iter_mut()
                .for_each(|(interval, stats)| {
                    if i / interval == 0 {
                        stats.add(*price);
                    }
                })
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_batch() {
        let k = 8usize;
        let mut store = SymbolDataStore::new(10_usize.pow(k as u32));

        // adding 10000 prices
        store.add_batch(&vec![1.0; 10000]);
        assert_eq!(store.prices.len(), 10000);
        assert_eq!(store.interval_stats.len(), k);
        store.interval_stats.iter().for_each(|(interval, stats)| {
            let count = 10000usize.min(*interval);
            assert_eq!(stats.count, count);
            assert_eq!(stats.min, 1.0);
            assert_eq!(stats.max, 1.0);
            assert_eq!(stats.sum, count as f32);
            assert_eq!(stats.sum_squares, count as f32);
            assert_eq!(stats.last, 1.0);
        });

        // adding another 10000 prices
        store.add_batch(&vec![2.0; 10000]);
        assert_eq!(store.prices.len(), 20000);
        assert_eq!(store.interval_stats.len(), k);

        let stats = store.interval_stats.get(&10000usize).unwrap();
        assert_eq!(stats.count, 10000);
        assert_eq!(stats.min, 2.0);
        assert_eq!(stats.max, 2.0);
        assert_eq!(stats.sum, 20000.0);
        assert_eq!(stats.sum_squares, 40000.0);
        assert_eq!(stats.last, 2.0);

        let stats = store.interval_stats.get(&1000usize).unwrap();
        assert_eq!(stats.count, 1000);
        assert_eq!(stats.min, 2.0);
        assert_eq!(stats.max, 2.0);
        assert_eq!(stats.sum, 2000.0);
        assert_eq!(stats.sum_squares, 4000.0);
        assert_eq!(stats.last, 2.0);

        let stats = store.interval_stats.get(&100_000).unwrap();
        assert_eq!(stats.count, 20000);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 2.0);
        assert_eq!(stats.sum, 30000.0);
        assert_eq!(stats.sum_squares, 50000.0);
        assert_eq!(stats.last, 2.0);
    }

    #[test]
    fn test_capacity_not_growing() {
        let capacity = 10_usize.pow(4);
        let mut store = SymbolDataStore::new(capacity);

        let data = vec![1.0; capacity];

        store.add_batch(&data);
        store.add_batch(&data);

        assert_eq!(store.prices.capacity(), capacity);
        assert_eq!(store.prices.len(), capacity);
    }
}
