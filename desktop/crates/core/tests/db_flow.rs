//! 端到端：真实题库 → 考试 → 交卷 → 错题本 → 复习 → 统计。
use az900_core::model::*;
use az900_core::{scoring, Db};
use std::path::PathBuf;

const HOUR: i64 = 3_600_000;

fn bank_dir() -> PathBuf {
    // crates/core → desktop → 仓库根 → data/questions
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../data/questions")
        .canonicalize()
        .expect("找不到题库目录")
}

fn seeded_db() -> Db {
    let questions = az900_core::load_bank_dir(bank_dir()).expect("题库读取失败");
    let mut db = Db::open_in_memory().unwrap();
    db.import_questions(&questions).unwrap();
    db
}

#[test]
fn 真实题库能完整导入() {
    let db = seeded_db();
    assert!(db.bank_size().unwrap() >= 150, "题量应不少于 150");
    let all = db.all_questions().unwrap();
    assert_eq!(all.len() as i64, db.bank_size().unwrap());

    // 每题都要有答案、有解析、有官方出处，且每个错误选项都写明了为什么错
    for q in &all {
        assert!(!q.answer.is_empty(), "{} 缺答案", q.id);
        assert!(q.answer.iter().all(|k| q.options.contains_key(k)), "{} 答案不在选项里", q.id);
        assert!(q.explanation.chars().count() >= 20, "{} 解析过短", q.id);
        assert!(
            q.reference_url.starts_with("https://learn.microsoft.com"),
            "{} 的出处不是官方文档", q.id
        );
        for key in q.options.keys() {
            if !q.answer.contains(key) {
                assert!(
                    q.option_reasons.contains_key(key),
                    "{} 的错误选项 {} 没写为什么错", q.id, key
                );
            }
        }
    }
}

#[test]
fn 重复导入不会产生重复行() {
    let questions = az900_core::load_bank_dir(bank_dir()).unwrap();
    let mut db = Db::open_in_memory().unwrap();
    db.import_questions(&questions).unwrap();
    let first = db.bank_size().unwrap();
    db.import_questions(&questions).unwrap();
    assert_eq!(db.bank_size().unwrap(), first, "同 id 应覆盖而不是新增");
}

#[test]
fn 一次完整的考试流程() {
    let db = seeded_db();
    let all = db.all_questions().unwrap();
    let paper: Vec<&Question> = all.iter().take(10).collect();

    let sid = db.start_session(SessionKind::Exam, 10, 0, Some(540)).unwrap();

    // 前 7 题答对，后 3 题答错
    for (i, q) in paper.iter().enumerate() {
        let right = i < 7;
        let picked = if right {
            q.answer.clone()
        } else {
            // 挑一个不是答案的选项
            vec![q.options.keys().find(|k| !q.answer.contains(k)).unwrap().clone()]
        };
        assert_eq!(q.is_correct(&picked), right);
        db.record_answer(&AnswerRecord {
            session_id: sid,
            question_id: q.id.clone(),
            seq: i as i64 + 1,
            picked: picked.clone(),
            correct: right,
            seconds: 30.0,
            answered_at: i as i64,
        })
        .unwrap();
        if !right {
            db.mark_wrong(&q.id, picked, 0).unwrap();
        }
    }

    let session = db.finish_session(sid, 1000, 300, false).unwrap();
    assert_eq!(session.correct, 7);
    assert_eq!(session.total, 10);
    assert_eq!(session.score, scoring::scaled_score(7, 10));
    assert_eq!(session.score, 700, "7/10 正好压线");
    assert!(scoring::passed(session.score));

    // 分数由服务端从 answer 重算，不信任调用方
    assert_eq!(db.session(sid).unwrap().unwrap().score, 700);

    let wrong = db.wrong_entries(false).unwrap();
    assert_eq!(wrong.len(), 3, "3 道错题进了错题本");
    assert!(wrong.iter().all(|w| w.box_level == 1 && !w.mastered));

    let history = db.exam_history().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].score, 700);
}

#[test]
fn 未交卷的考试不进成绩趋势() {
    let db = seeded_db();
    db.start_session(SessionKind::Exam, 50, 0, Some(2700)).unwrap();
    assert!(db.exam_history().unwrap().is_empty(), "没交卷不该出现在趋势图里");
}

#[test]
fn 练习和复习不算进成绩趋势() {
    let db = seeded_db();
    for kind in [SessionKind::Practice, SessionKind::Review] {
        let sid = db.start_session(kind, 5, 0, None).unwrap();
        db.finish_session(sid, 100, 60, false).unwrap();
    }
    assert!(db.exam_history().unwrap().is_empty(), "趋势图只看模拟考");
}

#[test]
fn 错题复习到攻克的全过程() {
    let db = seeded_db();
    let qid = db.all_questions().unwrap()[0].id.clone();

    db.mark_wrong(&qid, vec!["B".into()], 0).unwrap();
    assert_eq!(db.due_entries(0).unwrap().len(), 1, "刚答错立刻到期");

    db.mark_review_right(&qid, 0).unwrap();
    assert!(db.due_entries(HOUR).unwrap().is_empty(), "答对后 3 小时内不再出现");
    assert_eq!(db.due_entries(3 * HOUR).unwrap().len(), 1, "到点重新出现");

    db.mark_review_right(&qid, 3 * HOUR).unwrap();
    db.mark_review_right(&qid, 20 * HOUR).unwrap();

    let e = db.wrong_entry(&qid).unwrap().unwrap();
    assert!(e.mastered, "4 级 + 连对 2 次 = 攻克");
    assert!(db.wrong_entries(false).unwrap().is_empty(), "攻克后离开待复习列表");
    assert_eq!(db.wrong_entries(true).unwrap().len(), 1, "但仍留在错题本里");

    // 再答错就打回原形
    db.mark_wrong(&qid, vec!["C".into()], 100 * HOUR).unwrap();
    let e = db.wrong_entry(&qid).unwrap().unwrap();
    assert_eq!(e.box_level, 1);
    assert!(!e.mastered);
    assert_eq!(e.wrong_count, 2);
}

