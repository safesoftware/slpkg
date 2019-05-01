pub fn split_indices_into_ranges(num_entries: usize, num_ranges: usize) -> Vec<(usize, usize)> {
    let max_entries_per_thread = ((num_entries as f64) / (num_ranges as f64)).ceil() as usize;
    let mut ranges = Vec::with_capacity(num_ranges);

    for i in 0..num_ranges {
        let start_index = i * max_entries_per_thread;
        if start_index < num_entries {
            let end_index = std::cmp::min(num_entries, start_index + max_entries_per_thread);
            ranges.push((start_index, end_index));
        }
    }

    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_range() {
        let ranges = split_indices_into_ranges(100, 1);
        assert_eq!(ranges, vec![(0, 100)]);
    }

    #[test]
    fn more_ranges_than_indices() {
        let ranges = split_indices_into_ranges(10, 16);
        assert_eq!(
            ranges,
            vec![
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 8),
                (8, 9),
                (9, 10)
            ]
        )
    }

    #[test]
    fn typical_case() {
        let ranges = split_indices_into_ranges(123460, 16);
        assert_eq!(
            ranges,
            vec![
                (0, 7717),
                (7717, 15434),
                (15434, 23151),
                (23151, 30868),
                (30868, 38585),
                (38585, 46302),
                (46302, 54019),
                (54019, 61736),
                (61736, 69453),
                (69453, 77170),
                (77170, 84887),
                (84887, 92604),
                (92604, 100321),
                (100321, 108038),
                (108038, 115755),
                (115755, 123460),
            ]
        )
    }
}
