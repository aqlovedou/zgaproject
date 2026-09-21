//! 组卷：按官方领域权重抽题，模拟真实考卷的领域分布。
use crate::model::{Domain, Question};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;

/// 按领域权重抽 `count` 道题。
///
/// 先按权重给每个领域分配名额，取整造成的缺口用剩余题目补齐，
/// 最后整体打乱，保证同一领域的题不会扎堆出现。
pub fn build_exam<R: Rng + ?Sized>(
    pool: &[Question],
    count: usize,
    rng: &mut R,
    exclude: &HashSet<String>,
) -> Vec<Question> {
    let count = count.min(pool.len());
    let mut picked: Vec<Question> = Vec::with_capacity(count);
    let mut chosen: HashSet<String> = HashSet::new();

    for domain in Domain::ALL {
        let mut bucket: Vec<&Question> = pool
            .iter()
            .filter(|q| q.domain == domain && !exclude.contains(&q.id))
            .collect();
        bucket.shuffle(rng);
        let want = (count as f64 * domain.weight()).round() as usize;
        for q in bucket.into_iter().take(want) {
            chosen.insert(q.id.clone());
            picked.push(q.clone());
        }
    }

    // 权重取整后可能少几道，用没抽中的题补满
    if picked.len() < count {
        let mut rest: Vec<&Question> = pool
            .iter()
            .filter(|q| !chosen.contains(&q.id) && !exclude.contains(&q.id))
            .collect();
        rest.shuffle(rng);
        for q in rest.into_iter().take(count - picked.len()) {
            picked.push(q.clone());
        }
    }

    picked.truncate(count);
    picked.shuffle(rng);
    picked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Question, QuestionType};
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::BTreeMap;

    fn q(id: &str, domain: Domain) -> Question {
        let mut options = BTreeMap::new();
        options.insert("A".to_string(), "甲".to_string());
        options.insert("B".to_string(), "乙".to_string());
        Question {
            id: id.to_string(),
            domain,
            topic: "测试考点".into(),
            question_type: QuestionType::Single,
            question: "题干".into(),
            options,
            answer: vec!["A".into()],
            explanation: "解析".into(),
            option_reasons: BTreeMap::new(),
            reference_title: "出处".into(),
            reference_url: "https://learn.microsoft.com/azure/".into(),
        }
    }

    fn pool() -> Vec<Question> {
        let mut v = Vec::new();
        for i in 0..60 {
            v.push(q(&format!("D1-{i:03}"), Domain::CloudConcepts));
        }
        for i in 0..80 {
            v.push(q(&format!("D2-{i:03}"), Domain::ArchitectureServices));
        }
        for i in 0..70 {
            v.push(q(&format!("D3-{i:03}"), Domain::ManagementGovernance));
        }
        v
    }

    #[test]
    fn 抽题数量准确() {
        let mut rng = StdRng::seed_from_u64(7);
        let exam = build_exam(&pool(), 50, &mut rng, &HashSet::new());
        assert_eq!(exam.len(), 50);
    }

    #[test]
    fn 领域分布贴合官方权重() {
        let mut rng = StdRng::seed_from_u64(7);
        let exam = build_exam(&pool(), 50, &mut rng, &HashSet::new());
        for domain in Domain::ALL {
            let got = exam.iter().filter(|q| q.domain == domain).count() as f64;
            let want = 50.0 * domain.weight();
            assert!(
                (got - want).abs() <= 2.0,
                "{:?} 抽了 {got} 道，期望约 {want} 道",
                domain
            );
        }
    }

    #[test]
    fn 同一份卷子不出现重复题() {
        let mut rng = StdRng::seed_from_u64(11);
        let exam = build_exam(&pool(), 50, &mut rng, &HashSet::new());
        let uniq: HashSet<&String> = exam.iter().map(|q| &q.id).collect();
        assert_eq!(uniq.len(), exam.len());
    }

    #[test]
    fn 排除的题不会被抽中() {
        let mut rng = StdRng::seed_from_u64(3);
        let excluded: HashSet<String> =
            (0..60).map(|i| format!("D1-{i:03}")).collect();
        let exam = build_exam(&pool(), 40, &mut rng, &excluded);
        assert!(exam.iter().all(|q| !excluded.contains(&q.id)));
        assert_eq!(exam.len(), 40, "排除后仍能凑满题数");
    }

    #[test]
    fn 要的题比库里多时返回全部() {
        let small = vec![q("D1-001", Domain::CloudConcepts)];
        let mut rng = StdRng::seed_from_u64(1);
        let exam = build_exam(&small, 50, &mut rng, &HashSet::new());
        assert_eq!(exam.len(), 1);
    }

    #[test]
    fn 同一份卷子不会按领域扎堆() {
        let mut rng = StdRng::seed_from_u64(99);
        let exam = build_exam(&pool(), 50, &mut rng, &HashSet::new());
        // 若未打乱，前 14 题会全是领域1
        let first14_all_d1 = exam
            .iter()
            .take(14)
            .all(|q| q.domain == Domain::CloudConcepts);
        assert!(!first14_all_d1, "组卷后应打乱顺序");
    }
}
