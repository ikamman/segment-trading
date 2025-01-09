use std::collections::VecDeque;

#[derive(Debug)]
pub struct IntervalStatDeque {
    deque: VecDeque<(usize, f32)>,
    window_size: usize,
    stat_type: StatType,
    pos: usize,
}

#[derive(Debug)]
pub enum StatType {
    Min,
    Max,
}

impl StatType {
    fn eval(&self, last_value: f32, new_value: f32) -> bool {
        match self {
            Self::Min => last_value > new_value,
            Self::Max => last_value < new_value,
        }
    }
}

impl IntervalStatDeque {
    pub fn new(window_size: usize, stat_type: StatType) -> Self {
        Self {
            deque: VecDeque::new(),
            window_size,
            stat_type,
            pos: 0,
        }
    }

    pub fn push(&mut self, val: f32) {
        // Remove elements that are outside the window
        while let Some(&(pos, _)) = self.deque.front() {
            if !self.is_in_window(pos) {
                self.deque.pop_front();
            } else {
                break;
            }
        }

        // Remove elements that are larger than the current value
        while let Some(&(_, back_val)) = self.deque.back() {
            if self.stat_type.eval(back_val, val) {
                self.deque.pop_back();
            } else {
                break;
            }
        }
        self.deque.push_back((self.pos, val));

        self.pos = self.pos.wrapping_add(1);
    }

    fn is_in_window(&self, pos: usize) -> bool {
        let window_start = self.pos.wrapping_sub(self.window_size);
        if window_start <= self.pos {
            pos > window_start && pos <= self.pos
        }
        // If window wraps around
        else {
            pos > window_start || pos <= self.pos
        }
    }

    pub fn stat(&self) -> f32 {
        self.deque.front().map(|v| v.1).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_max() {
        let k = 3;
        let mut swm = IntervalStatDeque::new(k, StatType::Max);

        let inputs = vec![5.0, 1.0, 3.0, 2.0, 6.0, 0.0, 2.0, 1.0, 1.0];

        let mut results = Vec::new();
        for &val in &inputs {
            swm.push(val);
            results.push(swm.stat());
        }

        let expected = vec![5.0, 5.0, 5.0, 3.0, 6.0, 6.0, 6.0, 2.0, 2.0];
        assert_eq!(results, expected, "MAXs do not match expected values");
    }

    #[test]
    fn test_interval_min() {
        let k = 3;
        let mut swm = IntervalStatDeque::new(k, StatType::Min);

        let inputs = vec![5.0, 1.0, 3.0, 2.0, 6.0, 0.0, 2.0, 1.0, 1.0];

        let mut results = Vec::new();
        for &val in &inputs {
            swm.push(val);
            results.push(swm.stat());
        }

        let expected = vec![5.0, 1.0, 1.0, 1.0, 2.0, 0.0, 0.0, 0.0, 1.0];
        assert_eq!(results, expected, "MINs do not match expected values");
    }
}
