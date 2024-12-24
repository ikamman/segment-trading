use core::f64;

// NodeData holding row ingradients for statistics calculation
#[derive(Clone, Copy, Debug)]
pub struct NodeData {
    pub min: f64,
    pub max: f64,
    pub sum: f64,
    pub sum_squares: f64,
    pub count: i64,
    pub last: f64,
}

impl NodeData {
    fn new(value: f64) -> Self {
        NodeData {
            min: value,
            max: value,
            sum: value,
            sum_squares: value * value,
            count: 1,
            last: value,
        }
    }

    fn merge(left: &NodeData, right: &NodeData) -> Self {
        if left.count == 0 {
            return *right;
        }
        if right.count == 0 {
            return *left;
        }
        NodeData {
            min: left.min.min(right.min),
            max: left.max.max(right.max),
            sum: left.sum + right.sum,
            sum_squares: left.sum_squares + right.sum_squares,
            count: left.count + right.count,
            last: right.last,
        }
    }

    fn zero() -> Self {
        NodeData {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum: 0.0,
            sum_squares: 0.0,
            count: 0,
            last: 0.0,
        }
    }
}

// Core strcture that holds the segment tree
pub struct SegmentTree {
    pub tree: Vec<NodeData>,
    pub size: usize,
    pub current_position: usize,
}

impl SegmentTree {
    pub fn new() -> Self {
        let initial_size = 1024;
        let tree_size = initial_size * 2;
        SegmentTree {
            tree: vec![NodeData::zero(); tree_size],
            size: initial_size,
            current_position: 0,
        }
    }

    // We need to ensure that the tree has enough capacity to store the values
    // The tree need to be resized if the current_position is greater than the size
    // This rewrite the tree with a new size and copy the leaf nodes
    fn ensure_capacity(&mut self, needed_size: usize) {
        if needed_size <= self.size {
            return;
        }

        let mut new_size = self.size;
        while new_size < needed_size {
            new_size *= 2;
        }

        // Save leaf nodes
        let mut leaves = Vec::with_capacity(self.current_position);
        for i in 0..self.current_position {
            leaves.push(self.tree[self.size + i]);
        }

        // Resize the tree
        self.tree = vec![NodeData::zero(); new_size * 2];
        self.size = new_size;

        // Restore leaf nodes
        for (i, leaf) in leaves.into_iter().enumerate() {
            self.tree[self.size + i] = leaf;
        }

        // Rebuild internal nodes from leaves up
        for i in (1..self.size).rev() {
            self.tree[i] = NodeData::merge(&self.tree[i * 2], &self.tree[i * 2 + 1]);
        }
    }

    fn update(&mut self, pos: usize, value: f64) {
        let mut node = pos + self.size;
        self.tree[node] = NodeData::new(value);

        while node > 1 {
            node /= 2;
            self.tree[node] = NodeData::merge(&self.tree[node * 2], &self.tree[node * 2 + 1]);
        }
    }

    pub fn query_range(&self, start: usize, end: usize) -> NodeData {
        self.query_internal(1, 0, self.size, start, end)
    }

    fn query_internal(
        &self,
        node: usize,
        left: usize,
        right: usize,
        start: usize,
        end: usize,
    ) -> NodeData {
        if end <= left || right <= start {
            return NodeData::zero();
        }

        if start <= left && right <= end {
            return self.tree[node];
        }

        let mid = (left + right) / 2;
        let left_result = self.query_internal(node * 2, left, mid, start, end);
        let right_result = self.query_internal(node * 2 + 1, mid, right, start, end);

        NodeData::merge(&left_result, &right_result)
    }