#[test]
fn 不在错题本里的题复习答对不报错() {
    let db = seeded_db();
    assert!(db.mark_review_right("D1-001", 0).unwrap().is_none());
}

#[test]
fn 统计全部从作答明细聚合() {
    let db = seeded_db();
    let all = db.all_questions().unwrap();
    let sid = db.start_session(SessionKind::Practice, 20, 0, None).unwrap();

    let mut expect_right = 0;
    for (i, q) in all.iter().take(20).enumerate() {
        let right = i % 2 == 0;
        if right {
            expect_right += 1;
        }
        db.record_answer(&AnswerRecord {
            session_id: sid,
            question_id: q.id.clone(),
            seq: i as i64 + 1,
            picked: q.answer.clone(),
            correct: right,
            seconds: 25.0,
            answered_at: 0,
        })
        .unwrap();
    }
    db.finish_session(sid, 500, 500, false).unwrap();

    let ov = db.overview(0).unwrap();
    assert_eq!(ov.seen_count, 20);
    assert_eq!(ov.attempts, 20);
    assert_eq!(ov.rights, expect_right);
    assert_eq!(ov.avg_seconds, Some(25.0));

    let domains = db.domain_stats().unwrap();
    assert_eq!(domains.len(), 3, "三个领域都要有行，没练过的补 0");
    assert_eq!(domains.iter().map(|d| d.attempts).sum::<i64>(), 20);

    let seen = db.seen_question_ids().unwrap();
    assert_eq!(seen.len(), 20);
}

#[test]
fn 同一会话重复提交同一题只留一条() {
    let db = seeded_db();
    let q = db.all_questions().unwrap()[0].clone();
    let sid = db.start_session(SessionKind::Practice, 1, 0, None).unwrap();

    for (picked, correct) in [(vec!["B".to_string()], false), (q.answer.clone(), true)] {
        db.record_answer(&AnswerRecord {
            session_id: sid,
            question_id: q.id.clone(),
            seq: 1,
            picked,
            correct,
            seconds: 10.0,
            answered_at: 0,
        })
        .unwrap();
    }

    let answers = db.session_answers(sid).unwrap();
    assert_eq!(answers.len(), 1, "覆盖而不是追加");
    assert!(answers[0].correct, "保留的是最后一次");
    assert_eq!(db.finish_session(sid, 10, 1, false).unwrap().correct, 1);
}

#[test]
fn 准备度四项全满足才建议约考() {
    let db = seeded_db();
    let r = db.readiness(0).unwrap();
    assert!(!r.ready, "什么都没做时不该建议约考");
    assert!(r.next_step.contains("摸底"));

    // 有错题时优先提示清错题本
    db.mark_wrong(&db.all_questions().unwrap()[0].id.clone(), vec!["B".into()], 0)
        .unwrap();
    let r = db.readiness(0).unwrap();
    assert!(!r.wrongbook_clear);
    assert!(r.next_step.contains("错题本"));
}

#[test]
fn 掌握度分布总是五档() {
    let db = seeded_db();
    let dist = db.box_distribution().unwrap();
    assert_eq!(dist.len(), 5);
    assert!(dist.iter().all(|b| b.count == 0));
    assert_eq!(dist.iter().map(|b| b.box_level).collect::<Vec<_>>(), vec![1, 2, 3, 4, 5]);
}

#[test]
fn 清空错题本() {
    let db = seeded_db();
    let all = db.all_questions().unwrap();
    for q in all.iter().take(3) {
        db.mark_wrong(&q.id, vec!["B".into()], 0).unwrap();
    }
    assert_eq!(db.wrong_entries(true).unwrap().len(), 3);
    db.clear_wrongbook().unwrap();
    assert!(db.wrong_entries(true).unwrap().is_empty());
}

#[test]
fn 数据落盘后重开还在() {
    let dir = std::env::temp_dir().join(format!("az900-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.db");
    let _ = std::fs::remove_file(&path);

    let questions = az900_core::load_bank_dir(bank_dir()).unwrap();
    {
        let mut db = Db::open(&path).unwrap();
        db.import_questions(&questions).unwrap();
        let sid = db.start_session(SessionKind::Exam, 1, 0, None).unwrap();
        db.record_answer(&AnswerRecord {
            session_id: sid,
            question_id: questions[0].id.clone(),
            seq: 1,
            picked: questions[0].answer.clone(),
            correct: true,
            seconds: 12.0,
            answered_at: 0,
        })
        .unwrap();
        db.finish_session(sid, 100, 12, false).unwrap();
        db.mark_wrong(&questions[1].id, vec!["B".into()], 0).unwrap();
    }

    let db = Db::open(&path).unwrap();
    assert_eq!(db.exam_history().unwrap().len(), 1);
    assert_eq!(db.exam_history().unwrap()[0].score, 1000);
    assert_eq!(db.wrong_entries(false).unwrap().len(), 1);

    std::fs::remove_dir_all(&dir).ok();
}
