use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketStat {
    pub bucket_label: String,
    pub min_conf: u8,
    pub max_conf: u8,
    pub total: usize,
    pub true_count: usize,
    pub actual_accuracy: f32, // percentage (0.0 - 100.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub total_resolved: usize,
    pub brier_score: f32,
    pub buckets: Vec<BucketStat>,
}

/// Calculate the Brier score: average of (p - outcome)^2
/// p = confidence / 100.0, outcome = 1.0 or 0.0
pub fn compute_brier_score(decisions: &[(u8, i32)]) -> f32 {
    if decisions.is_empty() {
        return 0.0;
    }
    let sum_sq_err: f32 = decisions
        .iter()
        .map(|&(conf, outcome)| {
            let p = (conf as f32) / 100.0;
            let o = if outcome > 0 { 1.0 } else { 0.0 };
            let diff = p - o;
            diff * diff
        })
        .sum();

    sum_sq_err / (decisions.len() as f32)
}

/// Generate calibration buckets: 50-59, 60-69, 70-79, 80-89, 90-99
pub fn generate_calibration_report(decisions: &[(u8, i32)]) -> CalibrationReport {
    let brier_score = compute_brier_score(decisions);
    let bucket_ranges = [
        ("50-59%", 50, 59),
        ("60-69%", 60, 69),
        ("70-79%", 70, 79),
        ("80-89%", 80, 89),
        ("90-99%", 90, 99),
    ];

    let mut buckets = Vec::new();
    for (label, min_c, max_c) in bucket_ranges {
        let items: Vec<_> = decisions
            .iter()
            .filter(|(c, _)| *c >= min_c && *c <= max_c)
            .collect();
        let total = items.len();
        let true_count = items.iter().filter(|(_, o)| *o > 0).count();
        let actual_accuracy = if total > 0 {
            (true_count as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        buckets.push(BucketStat {
            bucket_label: label.to_string(),
            min_conf: min_c,
            max_conf: max_c,
            total,
            true_count,
            actual_accuracy,
        });
    }

    CalibrationReport {
        total_resolved: decisions.len(),
        brier_score,
        buckets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_calibration() {
        let decisions = vec![(100, 1), (100, 1), (0, 0)];
        let brier = compute_brier_score(&decisions);
        assert_eq!(brier, 0.0);
    }

    #[test]
    fn test_coin_flip_calibration() {
        // 50% confidence on 1 and 0
        let decisions = vec![(50, 1), (50, 0)];
        let brier = compute_brier_score(&decisions);
        assert!((brier - 0.25).abs() < 1e-4);
    }

    #[test]
    fn test_bucket_statistics() {
        let decisions = vec![
            (70, 1),
            (75, 1),
            (78, 0), // 70-79: 2/3 true = 66.67%
            (95, 1), // 90-99: 1/1 true = 100%
        ];
        let report = generate_calibration_report(&decisions);
        assert_eq!(report.total_resolved, 4);
        let b70 = report.buckets.iter().find(|b| b.min_conf == 70).unwrap();
        assert_eq!(b70.total, 3);
        assert_eq!(b70.true_count, 2);
    }
}
