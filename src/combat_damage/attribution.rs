pub fn confidence(candidate_count: usize) -> f64 {
    1.0 / candidate_count.max(1) as f64
}

pub fn catalog_weights(ranks: &[Option<i32>]) -> Vec<f64> {
    let sum_ranks: i32 = ranks.iter().filter_map(|rank| *rank).sum();
    if sum_ranks > 0 {
        ranks
            .iter()
            .map(|rank| {
                rank.map(|value| f64::from(value) / f64::from(sum_ranks))
                    .unwrap_or(0.0)
            })
            .collect()
    } else {
        let share = confidence(ranks.len());
        vec![share; ranks.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_confidence_and_weight_are_one() {
        assert!((confidence(1) - 1.0).abs() < 1e-9);
        assert_eq!(catalog_weights(&[Some(4)]), vec![1.0]);
    }

    #[test]
    fn ambiguous_without_ranks_splits_evenly() {
        assert!((confidence(2) - 0.5).abs() < 1e-9);
        let weights = catalog_weights(&[None, None]);
        assert!((weights[0] - 0.5).abs() < 1e-9);
        assert!((weights[1] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn ambiguous_with_ranks_splits_proportionally() {
        let weights = catalog_weights(&[Some(3), Some(5)]);
        assert!((weights[0] - 0.375).abs() < 1e-9);
        assert!((weights[1] - 0.625).abs() < 1e-9);
    }
}
