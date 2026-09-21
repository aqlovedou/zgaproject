//! AZ-900 刷题台的核心层：领域模型、组卷、判分、间隔重复、SQLite 存储。
//!
//! 这个 crate 不依赖 Tauri，可以单独跑单元测试；`src-tauri` 只是它的一层薄壳。
pub mod db;
pub mod exam;
pub mod leitner;
pub mod model;
pub mod scoring;
pub mod stats;

pub use db::Db;
pub use model::{
    AnswerRecord, Domain, Question, QuestionType, Session, SessionKind, WrongEntry,
};
pub use scoring::{passed, scaled_score, MAX_SCORE, PASS_SCORE};

use anyhow::{Context, Result};
use std::path::Path;

/// 从题库 JSON 目录读题（data/questions/*.json）。
pub fn load_bank_dir(dir: impl AsRef<Path>) -> Result<Vec<Question>> {
    let dir = dir.as_ref();
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("读不到题库目录 {}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();

    let mut out = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("读不了 {}", path.display()))?;
        out.extend(parse_bank_file(&text).with_context(|| format!("解析失败 {}", path.display()))?);
    }
    Ok(out)
}

/// 解析一个题库文件。JSON 里的字段名是蛇形，这里转成领域模型。
pub fn parse_bank_file(text: &str) -> Result<Vec<Question>> {
    #[derive(serde::Deserialize)]
    struct Reference {
        title: String,
        url: String,
    }
    #[derive(serde::Deserialize)]
    struct Raw {
        id: String,
        domain: String,
        topic: String,
        #[serde(rename = "type")]
        qtype: String,
        question: String,
        options: std::collections::BTreeMap<String, String>,
        answer: Vec<String>,
        explanation: String,
        #[serde(default)]
        option_reasons: std::collections::BTreeMap<String, String>,
        reference: Reference,
    }
    #[derive(serde::Deserialize)]
    struct File {
        questions: Vec<Raw>,
    }

    let file: File = serde_json::from_str(text)?;
    file.questions
        .into_iter()
        .map(|r| {
            Ok(Question {
                domain: Domain::parse(&r.domain)
                    .with_context(|| format!("{} 的领域无法识别：{}", r.id, r.domain))?,
                question_type: QuestionType::parse(&r.qtype)
                    .with_context(|| format!("{} 的题型无法识别：{}", r.id, r.qtype))?,
                id: r.id,
                topic: r.topic,
                question: r.question,
                options: r.options,
                answer: r.answer,
                explanation: r.explanation,
                option_reasons: r.option_reasons,
                reference_title: r.reference.title,
                reference_url: r.reference.url,
            })
        })
        .collect()
}
