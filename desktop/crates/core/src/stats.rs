//! 统计视图：全部从 answer / session 聚合，不维护缓存表。
use serde::{Deserialize, Serialize};

/// 仪表盘顶部的四个数字。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub bank_size: i64,
    pub seen_count: i64,
    pub attempts: i64,
    pub rights: i64,
    pub wrong_pending: i64,
    pub wrong_mastered: i64,
    pub wrong_due_now: i64,
    pub last_exam_score: Option<i64>,
    pub best_exam_score: Option<i64>,
    pub avg_seconds: Option<f64>,
}

/// 单个领域的累计正确率。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainStat {
    pub domain: String,
    pub number: u8,
    pub name_cn: String,
    pub official_range: String,
    pub attempts: i64,
    pub rights: i64,
}

impl DomainStat {
    pub fn rate(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            self.rights as f64 / self.attempts as f64
        }
    }
}

/// 单个考点的累计正确率，用于「最该补的考点」。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicStat {
    pub topic: String,
    pub domain: String,
    pub attempts: i64,
    pub rights: i64,
    pub bank_count: i64,
}

/// 成绩趋势折线图的一个点。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamPoint {
    pub session_id: i64,
    pub at: i64,
    pub score: i64,
    pub correct: i64,
    pub total: i64,
}

/// 错题掌握度分布（1-5 级各多少题）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoxBucket {
    pub box_level: i64,
    pub count: i64,
}

/// 上考场准备度：四项硬指标，全满足才建议约考。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Readiness {
    pub two_exams_over_800: bool,
    pub all_domains_over_80: bool,
    pub wrongbook_clear: bool,
    pub coverage_over_90: bool,
    pub ready: bool,
    pub next_step: String,
}
