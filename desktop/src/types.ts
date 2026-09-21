/**
 * 与 Rust 端 serde 输出一一对应的类型。
 * Rust 那边用 #[serde(rename_all = "camelCase")]，所以这里全是小驼峰。
 * 改 Rust 结构体后这里也要改，否则 vue-tsc 会在用到的地方报错。
 */

export type DomainKey =
  | "cloud-concepts"
  | "architecture-services"
  | "management-governance";

export type QuestionType = "single" | "multi" | "truefalse";
export type SessionKind = "exam" | "practice" | "review";

export interface Question {
  id: string;
  domain: DomainKey;
  topic: string;
  type: QuestionType;
  question: string;
  options: Record<string, string>;
  answer: string[];
  explanation: string;
  optionReasons: Record<string, string>;
  referenceTitle: string;
  referenceUrl: string;
}

export interface Paper {
  sessionId: number;
  kind: SessionKind;
  questions: Question[];
  timeLimitSec: number | null;
  label: string;
}

export interface PracticeFilter {
  domain?: DomainKey;
  topic?: string;
  topicLike?: string;
  keyword?: string;
  onlyUnseen?: boolean;
  limit?: number;
}

export interface WrongEntry {
  questionId: string;
  wrongCount: number;
  rightStreak: number;
  boxLevel: number;
  mastered: boolean;
  firstWrongAt: number;
  lastWrongAt: number | null;
  lastPicked: string[];
  dueAt: number;
}

export interface AnswerOutcome {
  correct: boolean;
  answer: string[];
  wrongEntry: WrongEntry | null;
}

export interface Session {
  id: number;
  kind: SessionKind;
  startedAt: number;
  finishedAt: number | null;
  total: number;
  correct: number;
  score: number;
  elapsedSec: number;
  timeLimitSec: number | null;
  timedOut: boolean;
}

export interface DomainResult {
  domain: DomainKey;
  number: number;
  nameCn: string;
  correct: number;
  total: number;
}

export interface WrongReview {
  question: Question;
  picked: string[];
}

export interface SessionResult {
  session: Session;
  domains: DomainResult[];
  wrongQuestions: WrongReview[];
  passed: boolean;
}

export interface WrongRow {
  entry: WrongEntry;
  question: Question;
  dueNow: boolean;
}

export interface Overview {
  bankSize: number;
  seenCount: number;
  attempts: number;
  rights: number;
  wrongPending: number;
  wrongMastered: number;
  wrongDueNow: number;
  lastExamScore: number | null;
  bestExamScore: number | null;
  avgSeconds: number | null;
}

export interface DomainStat {
  domain: DomainKey;
  number: number;
  nameCn: string;
  officialRange: string;
  attempts: number;
  rights: number;
}

export interface TopicStat {
  topic: string;
  domain: DomainKey;
  attempts: number;
  rights: number;
  bankCount: number;
}

export interface ExamPoint {
  sessionId: number;
  at: number;
  score: number;
  correct: number;
  total: number;
}

export interface BoxBucket {
  boxLevel: number;
  count: number;
}

export interface Readiness {
  twoExamsOver800: boolean;
  allDomainsOver80: boolean;
  wrongbookClear: boolean;
  coverageOver90: boolean;
  ready: boolean;
  nextStep: string;
}

export interface Dashboard {
  overview: Overview;
  domains: DomainStat[];
  exams: ExamPoint[];
  boxes: BoxBucket[];
  weakTopics: TopicStat[];
  readiness: Readiness;
  passScore: number;
  maxScore: number;
}

export interface TopicRow {
  topic: string;
  domain: DomainKey;
  number: number;
  nameCn: string;
  officialRange: string;
  count: number;
}

export const TYPE_LABEL: Record<QuestionType, string> = {
  single: "单选题",
  multi: "多选题",
  truefalse: "判断题",
};
