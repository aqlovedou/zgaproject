//! 评分：折算成官方的 1000 分制。
use crate::model::{Domain, Question};
use std::collections::HashMap;

/// 官方及格线：满分 1000，700 分通过。
pub const PASS_SCORE: i64 = 700;
pub const MAX_SCORE: i64 = 1000;

/// 把「答对 n / 共 m」折算成 1000 分制。m 为 0 时返回 0 而不是 panic。
pub fn scaled_score(correct: i64, total: i64) -> i64 {
    if total <= 0 {
        return 0;
    }
    ((correct as f64 / total as f64) * MAX_SCORE as f64).round() as i64
}

pub fn passed(score: i64) -> bool {
    score >= PASS_SCORE
}

/// 某个领域的得分情况。
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainScore {
    pub correct: i64,
    pub total: i64,
}

impl DomainScore {
    pub fn rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.correct as f64 / self.total as f64
        }
    }
}

/// 按领域拆分一份卷子的得分。`results` 是「题目 → 是否答对」。
pub fn domain_breakdown(
    questions: &[Question],
    results: &HashMap<String, bool>,
) -> HashMap<Domain, DomainScore> {
    let mut out: HashMap<Domain, DomainScore> = Domain::ALL
        .iter()
        .map(|d| (*d, DomainScore { correct: 0, total: 0 }))
        .collect();
    for q in questions {
        let entry = out.get_mut(&q.domain).expect("三个领域都已预置");
        entry.total += 1;
        if results.get(&q.id).copied().unwrap_or(false) {
            entry.correct += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 及格线是五十题答对三十五题() {
        assert_eq!(scaled_score(35, 50), 700);
        assert!(passed(scaled_score(35, 50)));
        assert!(!passed(scaled_score(34, 50)));
    }

    #[test]
    fn 满分和零分() {
        assert_eq!(scaled_score(50, 50), MAX_SCORE);
        assert_eq!(scaled_score(0, 50), 0);
    }

    #[test]
    fn 题数为零不会崩() {
        assert_eq!(scaled_score(0, 0), 0);
        assert_eq!(scaled_score(3, 0), 0);
    }

    #[test]
    fn 领域正确率在零到一之间() {
        let s = DomainScore { correct: 7, total: 10 };
        assert!((s.rate() - 0.7).abs() < 1e-9);
        let empty = DomainScore { correct: 0, total: 0 };
        assert_eq!(empty.rate(), 0.0);
    }
}
