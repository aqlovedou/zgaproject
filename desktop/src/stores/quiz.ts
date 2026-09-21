/**
 * 当前这套题的状态：题目、已答、选中项、计时器。
 *
 * 计时器只存一个「开始时刻」，剩余时间由 tick 触发重算 —— 不累加计数，
 * 这样窗口挂起或休眠唤醒后时间依然准确。
 */
import { defineStore } from "pinia";
import { api } from "@/api";
import type { AnswerOutcome, Paper, Question, SessionKind } from "@/types";
import { sameSet } from "@/format";

interface AnswerState {
  picked: string[];
  correct: boolean;
}

interface State {
  paper: Paper | null;
  index: number;
  picked: string[];
  submitted: boolean;
  lastOutcome: AnswerOutcome | null;
  answers: Record<string, AnswerState>;
  startedAt: number;
  questionStartedAt: number;
  nowTick: number;
  busy: boolean;
  error: string | null;
}

export const useQuizStore = defineStore("quiz", {
  state: (): State => ({
    paper: null,
    index: 0,
    picked: [],
    submitted: false,
    lastOutcome: null,
    answers: {},
    startedAt: 0,
    questionStartedAt: 0,
    nowTick: Date.now(),
    busy: false,
    error: null,
  }),

  getters: {
    active: (s): boolean => s.paper !== null,
    total: (s): number => s.paper?.questions.length ?? 0,
    kind: (s): SessionKind => s.paper?.kind ?? "practice",
    current(s): Question | null {
      return s.paper?.questions[s.index] ?? null;
    },
    /** 模拟考交卷后才讲评；练习和复习答一题讲一题。 */
    instantFeedback: (s): boolean => s.paper?.kind !== "exam",
    answeredCount: (s): number => Object.keys(s.answers).length,
    isLast(s): boolean {
      return s.index + 1 >= (s.paper?.questions.length ?? 0);
    },
    /** 限时模式下剩余秒数；不限时返回 null。 */
    secondsLeft(s): number | null {
      if (!s.paper?.timeLimitSec) return null;
      const elapsed = (s.nowTick - s.startedAt) / 1000;
      return Math.max(0, Math.round(s.paper.timeLimitSec - elapsed));
    },
    elapsedSec(s): number {
      return Math.round((s.nowTick - s.startedAt) / 1000);
    },
  },

  actions: {
    _load(paper: Paper) {
      const now = Date.now();
      this.paper = paper;
      this.index = 0;
      this.picked = [];
      this.submitted = false;
      this.lastOutcome = null;
      this.answers = {};
      this.startedAt = now;
      this.questionStartedAt = now;
      this.nowTick = now;
      this.error = null;
    },

    async startExam(count: number) {
      await this._start(() => api.startExam(count));
    },
    async startPractice(filter: Parameters<typeof api.startPractice>[0]) {
      await this._start(() => api.startPractice(filter));
    },
    async startReview() {
      await this._start(() => api.startReview());
    },

    async _start(fn: () => Promise<Paper>) {
      this.busy = true;
      this.error = null;
      try {
        this._load(await fn());
      } catch (e) {
        this.error = String(e);
        throw e;
      } finally {
        this.busy = false;
      }
    },

    /** 点选项：单选替换，多选切换。 */
    toggle(key: string) {
      if (this.submitted || !this.current) return;
      if (this.current.type === "multi") {
        const i = this.picked.indexOf(key);
        if (i >= 0) this.picked.splice(i, 1);
        else this.picked.push(key);
      } else {
        this.picked = [key];
      }
    },

    /** 提交当前题。判分以后端返回为准。 */
    async submit(): Promise<AnswerOutcome | null> {
      const q = this.current;
      if (!q || !this.paper || this.submitted || this.picked.length === 0) return null;

      this.busy = true;
      try {
        const outcome = await api.submitAnswer({
          sessionId: this.paper.sessionId,
          questionId: q.id,
          seq: this.index + 1,
          picked: [...this.picked],
          seconds: (Date.now() - this.questionStartedAt) / 1000,
          kind: this.paper.kind,
        });
        this.answers[q.id] = { picked: [...this.picked], correct: outcome.correct };
        this.lastOutcome = outcome;
        if (this.instantFeedback) this.submitted = true;
        return outcome;
      } catch (e) {
        this.error = String(e);
        return null;
      } finally {
        this.busy = false;
      }
    },

    /** 跳过：记为未作答（按错误计分），并进错题本。 */
    async skip() {
      const q = this.current;
      if (!q || !this.paper || this.answers[q.id]) return;
      this.picked = [];
      try {
        const outcome = await api.submitAnswer({
          sessionId: this.paper.sessionId,
          questionId: q.id,
          seq: this.index + 1,
          picked: [],
          seconds: (Date.now() - this.questionStartedAt) / 1000,
          kind: this.paper.kind,
        });
        this.answers[q.id] = { picked: [], correct: outcome.correct };
      } catch (e) {
        this.error = String(e);
      }
    },

    next() {
      this.submitted = false;
      this.picked = [];
      this.lastOutcome = null;
      this.questionStartedAt = Date.now();
      if (!this.isLast) this.index += 1;
    },

    /** 交卷。分数由后端从作答明细重算。 */
    async finish(timedOut = false) {
      if (!this.paper) return null;
      this.busy = true;
      try {
        return await api.finishSession({
          sessionId: this.paper.sessionId,
          elapsedSec: this.elapsedSec,
          timedOut,
        });
      } finally {
        this.busy = false;
        this.reset();
      }
    },

    tick() {
      this.nowTick = Date.now();
    },

    reset() {
      this.paper = null;
      this.index = 0;
      this.picked = [];
      this.submitted = false;
      this.lastOutcome = null;
      this.answers = {};
    },

    /** 本地判分，仅用于讲评时高亮；权威判分在后端。 */
    localCorrect(q: Question, picked: string[]): boolean {
      return sameSet(picked, q.answer);
    },
  },
});
