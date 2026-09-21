-- AZ-900 刷题台的库结构。
-- 设计原则：统计不另存缓存表，成绩趋势/领域正确率/覆盖率全部从 answer 实时聚合，
-- 这样永远不会出现「统计数字和明细对不上」。

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- 题库：构建时从 data/questions/*.json 导入，运行期只读
CREATE TABLE IF NOT EXISTS question (
  id             TEXT PRIMARY KEY,
  domain         TEXT NOT NULL,
  topic          TEXT NOT NULL,
  type           TEXT NOT NULL CHECK (type IN ('single','multi','truefalse')),
  question       TEXT NOT NULL,
  options        TEXT NOT NULL,   -- JSON {"A":"…"}
  answer         TEXT NOT NULL,   -- JSON ["A","C"]
  explanation    TEXT NOT NULL,
  option_reasons TEXT NOT NULL,   -- JSON {"B":"为什么错"}
  ref_title      TEXT NOT NULL,
  ref_url        TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_question_domain ON question(domain);
CREATE INDEX IF NOT EXISTS idx_question_topic  ON question(topic);

-- 每次考试／练习／复习的汇总
CREATE TABLE IF NOT EXISTS session (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  kind           TEXT    NOT NULL CHECK (kind IN ('exam','practice','review')),
  started_at     INTEGER NOT NULL,
  finished_at    INTEGER,
  total          INTEGER NOT NULL,
  correct        INTEGER NOT NULL DEFAULT 0,
  score          INTEGER NOT NULL DEFAULT 0,
  elapsed_sec    INTEGER NOT NULL DEFAULT 0,
  time_limit_sec INTEGER,
  timed_out      INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_session_kind ON session(kind, started_at DESC);

-- 每道题的作答明细：所有统计的唯一数据源
CREATE TABLE IF NOT EXISTS answer (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id  INTEGER NOT NULL REFERENCES session(id) ON DELETE CASCADE,
  question_id TEXT    NOT NULL REFERENCES question(id),
  seq         INTEGER NOT NULL,
  picked      TEXT    NOT NULL,   -- JSON ["A"]
  correct     INTEGER NOT NULL,
  seconds     REAL    NOT NULL,
  answered_at INTEGER NOT NULL,
  UNIQUE(session_id, question_id)
);
CREATE INDEX IF NOT EXISTS idx_answer_question ON answer(question_id);
CREATE INDEX IF NOT EXISTS idx_answer_session  ON answer(session_id);

-- 错题本：一道错题一行，box 是 Leitner 掌握度 1-5
CREATE TABLE IF NOT EXISTS wrong_entry (
  question_id    TEXT    PRIMARY KEY REFERENCES question(id),
  wrong_count    INTEGER NOT NULL DEFAULT 0,
  right_streak   INTEGER NOT NULL DEFAULT 0,
  box            INTEGER NOT NULL DEFAULT 1 CHECK (box BETWEEN 1 AND 5),
  mastered       INTEGER NOT NULL DEFAULT 0,
  first_wrong_at INTEGER NOT NULL,
  last_wrong_at  INTEGER,
  last_picked    TEXT    NOT NULL DEFAULT '[]',
  due_at         INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_wrong_due ON wrong_entry(mastered, due_at);
