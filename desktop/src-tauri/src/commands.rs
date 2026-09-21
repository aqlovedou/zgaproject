//! Tauri 命令：前端 invoke 的全部入口。
//!
//! 这一层只做「参数转换 + 调用 core + 包装错误」，业务逻辑都在 az900-core 里，
//! 那边有完整的单元测试。
use crate::state::{now_ms, AppState};
use az900_core::model::*;
use az900_core::scoring::{self, DomainScore};
use az900_core::stats::*;
use az900_core::{exam, leitner};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tauri::State;

/// 前端拿到的错误就是一句人话。
pub type CmdResult<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// 一套题：会话 id + 题目 + 限时。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paper {
    pub session_id: i64,
    pub kind: SessionKind,
    pub questions: Vec<Question>,
    pub time_limit_sec: Option<i64>,
    pub label: String,
}

/// 专项练习的筛选条件。
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeFilter {
    pub domain: Option<String>,
    pub topic: Option<String>,
    pub topic_like: Option<String>,
    pub keyword: Option<String>,
    pub only_unseen: Option<bool>,
    pub limit: Option<usize>,
}

/// 交卷结果：成绩 + 领域拆分 + 错题清单。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResult {
    pub session: Session,
    pub domains: Vec<DomainResult>,
    pub wrong_questions: Vec<WrongReview>,
    pub passed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainResult {
    pub domain: String,
    pub number: u8,
    pub name_cn: String,
    pub correct: i64,
    pub total: i64,
}

/// 错题讲评：题目 + 你选了什么。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrongReview {
    pub question: Question,
    pub picked: Vec<String>,
}

/// 错题本的一行：错题状态 + 题目全文。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrongRow {
    pub entry: WrongEntry,
    pub question: Question,
    pub due_now: bool,
}

/// 提交一题后的回执。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerOutcome {
    pub correct: bool,
    pub answer: Vec<String>,
    /// 更新后的错题本状态（答对且不在错题本里时为 null）。
    pub wrong_entry: Option<WrongEntry>,
}

// ---------------- 题库 ----------------

