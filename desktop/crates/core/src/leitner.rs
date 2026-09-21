//! 错题本的间隔重复（Leitner 盒子）。
//!
//! 每道错题有 1-5 级掌握度：答对升一级并推迟下次出现，答错立刻打回 1 级。
//! 升到 4 级且连续答对 2 次才算攻克，从待复习列表移除。
use crate::model::WrongEntry;

/// 各级对应的复习间隔（小时）。1 级表示立刻要复习。
pub const BOX_INTERVAL_HOURS: [i64; 6] = [0, 0, 3, 12, 24, 48];
pub const MAX_BOX: i64 = 5;
/// 判定攻克的门槛。
pub const MASTERY_BOX: i64 = 4;
pub const MASTERY_STREAK: i64 = 2;

fn interval_ms(box_level: i64) -> i64 {
    let idx = box_level.clamp(1, MAX_BOX) as usize;
    BOX_INTERVAL_HOURS[idx] * 3_600_000
}

/// 新建一条错题记录。
pub fn new_entry(question_id: &str, picked: Vec<String>, now: i64) -> WrongEntry {
    WrongEntry {
        question_id: question_id.to_string(),
        wrong_count: 1,
        right_streak: 0,
        box_level: 1,
        mastered: false,
        first_wrong_at: now,
        last_wrong_at: Some(now),
        last_picked: picked,
        due_at: now,
    }
}

/// 又答错了一次（不论是初次收录还是复习时答错）。
pub fn on_wrong(entry: &mut WrongEntry, picked: Vec<String>, now: i64) {
    entry.wrong_count += 1;
    entry.right_streak = 0;
    entry.box_level = 1;
    entry.mastered = false;
    entry.last_wrong_at = Some(now);
    entry.last_picked = picked;
    entry.due_at = now;
}

/// 复习时答对：升一级、推迟下次、够格就标记攻克。
pub fn on_right(entry: &mut WrongEntry, now: i64) {
    entry.right_streak += 1;
    entry.box_level = (entry.box_level + 1).min(MAX_BOX);
    entry.due_at = now + interval_ms(entry.box_level);
    if entry.box_level >= MASTERY_BOX && entry.right_streak >= MASTERY_STREAK {
        entry.mastered = true;
    }
}

/// 到期该复习了吗？已攻克的不再出现。
pub fn is_due(entry: &WrongEntry, now: i64) -> bool {
    !entry.mastered && entry.due_at <= now
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: i64 = 3_600_000;

    fn entry() -> WrongEntry {
        new_entry("D1-001", vec!["A".into()], 0)
    }

    #[test]
    fn 新错题立刻到期且在一级() {
        let e = entry();
        assert_eq!(e.box_level, 1);
        assert_eq!(e.wrong_count, 1);
        assert!(!e.mastered);
        assert!(is_due(&e, 0));
    }

    #[test]
    fn 连续答对四次才攻克() {
        let mut e = entry();
        // 1→2 级：推迟 3 小时，此时 streak=1，还不够攻克
        on_right(&mut e, 0);
        assert_eq!(e.box_level, 2);
        assert_eq!(e.due_at, 3 * HOUR);
        assert!(!e.mastered);

        on_right(&mut e, 3 * HOUR); // →3 级
        assert_eq!(e.box_level, 3);
        assert!(!e.mastered, "3 级即使连对 2 次也不算攻克");

        on_right(&mut e, 20 * HOUR); // →4 级，streak=3
        assert_eq!(e.box_level, 4);
        assert!(e.mastered, "到 4 级且连对 ≥2 次即攻克");
    }

    #[test]
    fn 攻克后再答错会打回一级() {
        let mut e = entry();
        for t in [0, 3 * HOUR, 20 * HOUR] {
            on_right(&mut e, t);
        }
        assert!(e.mastered);

        on_wrong(&mut e, vec!["C".into()], 50 * HOUR);
        assert_eq!(e.box_level, 1);
        assert_eq!(e.right_streak, 0);
        assert!(!e.mastered);
        assert_eq!(e.wrong_count, 2);
        assert_eq!(e.last_picked, vec!["C".to_string()]);
        assert!(is_due(&e, 50 * HOUR), "答错后立刻重新到期");
    }

    #[test]
    fn 答对后未到间隔不会重复出现() {
        let mut e = entry();
        on_right(&mut e, 0);
        assert!(!is_due(&e, HOUR), "3 小时间隔内不该再出现");
        assert!(is_due(&e, 3 * HOUR), "到点就该出现");
    }

    #[test]
    fn 掌握度不会超过五级() {
        let mut e = entry();
        for i in 0..10 {
            on_right(&mut e, i * 100 * HOUR);
        }
        assert_eq!(e.box_level, MAX_BOX);
        assert_eq!(e.due_at, 900 * HOUR + 48 * HOUR);
    }

    #[test]
    fn 已攻克的题不再到期() {
        let mut e = entry();
        for t in [0, 3 * HOUR, 20 * HOUR] {
            on_right(&mut e, t);
        }
        assert!(e.mastered);
        assert!(!is_due(&e, i64::MAX / 2), "攻克后无论多久都不再出现");
    }
}
