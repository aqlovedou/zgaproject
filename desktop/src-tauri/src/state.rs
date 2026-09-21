//! 应用状态：一个持有 SQLite 连接的互斥锁，外加题库缓存。
use anyhow::Result;
use az900_core::{model::Question, Db};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AppState {
    pub db: Mutex<Db>,
    /// 题库全量缓存。179 道题常驻内存约几百 KB，省掉每次组卷的查询。
    pub bank: Vec<Question>,
    pub db_path: PathBuf,
}

/// 当前时间（epoch 毫秒）。时间只从后端取，前端不参与，避免时区和时钟漂移。
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl AppState {
    /// 打开（或新建）数据库，并把题库 JSON 导入 question 表。
    pub fn init(db_path: PathBuf, bank_json: &[(&str, &str)]) -> Result<Self> {
        let mut db = Db::open(&db_path)?;

        let mut bank = Vec::new();
        for (name, text) in bank_json {
            let parsed = az900_core::parse_bank_file(text)
                .map_err(|e| anyhow::anyhow!("题库 {name} 解析失败：{e}"))?;
            bank.extend(parsed);
        }
        bank.sort_by(|a, b| a.id.cmp(&b.id));
        db.import_questions(&bank)?;

        Ok(AppState { db: Mutex::new(db), bank, db_path })
    }
}