#[tauri::command]
pub fn get_bank_size(state: State<AppState>) -> usize {
    state.bank.len()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicRow {
    pub topic: String,
    pub domain: String,
    pub number: u8,
    pub name_cn: String,
    pub official_range: String,
    pub count: usize,
}

#[tauri::command]
pub fn get_topics(state: State<AppState>) -> Vec<TopicRow> {
    let mut seen: Vec<TopicRow> = Vec::new();
    for q in &state.bank {
        if let Some(row) = seen.iter_mut().find(|r| r.topic == q.topic) {
            row.count += 1;
        } else {
            seen.push(TopicRow {
                topic: q.topic.clone(),
                domain: q.domain.as_str().to_string(),
                number: q.domain.number(),
                name_cn: q.domain.name_cn().to_string(),
                official_range: q.domain.official_range().to_string(),
                count: 1,
            });
        }
    }
    seen.sort_by_key(|r| (r.number, r.topic.clone()));
    seen
}

#[tauri::command]
pub fn get_question(state: State<AppState>, id: String) -> CmdResult<Option<Question>> {
    Ok(state.bank.iter().find(|q| q.id == id).cloned())
}

// ---------------- 开考 ----------------

#[tauri::command]
pub fn start_exam(state: State<AppState>, count: usize) -> CmdResult<Paper> {
    let mut rng = rand::thread_rng();
    let questions = exam::build_exam(&state.bank, count, &mut rng, &HashSet::new());
    if questions.is_empty() {
        return Err("题库是空的，无法组卷。".into());
    }
    // 真实考试 45 分钟 50 题，按同样的节奏折算（每题 54 秒）
    let limit = (questions.len() as f64 * 54.0).round() as i64;
    let db = state.db.lock().map_err(err)?;
    let session_id = db
        .start_session(SessionKind::Exam, questions.len() as i64, now_ms(), Some(limit))
        .map_err(err)?;
    Ok(Paper {
        session_id,
        kind: SessionKind::Exam,
        label: format!("全真模拟考 · {} 题", questions.len()),
        questions,
        time_limit_sec: Some(limit),
    })
}

#[tauri::command]
pub fn start_practice(state: State<AppState>, filter: PracticeFilter) -> CmdResult<Paper> {
    let db = state.db.lock().map_err(err)?;
    let seen = if filter.only_unseen.unwrap_or(false) {
        db.seen_question_ids().map_err(err)?
    } else {
        HashSet::new()
    };

    let keyword = filter.keyword.as_deref().map(str::to_lowercase);
    let mut pool: Vec<Question> = state
        .bank
        .iter()
        .filter(|q| {
            if let Some(d) = &filter.domain {
                if q.domain.as_str() != d {
                    return false;
                }
            }
            if let Some(t) = &filter.topic {
                if &q.topic != t {
                    return false;
                }
            }
            if let Some(t) = &filter.topic_like {
                if !q.topic.contains(t.as_str()) {
                    return false;
                }
            }
            if let Some(k) = &keyword {
                let hit = q.question.to_lowercase().contains(k)
                    || q.topic.to_lowercase().contains(k)
                    || q.options.values().any(|v| v.to_lowercase().contains(k));
                if !hit {
                    return false;
                }
            }
            !seen.contains(&q.id)
        })
        .cloned()
        .collect();

    if pool.is_empty() {
        return Err("没有符合条件的题目。".into());
    }

    use rand::seq::SliceRandom;
    pool.shuffle(&mut rand::thread_rng());
    if let Some(n) = filter.limit {
        pool.truncate(n.max(1));
    }

    let label = filter
        .topic
        .or(filter.topic_like)
        .or(filter.keyword)
        .unwrap_or_else(|| "专项练习".into());

    let session_id = db
        .start_session(SessionKind::Practice, pool.len() as i64, now_ms(), None)
        .map_err(err)?;
    Ok(Paper {
        session_id,
        kind: SessionKind::Practice,
        label: format!("{label} · {} 题", pool.len()),
        questions: pool,
        time_limit_sec: None,
    })
}

#[tauri::command]
pub fn start_review(state: State<AppState>) -> CmdResult<Paper> {
    let now = now_ms();
    let db = state.db.lock().map_err(err)?;

    // 优先复习到期的；一道都没到期就复习全部未攻克的
    let due = db.due_entries(now).map_err(err)?;
    let (entries, label) = if due.is_empty() {
        (db.wrong_entries(false).map_err(err)?, "错题复习")
    } else {
        (due, "到期错题复习")
    };
    if entries.is_empty() {
        return Err("错题本是空的，先去做一套模拟考。".into());
    }

    let ids: Vec<String> = entries.iter().map(|e| e.question_id.clone()).collect();
    let mut questions: Vec<Question> = state
        .bank
        .iter()
        .filter(|q| ids.contains(&q.id))
        .cloned()
        .collect();
    use rand::seq::SliceRandom;
    questions.shuffle(&mut rand::thread_rng());

    let session_id = db
        .start_session(SessionKind::Review, questions.len() as i64, now, None)
        .map_err(err)?;
    Ok(Paper {
        session_id,
        kind: SessionKind::Review,
        label: format!("{label} · {} 题", questions.len()),
        questions,
        time_limit_sec: None,
    })
}

// ---------------- 答题与交卷 ----------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitArgs {
    pub session_id: i64,
    pub question_id: String,
    pub seq: i64,
    pub picked: Vec<String>,
    pub seconds: f64,
    pub kind: SessionKind,
}

