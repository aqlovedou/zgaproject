//! 领域模型。所有类型都带 serde 派生，前端通过 Tauri 命令拿到的就是这些结构。
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 官方技能大纲的三大领域。权重用于模拟考组卷。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Domain {
    CloudConcepts,
    ArchitectureServices,
    ManagementGovernance,
}

impl Domain {
    pub const ALL: [Domain; 3] = [
        Domain::CloudConcepts,
        Domain::ArchitectureServices,
        Domain::ManagementGovernance,
    ];

    /// 组卷权重，取自官方大纲区间的中值（25-30% / 35-40% / 30-35%）。
    pub fn weight(self) -> f64 {
        match self {
            Domain::CloudConcepts => 0.275,
            Domain::ArchitectureServices => 0.375,
            Domain::ManagementGovernance => 0.325,
        }
    }

    pub fn number(self) -> u8 {
        match self {
            Domain::CloudConcepts => 1,
            Domain::ArchitectureServices => 2,
            Domain::ManagementGovernance => 3,
        }
    }

    pub fn name_cn(self) -> &'static str {
        match self {
            Domain::CloudConcepts => "云概念",
            Domain::ArchitectureServices => "Azure 架构与服务",
            Domain::ManagementGovernance => "Azure 管理与治理",
        }
    }

    /// 官方公布的权重区间，界面上原样展示。
    pub fn official_range(self) -> &'static str {
        match self {
            Domain::CloudConcepts => "25–30%",
            Domain::ArchitectureServices => "35–40%",
            Domain::ManagementGovernance => "30–35%",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Domain::CloudConcepts => "cloud-concepts",
            Domain::ArchitectureServices => "architecture-services",
            Domain::ManagementGovernance => "management-governance",
        }
    }

    pub fn parse(s: &str) -> Option<Domain> {
        match s {
            "cloud-concepts" => Some(Domain::CloudConcepts),
            "architecture-services" => Some(Domain::ArchitectureServices),
            "management-governance" => Some(Domain::ManagementGovernance),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionType {
    Single,
    Multi,
    TrueFalse,
}

impl QuestionType {
    pub fn as_str(self) -> &'static str {
        match self {
            QuestionType::Single => "single",
            QuestionType::Multi => "multi",
            QuestionType::TrueFalse => "truefalse",
        }
    }

    pub fn parse(s: &str) -> Option<QuestionType> {
        match s {
            "single" => Some(QuestionType::Single),
            "multi" => Some(QuestionType::Multi),
            "truefalse" => Some(QuestionType::TrueFalse),
            _ => None,
        }
    }

    pub fn label_cn(self) -> &'static str {
        match self {
            QuestionType::Single => "单选题",
            QuestionType::Multi => "多选题",
            QuestionType::TrueFalse => "判断题",
        }
    }
}

/// 一道题。`option_reasons` 是每个错误选项「为什么错」，答错时展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub id: String,
    pub domain: Domain,
    pub topic: String,
    #[serde(rename = "type")]
    pub question_type: QuestionType,
    pub question: String,
    pub options: BTreeMap<String, String>,
    pub answer: Vec<String>,
    pub explanation: String,
    pub option_reasons: BTreeMap<String, String>,
    pub reference_title: String,
    pub reference_url: String,
}

impl Question {
    /// 判分：选项集合完全相等才算对（多选题少选、多选都算错）。
    pub fn is_correct(&self, picked: &[String]) -> bool {
        if picked.len() != self.answer.len() {
            return false;
        }
        let mut a: Vec<&str> = picked.iter().map(String::as_str).collect();
        let mut b: Vec<&str> = self.answer.iter().map(String::as_str).collect();
        a.sort_unstable();
        a.dedup();
        b.sort_unstable();
        b.dedup();
        a == b
    }

    /// 某个错误选项为什么错；题库没写就给一句兜底。
    pub fn wrong_reason(&self, key: &str) -> &str {
        self.option_reasons
            .get(key)
            .map(String::as_str)
            .unwrap_or("该选项与题干要求的场景不符。")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionKind {
    Exam,
    Practice,
    Review,
}

impl SessionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SessionKind::Exam => "exam",
            SessionKind::Practice => "practice",
            SessionKind::Review => "review",
        }
    }

    pub fn parse(s: &str) -> Option<SessionKind> {
        match s {
            "exam" => Some(SessionKind::Exam),
            "practice" => Some(SessionKind::Practice),
            "review" => Some(SessionKind::Review),
            _ => None,
        }
    }
}

/// 一次考试／练习／复习。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub kind: SessionKind,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub total: i64,
    pub correct: i64,
    pub score: i64,
    pub elapsed_sec: i64,
    pub time_limit_sec: Option<i64>,
    pub timed_out: bool,
}

/// 一道题的作答明细。所有统计都从这张表聚合，不另存统计缓存。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerRecord {
    pub session_id: i64,
    pub question_id: String,
    pub seq: i64,
    pub picked: Vec<String>,
    pub correct: bool,
    pub seconds: f64,
    pub answered_at: i64,
}

/// 错题本条目，带 Leitner 间隔重复状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WrongEntry {
    pub question_id: String,
    pub wrong_count: i64,
    pub right_streak: i64,
    /// 掌握度 1-5，越高间隔越长。
    pub box_level: i64,
    pub mastered: bool,
    pub first_wrong_at: i64,
    pub last_wrong_at: Option<i64>,
    pub last_picked: Vec<String>,
    pub due_at: i64,
}