    pub fn add_batch(&mut self, values: &[f64]) {
        self.ensure_capacity(self.current_position + values.len());

        for &value in values {
            self.update(self.current_position, value);
            self.current_position += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    fn assert_float_eq(a: f64, b: f64) {
        if a.is_infinite() && b.is_infinite() {
            assert_eq!(a.is_sign_positive(), b.is_sign_positive());
        } else {
            assert!((a - b).abs() < EPSILON, "Expected {} but got {}", b, a);
        }
    }

    #[test]
    fn test_multiple_resize() {
        let mut tree = SegmentTree::new();
        // Add values that will cause multiple resizes
        for i in 0..5 {
            let values = vec![1.0; 1000];
            tree.add_batch(&values);
            let result = tree.query_range(0, (i + 1) * 1000);
            assert_float_eq(result.sum, ((i + 1) * 1000) as f64);
            assert_eq!(result.count as usize, (i + 1) * 1000);
        }
    }

    #[test]
    fn test_mixed_values_with_resize() {
        let mut tree = SegmentTree::new();
        // First batch
        let values: Vec<f64> = (0..1000).map(|x| x as f64).collect();
        tree.add_batch(&values);

        // Second batch that will cause resize
        let values: Vec<f64> = (1000..2000).map(|x| x as f64).collect();
        tree.add_batch(&values);

        let result = tree.query_range(0, 2000);
        assert_float_eq(result.min, 0.0);
        assert_float_eq(result.max, 1999.0);
        assert_float_eq(result.sum, 1999.0 * 1000.0); // sum of arithmetic sequence
        assert_eq!(result.count, 2000);
    }

    #[test]
    fn test_sequential_updates() {
        let mut tree = SegmentTree::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        tree.add_batch(&values);

        // Test full range
        let result = tree.query_range(0, 5);
        assert_float_eq(result.min, 1.0);
        assert_float_eq(result.max, 5.0);
        assert_float_eq(result.sum, 15.0);
        assert_float_eq(result.sum_squares, 55.0);
        assert_eq!(result.count, 5);
        assert_float_eq(result.last, 5.0);

        // Test partial ranges
        let result = tree.query_range(1, 4);
        assert_float_eq(result.min, 2.0);
        assert_float_eq(result.max, 4.0);
        assert_float_eq(result.sum, 9.0);
        assert_eq!(result.count, 3);

        // Test single element
        let result = tree.query_range(2, 3);
        assert_float_eq(result.min, 3.0);
        assert_float_eq(result.max, 3.0);
        assert_float_eq(result.sum, 3.0);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_multiple_batches() {
        let mut tree = SegmentTree::new();
        let values = vec![1.0, 2.0, 3.0];
        tree.add_batch(&values);

        let values = vec![4.0, 5.0];
        tree.add_batch(&values);

        // Test full range
        let result = tree.query_range(0, 5);
        assert_float_eq(result.min, 1.0);
        assert_float_eq(result.max, 5.0);
        assert_float_eq(result.sum, 15.0);
        assert_float_eq(result.sum_squares, 55.0);
        assert_eq!(result.count, 5);
        assert_float_eq(result.last, 5.0);

        // Test partial ranges
        let result = tree.query_range(1, 4);
        assert_float_eq(result.min, 2.0);
        assert_float_eq(result.max, 4.0);
        assert_float_eq(result.sum, 9.0);
        assert_eq!(result.count, 3);

        // Test single element
        let result = tree.query_range(3, 4);
        assert_float_eq(result.min, 4.0);
        assert_float_eq(result.max, 4.0);
        assert_float_eq(result.sum, 4.0);
        assert_eq!(result.count, 1);
    }
    #[test]
    fn test_infinity_handling() {
        let mut tree = SegmentTree::new();

        // Empty tree should return infinity values
        let result = tree.query_range(0, 1);
        assert!(result.min.is_infinite() && result.min.is_sign_positive());
        assert!(result.max.is_infinite() && result.max.is_sign_negative());
        assert_eq!(result.count, 0);

        // After adding values, empty ranges should still have infinity
        tree.add_batch(&[1.0, 2.0, 3.0]);
        let empty_result = tree.query_range(5, 5);
        assert!(empty_result.min.is_infinite() && empty_result.min.is_sign_positive());
        assert!(empty_result.max.is_infinite() && empty_result.max.is_sign_negative());
        assert_eq!(empty_result.count, 0);
    }

    #[test]
    fn test_floating_point_precision() {
        let mut tree = SegmentTree::new();
        let values = vec![1e-10, 1e10, -1e-10, -1e10];
        tree.add_batch(&values);

        let result = tree.query_range(0, 4);
        assert_float_eq(result.min, -1e10);
        assert_float_eq(result.max, 1e10);
        assert_float_eq(result.sum, 0.0);
        assert_eq!(result.count, 4);
    }

    #[test]
    fn test_nan_handling() {
        let mut tree = SegmentTree::new();
        let values = vec![1.0, f64::NAN, 3.0];
        tree.add_batch(&values);

        let result = tree.query_range(0, 3);
        assert!(result.sum.is_nan());
        assert!(result.sum_squares.is_nan());
        assert_eq!(result.count, 3);
    }

    #[test]
    fn test_large_batch_size() {
        let mut tree = SegmentTree::new();
        let values = vec![1.0; 9_000];
        tree.add_batch(&values);
        tree.add_batch(&values);

        let result = tree.query_range(0, 18_000);
        assert_float_eq(result.min, 1.0);
        assert_float_eq(result.max, 1.0);
        assert_float_eq(result.sum, 18_000.0);
        assert_float_eq(result.sum_squares, 18_000.0);
        assert_eq!(result.count, 18_000);
    }
}
