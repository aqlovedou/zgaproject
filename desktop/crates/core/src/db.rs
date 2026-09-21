//! SQLite 数据层。所有 SQL 集中在这里，上层拿到的都是领域模型。
use crate::leitner;
use crate::model::*;
use crate::scoring;
use crate::stats::*;
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

const SCHEMA: &str = include_str!("schema.sql");

pub struct Db {
    conn: Connection,
}

fn json_map(s: &str) -> Result<BTreeMap<String, String>> {
    Ok(serde_json::from_str(s)?)
}
fn json_vec(s: &str) -> Result<Vec<String>> {
    Ok(serde_json::from_str(s)?)
}

impl Db {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).ok();
        }
        let conn = Connection::open(path)
            .with_context(|| format!("打不开数据库 {}", path.display()))?;
        let db = Db { conn };
        db.migrate()?;
        Ok(db)
    }

    /// 测试用的内存库。
    pub fn open_in_memory() -> Result<Self> {
        let db = Db { conn: Connection::open_in_memory()? };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA).context("建表失败")?;
        Ok(())
    }

    // ---------------- 题库 ----------------

    /// 导入／更新题库。同一个 id 重复导入是覆盖，不会产生重复行。
    pub fn import_questions(&mut self, questions: &[Question]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO question
                   (id, domain, topic, type, question, options, answer,
                    explanation, option_reasons, ref_title, ref_url)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
                 ON CONFLICT(id) DO UPDATE SET
                   domain=excluded.domain, topic=excluded.topic, type=excluded.type,
                   question=excluded.question, options=excluded.options,
                   answer=excluded.answer, explanation=excluded.explanation,
                   option_reasons=excluded.option_reasons,
                   ref_title=excluded.ref_title, ref_url=excluded.ref_url",
            )?;
            for q in questions {
                stmt.execute(params![
                    q.id,
                    q.domain.as_str(),
                    q.topic,
                    q.question_type.as_str(),
                    q.question,
                    serde_json::to_string(&q.options)?,
                    serde_json::to_string(&q.answer)?,
                    q.explanation,
                    serde_json::to_string(&q.option_reasons)?,
                    q.reference_title,
                    q.reference_url,
                ])?;
            }
        }
        tx.commit()?;
        Ok(questions.len())
    }

    fn row_to_question(row: &Row) -> rusqlite::Result<Question> {
        let domain: String = row.get("domain")?;
        let qtype: String = row.get("type")?;
        let options: String = row.get("options")?;
        let answer: String = row.get("answer")?;
        let reasons: String = row.get("option_reasons")?;
        Ok(Question {
            id: row.get("id")?,
            domain: Domain::parse(&domain).unwrap_or(Domain::CloudConcepts),
            topic: row.get("topic")?,
            question_type: QuestionType::parse(&qtype).unwrap_or(QuestionType::Single),
            question: row.get("question")?,
            options: json_map(&options).unwrap_or_default(),
            answer: json_vec(&answer).unwrap_or_default(),
            explanation: row.get("explanation")?,
            option_reasons: json_map(&reasons).unwrap_or_default(),
            reference_title: row.get("ref_title")?,
            reference_url: row.get("ref_url")?,
        })
    }

    pub fn all_questions(&self) -> Result<Vec<Question>> {
        let mut stmt = self.conn.prepare("SELECT * FROM question ORDER BY id")?;
        let rows = stmt.query_map([], Self::row_to_question)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn question(&self, id: &str) -> Result<Option<Question>> {
        let mut stmt = self.conn.prepare("SELECT * FROM question WHERE id = ?1")?;
        Ok(stmt.query_row([id], Self::row_to_question).optional()?)
    }

    pub fn questions_by_ids(&self, ids: &[String]) -> Result<Vec<Question>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(q) = self.question(id)? {
                out.push(q);
            }
        }
        Ok(out)
    }

    pub fn bank_size(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM question", [], |r| r.get(0))?)
    }

    // ---------------- 会话与作答 ----------------

    pub fn start_session(
        &self,
        kind: SessionKind,
        total: i64,
        started_at: i64,
        time_limit_sec: Option<i64>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO session (kind, started_at, total, time_limit_sec)
             VALUES (?1,?2,?3,?4)",
            params![kind.as_str(), started_at, total, time_limit_sec],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 记录一道题的作答。同一会话同一题重复提交会覆盖。
    pub fn record_answer(&self, a: &AnswerRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO answer (session_id, question_id, seq, picked, correct, seconds, answered_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(session_id, question_id) DO UPDATE SET
               picked=excluded.picked, correct=excluded.correct,
               seconds=excluded.seconds, answered_at=excluded.answered_at",
            params![
                a.session_id,
                a.question_id,
                a.seq,
                serde_json::to_string(&a.picked)?,
                a.correct as i64,
                a.seconds,
                a.answered_at
            ],
        )?;
        Ok(())
    }

    /// 交卷：从 answer 表实时统计答对数并折算分数，不接受外部传入的分数。
    pub fn finish_session(
        &self,
        session_id: i64,
        finished_at: i64,
        elapsed_sec: i64,
        timed_out: bool,
    ) -> Result<Session> {
        let correct: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(correct),0) FROM answer WHERE session_id = ?1",
            [session_id],
            |r| r.get(0),
        )?;
        let total: i64 = self.conn.query_row(
            "SELECT total FROM session WHERE id = ?1",
            [session_id],
            |r| r.get(0),
        )?;
        let score = scoring::scaled_score(correct, total);
        self.conn.execute(
            "UPDATE session SET finished_at=?2, correct=?3, score=?4, elapsed_sec=?5, timed_out=?6
             WHERE id=?1",
            params![session_id, finished_at, correct, score, elapsed_sec, timed_out as i64],
        )?;
        self.session(session_id)?
            .context("交卷后找不到该会话")
    }

    fn row_to_session(row: &Row) -> rusqlite::Result<Session> {
        let kind: String = row.get("kind")?;
        let timed_out: i64 = row.get("timed_out")?;
        Ok(Session {
            id: row.get("id")?,
            kind: SessionKind::parse(&kind).unwrap_or(SessionKind::Practice),
            started_at: row.get("started_at")?,
            finished_at: row.get("finished_at")?,
            total: row.get("total")?,
            correct: row.get("correct")?,
            score: row.get("score")?,
            elapsed_sec: row.get("elapsed_sec")?,
            time_limit_sec: row.get("time_limit_sec")?,
            timed_out: timed_out != 0,
        })
    }

    pub fn session(&self, id: i64) -> Result<Option<Session>> {
        let mut stmt = self.conn.prepare("SELECT * FROM session WHERE id = ?1")?;
        Ok(stmt.query_row([id], Self::row_to_session).optional()?)
    }

    pub fn session_answers(&self, session_id: i64) -> Result<Vec<AnswerRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM answer WHERE session_id = ?1 ORDER BY seq",
        )?;
        let rows = stmt.query_map([session_id], |row| {
            let picked: String = row.get("picked")?;
            let correct: i64 = row.get("correct")?;
            Ok(AnswerRecord {
                session_id: row.get("session_id")?,
                question_id: row.get("question_id")?,
                seq: row.get("seq")?,
                picked: json_vec(&picked).unwrap_or_default(),
                correct: correct != 0,
                seconds: row.get("seconds")?,
                answered_at: row.get("answered_at")?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// 成绩趋势：只取已交卷的模拟考。
    pub fn exam_history(&self) -> Result<Vec<ExamPoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, started_at, score, correct, total FROM session
             WHERE kind='exam' AND finished_at IS NOT NULL
             ORDER BY started_at",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(ExamPoint {
                session_id: r.get(0)?,
                at: r.get(1)?,
                score: r.get(2)?,
                correct: r.get(3)?,
                total: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    // ---------------- 错题本 ----------------

    fn row_to_wrong(row: &Row) -> rusqlite::Result<WrongEntry> {
        let picked: String = row.get("last_picked")?;
        let mastered: i64 = row.get("mastered")?;
        Ok(WrongEntry {
            question_id: row.get("question_id")?,
            wrong_count: row.get("wrong_count")?,
            right_streak: row.get("right_streak")?,
            box_level: row.get("box")?,
            mastered: mastered != 0,
            first_wrong_at: row.get("first_wrong_at")?,
            last_wrong_at: row.get("last_wrong_at")?,
            last_picked: json_vec(&picked).unwrap_or_default(),
            due_at: row.get("due_at")?,
        })
    }

    pub fn wrong_entry(&self, question_id: &str) -> Result<Option<WrongEntry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM wrong_entry WHERE question_id = ?1")?;
        Ok(stmt.query_row([question_id], Self::row_to_wrong).optional()?)
    }

    fn save_wrong(&self, e: &WrongEntry) -> Result<()> {
        self.conn.execute(
            "INSERT INTO wrong_entry
               (question_id, wrong_count, right_streak, box, mastered,
                first_wrong_at, last_wrong_at, last_picked, due_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
             ON CONFLICT(question_id) DO UPDATE SET
               wrong_count=excluded.wrong_count, right_streak=excluded.right_streak,
               box=excluded.box, mastered=excluded.mastered,
               last_wrong_at=excluded.last_wrong_at, last_picked=excluded.last_picked,
               due_at=excluded.due_at",
            params![
                e.question_id,
                e.wrong_count,
                e.right_streak,
                e.box_level,
                e.mastered as i64,
                e.first_wrong_at,
                e.last_wrong_at,
                serde_json::to_string(&e.last_picked)?,
                e.due_at
            ],
        )?;
        Ok(())
    }

    /// 答错：新题收录，老题打回 1 级。
    pub fn mark_wrong(&self, question_id: &str, picked: Vec<String>, now: i64) -> Result<WrongEntry> {
        let entry = match self.wrong_entry(question_id)? {
            Some(mut e) => {
                leitner::on_wrong(&mut e, picked, now);
                e
            }
            None => leitner::new_entry(question_id, picked, now),
        };
        self.save_wrong(&entry)?;
        Ok(entry)
    }

    /// 复习答对：升一级、推迟下次；够格则标记攻克。不在错题本里的题忽略。
    pub fn mark_review_right(&self, question_id: &str, now: i64) -> Result<Option<WrongEntry>> {
        let Some(mut e) = self.wrong_entry(question_id)? else {
            return Ok(None);
        };
        leitner::on_right(&mut e, now);
        self.save_wrong(&e)?;
        Ok(Some(e))
    }

    pub fn wrong_entries(&self, include_mastered: bool) -> Result<Vec<WrongEntry>> {
        let sql = if include_mastered {
            "SELECT * FROM wrong_entry ORDER BY mastered, wrong_count DESC, due_at"
        } else {
            "SELECT * FROM wrong_entry WHERE mastered = 0 ORDER BY wrong_count DESC, due_at"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([], Self::row_to_wrong)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// 到期该复习的错题。
    pub fn due_entries(&self, now: i64) -> Result<Vec<WrongEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM wrong_entry WHERE mastered = 0 AND due_at <= ?1
             ORDER BY box, due_at",
        )?;
        let rows = stmt.query_map([now], Self::row_to_wrong)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn clear_wrongbook(&self) -> Result<usize> {
        Ok(self.conn.execute("DELETE FROM wrong_entry", [])?)
    }

    pub fn box_distribution(&self) -> Result<Vec<BoxBucket>> {
        let mut stmt = self.conn.prepare(
            "SELECT box, COUNT(*) FROM wrong_entry GROUP BY box ORDER BY box",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(BoxBucket { box_level: r.get(0)?, count: r.get(1)? })
        })?;
        let found: Vec<BoxBucket> = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        // 界面要画满 1-5 级，缺的补 0
        Ok((1..=5)
            .map(|b| BoxBucket {
                box_level: b,
                count: found.iter().find(|x| x.box_level == b).map_or(0, |x| x.count),
            })
            .collect())
    }

    // ---------------- 统计（全部从 answer 聚合） ----------------

    pub fn domain_stats(&self) -> Result<Vec<DomainStat>> {
        let mut stmt = self.conn.prepare(
            "SELECT q.domain, COUNT(*) AS attempts, COALESCE(SUM(a.correct),0) AS rights
             FROM answer a JOIN question q ON q.id = a.question_id
             GROUP BY q.domain",
        )?;
        let rows = stmt.query_map([], |r| {
            let d: String = r.get(0)?;
            Ok((d, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))
        })?;
        let found: Vec<(String, i64, i64)> = rows.collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(Domain::ALL
            .iter()
            .map(|d| {
                let hit = found.iter().find(|(k, _, _)| k == d.as_str());
                DomainStat {
                    domain: d.as_str().to_string(),
                    number: d.number(),
                    name_cn: d.name_cn().to_string(),
                    official_range: d.official_range().to_string(),
                    attempts: hit.map_or(0, |x| x.1),
                    rights: hit.map_or(0, |x| x.2),
                }
            })
            .collect())
    }

    /// 考点正确率；只统计练过的考点，`min_attempts` 过滤样本太少的。
    pub fn topic_stats(&self, min_attempts: i64) -> Result<Vec<TopicStat>> {
        let mut stmt = self.conn.prepare(
            "SELECT q.topic, q.domain,
                    COUNT(*) AS attempts,
                    COALESCE(SUM(a.correct),0) AS rights,
                    (SELECT COUNT(*) FROM question q2 WHERE q2.topic = q.topic) AS bank_count
             FROM answer a JOIN question q ON q.id = a.question_id
             GROUP BY q.topic, q.domain
             HAVING attempts >= ?1
             ORDER BY (CAST(rights AS REAL) / attempts) ASC, attempts DESC",
        )?;
        let rows = stmt.query_map([min_attempts], |r| {
            Ok(TopicStat {
                topic: r.get(0)?,
                domain: r.get(1)?,
                attempts: r.get(2)?,
                rights: r.get(3)?,
                bank_count: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// 已练过的题目 id（用于「只练没做过的」和覆盖率）。
    pub fn seen_question_ids(&self) -> Result<HashSet<String>> {
        let mut stmt = self.conn.prepare("SELECT DISTINCT question_id FROM answer")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<HashSet<_>>>()?)
    }

    pub fn overview(&self, now: i64) -> Result<Overview> {
        let bank_size = self.bank_size()?;
        let (seen_count, attempts, rights): (i64, i64, i64) = self.conn.query_row(
            "SELECT COUNT(DISTINCT question_id), COUNT(*), COALESCE(SUM(correct),0) FROM answer",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let (wrong_pending, wrong_mastered): (i64, i64) = self.conn.query_row(
            "SELECT SUM(CASE WHEN mastered=0 THEN 1 ELSE 0 END),
                    SUM(CASE WHEN mastered=1 THEN 1 ELSE 0 END)
             FROM wrong_entry",
            [],
            |r| Ok((r.get::<_, Option<i64>>(0)?.unwrap_or(0), r.get::<_, Option<i64>>(1)?.unwrap_or(0))),
        )?;
        let wrong_due_now: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM wrong_entry WHERE mastered=0 AND due_at <= ?1",
            [now],
            |r| r.get(0),
        )?;
        let last_exam_score: Option<i64> = self.conn.query_row(
            "SELECT score FROM session WHERE kind='exam' AND finished_at IS NOT NULL
             ORDER BY started_at DESC LIMIT 1",
            [],
            |r| r.get(0),
        ).optional()?;
        let best_exam_score: Option<i64> = self.conn.query_row(
            "SELECT MAX(score) FROM session WHERE kind='exam' AND finished_at IS NOT NULL",
            [],
            |r| r.get(0),
        ).optional()?.flatten();
        let avg_seconds: Option<f64> = self.conn.query_row(
            "SELECT AVG(seconds) FROM answer",
            [],
            |r| r.get(0),
        ).optional()?.flatten();

        Ok(Overview {
            bank_size,
            seen_count,
            attempts,
            rights,
            wrong_pending,
            wrong_mastered,
            wrong_due_now,
            last_exam_score,
            best_exam_score,
            avg_seconds,
        })
    }

    /// 上考场准备度：四项硬指标。
    pub fn readiness(&self, now: i64) -> Result<Readiness> {
        let history = self.exam_history()?;
        let two_exams_over_800 = history.len() >= 2
            && history[history.len() - 2..].iter().all(|p| p.score >= 800);

        let domains = self.domain_stats()?;
        let all_domains_over_80 = domains
            .iter()
            .all(|d| d.attempts >= 15 && d.rate() >= 0.8);

        let ov = self.overview(now)?;
        let wrongbook_clear = ov.wrong_pending == 0;
        let coverage_over_90 =
            ov.bank_size > 0 && (ov.seen_count as f64 / ov.bank_size as f64) >= 0.9;

        let ready = two_exams_over_800 && all_domains_over_80 && wrongbook_clear && coverage_over_90;

        let next_step = if ready {
            "四项都满足了。保持手感即可，考前一天再刷一套 50 题就够。".to_string()
        } else if ov.wrong_pending > 0 {
            format!("先把错题本清空（还剩 {} 题），再做模拟考。", ov.wrong_pending)
        } else if history.is_empty() {
            "先做一套 50 题全真模拟摸底。".to_string()
        } else {
            "再做一套 50 题全真模拟。".to_string()
        };

        Ok(Readiness {
            two_exams_over_800,
            all_domains_over_80,
            wrongbook_clear,
            coverage_over_90,
            ready,
            next_step,
        })
    }
}