/// 提交一道题：判分由后端做，同时更新错题本。
#[tauri::command]
pub fn submit_answer(state: State<AppState>, args: SubmitArgs) -> CmdResult<AnswerOutcome> {
    let question = state
        .bank
        .iter()
        .find(|q| q.id == args.question_id)
        .ok_or_else(|| format!("题库里没有 {}", args.question_id))?;

    let correct = question.is_correct(&args.picked);
    let now = now_ms();
    let db = state.db.lock().map_err(err)?;

    db.record_answer(&AnswerRecord {
        session_id: args.session_id,
        question_id: question.id.clone(),
        seq: args.seq,
        picked: args.picked.clone(),
        correct,
        seconds: args.seconds,
        answered_at: now,
    })
    .map_err(err)?;

    // 复习模式下答对要升级掌握度；其它模式下答对不动错题本
    let wrong_entry = match (args.kind, correct) {
        (SessionKind::Review, true) => db.mark_review_right(&question.id, now).map_err(err)?,
        (_, false) => Some(
            db.mark_wrong(&question.id, args.picked.clone(), now)
                .map_err(err)?,
        ),
        _ => None,
    };

    Ok(AnswerOutcome {
        correct,
        answer: question.answer.clone(),
        wrong_entry,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishArgs {
    pub session_id: i64,
    pub elapsed_sec: i64,
    pub timed_out: bool,
}

#[tauri::command]
pub fn finish_session(state: State<AppState>, args: FinishArgs) -> CmdResult<SessionResult> {
    let db = state.db.lock().map_err(err)?;
    let session = db
        .finish_session(args.session_id, now_ms(), args.elapsed_sec, args.timed_out)
        .map_err(err)?;

    let answers = db.session_answers(args.session_id).map_err(err)?;
    let by_id: HashMap<&str, &AnswerRecord> =
        answers.iter().map(|a| (a.question_id.as_str(), a)).collect();

    let questions: Vec<Question> = answers
        .iter()
        .filter_map(|a| state.bank.iter().find(|q| q.id == a.question_id).cloned())
        .collect();

    let results: HashMap<String, bool> =
        answers.iter().map(|a| (a.question_id.clone(), a.correct)).collect();
    let breakdown = scoring::domain_breakdown(&questions, &results);

    let mut domains: Vec<DomainResult> = Domain::ALL
        .iter()
        .map(|d| {
            let s = breakdown.get(d).copied().unwrap_or(DomainScore { correct: 0, total: 0 });
            DomainResult {
                domain: d.as_str().to_string(),
                number: d.number(),
                name_cn: d.name_cn().to_string(),
                correct: s.correct,
                total: s.total,
            }
        })
        .collect();
    domains.sort_by_key(|d| d.number);

    let wrong_questions: Vec<WrongReview> = questions
        .iter()
        .filter(|q| !results.get(&q.id).copied().unwrap_or(false))
        .map(|q| WrongReview {
            question: q.clone(),
            picked: by_id.get(q.id.as_str()).map(|a| a.picked.clone()).unwrap_or_default(),
        })
        .collect();

    Ok(SessionResult {
        passed: scoring::passed(session.score),
        session,
        domains,
        wrong_questions,
    })
}

// ---------------- 错题本 ----------------

#[tauri::command]
pub fn get_wrongbook(state: State<AppState>, include_mastered: bool) -> CmdResult<Vec<WrongRow>> {
    let now = now_ms();
    let db = state.db.lock().map_err(err)?;
    let entries = db.wrong_entries(include_mastered).map_err(err)?;
    Ok(entries
        .into_iter()
        .filter_map(|e| {
            state.bank.iter().find(|q| q.id == e.question_id).map(|q| WrongRow {
                due_now: leitner::is_due(&e, now),
                entry: e,
                question: q.clone(),
            })
        })
        .collect())
}

#[tauri::command]
pub fn clear_wrongbook(state: State<AppState>) -> CmdResult<usize> {
    let db = state.db.lock().map_err(err)?;
    db.clear_wrongbook().map_err(err)
}

// ---------------- 统计 ----------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub overview: Overview,
    pub domains: Vec<DomainStat>,
    pub exams: Vec<ExamPoint>,
    pub boxes: Vec<BoxBucket>,
    pub weak_topics: Vec<TopicStat>,
    pub readiness: Readiness,
    pub pass_score: i64,
    pub max_score: i64,
}

/// 仪表盘需要的所有数据，一次命令取回，避免前端连发六个请求。
#[tauri::command]
pub fn get_dashboard(state: State<AppState>) -> CmdResult<Dashboard> {
    let now = now_ms();
    let db = state.db.lock().map_err(err)?;
    let mut weak_topics = db.topic_stats(3).map_err(err)?;
    weak_topics.truncate(8);
    Ok(Dashboard {
        overview: db.overview(now).map_err(err)?,
        domains: db.domain_stats().map_err(err)?,
        exams: db.exam_history().map_err(err)?,
        boxes: db.box_distribution().map_err(err)?,
        weak_topics,
        readiness: db.readiness(now).map_err(err)?,
        pass_score: scoring::PASS_SCORE,
        max_score: scoring::MAX_SCORE,
    })
}

#[tauri::command]
pub fn get_db_path(state: State<AppState>) -> String {
    state.db_path.display().to_string()
}
