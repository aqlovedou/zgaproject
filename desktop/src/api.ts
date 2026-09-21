/** 带类型的 Tauri 命令封装。前端只通过这里和 Rust 说话。 */
import { invoke } from "@tauri-apps/api/core";
import type {
  AnswerOutcome,
  Dashboard,
  Paper,
  PracticeFilter,
  Question,
  SessionKind,
  SessionResult,
  TopicRow,
  WrongRow,
} from "./types";

export const api = {
  bankSize: () => invoke<number>("get_bank_size"),
  topics: () => invoke<TopicRow[]>("get_topics"),
  question: (id: string) => invoke<Question | null>("get_question", { id }),

  startExam: (count: number) => invoke<Paper>("start_exam", { count }),
  startPractice: (filter: PracticeFilter) =>
    invoke<Paper>("start_practice", { filter }),
  startReview: () => invoke<Paper>("start_review"),

  submitAnswer: (args: {
    sessionId: number;
    questionId: string;
    seq: number;
    picked: string[];
    seconds: number;
    kind: SessionKind;
  }) => invoke<AnswerOutcome>("submit_answer", { args }),

  finishSession: (args: {
    sessionId: number;
    elapsedSec: number;
    timedOut: boolean;
  }) => invoke<SessionResult>("finish_session", { args }),

  wrongbook: (includeMastered: boolean) =>
    invoke<WrongRow[]>("get_wrongbook", { includeMastered }),
  clearWrongbook: () => invoke<number>("clear_wrongbook"),

  dashboard: () => invoke<Dashboard>("get_dashboard"),
  dbPath: () => invoke<string>("get_db_path"),
};
