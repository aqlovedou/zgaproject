<script setup lang="ts">
/**
 * 答题页。模拟考交卷后统一讲评，练习和复习答一题讲一题。
 * 计时器用「开始时刻 + 定时重算」而不是累加，休眠唤醒后依然准确。
 */
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useQuizStore } from "@/stores/quiz";
import { useResultStore } from "@/stores/result";
import { clock } from "@/format";
import type { DomainKey } from "@/types";
import QuestionCard from "@/components/QuestionCard.vue";

const route = useRoute();
const router = useRouter();
const quiz = useQuizStore();
const resultStore = useResultStore();

const DOMAIN_NO: Record<DomainKey, number> = {
  "cloud-concepts": 1,
  "architecture-services": 2,
  "management-governance": 3,
};

const starting = ref(true);
const startError = ref<string | null>(null);
let timer: number | undefined;

onMounted(async () => {
  const q = route.query;
  try {
    if (q.mode === "exam") {
      await quiz.startExam(Number(q.count) || 50);
    } else if (q.mode === "review") {
      await quiz.startReview();
    } else {
      await quiz.startPractice({
        domain: (q.domain as DomainKey) || undefined,
        topic: (q.topic as string) || undefined,
        topicLike: (q.topicLike as string) || undefined,
        keyword: (q.keyword as string) || undefined,
        onlyUnseen: q.onlyUnseen === "1",
        limit: q.count ? Number(q.count) : undefined,
      });
    }
  } catch (e) {
    startError.value = String(e);
    return;
  } finally {
    starting.value = false;
  }

  timer = window.setInterval(() => {
    quiz.tick();
    if (quiz.secondsLeft === 0) void finish(true);
  }, 500);

  window.addEventListener("keydown", onKey);
});

onBeforeUnmount(() => {
  if (timer) window.clearInterval(timer);
  window.removeEventListener("keydown", onKey);
});

const timeLabel = computed(() =>
  quiz.secondsLeft === null ? null : clock(quiz.secondsLeft),
);
const lowTime = computed(() => quiz.secondsLeft !== null && quiz.secondsLeft < 300);

async function onSubmit() {
  const outcome = await quiz.submit();
  if (!outcome) return;
  if (!quiz.instantFeedback) await advance();
}

async function advance() {
  if (quiz.isLast) {
    await finish(false);
  } else {
    quiz.next();
    window.scrollTo({ top: 0 });
  }
}

async function onSkip() {
  await quiz.skip();
  await advance();
}

async function finish(timedOut: boolean) {
  if (timer) window.clearInterval(timer);
  const result = await quiz.finish(timedOut);
  if (result) {
    resultStore.set(result);
    router.replace({ name: "result" });
  } else {
    router.replace({ name: "dashboard" });
  }
}

function onKey(ev: KeyboardEvent) {
  const q = quiz.current;
  if (!q) return;
  const target = ev.target as HTMLElement | null;
  if (target && /input|textarea|select/i.test(target.tagName)) return;

  const key = ev.key.toUpperCase();
  if (!quiz.submitted && Object.prototype.hasOwnProperty.call(q.options, key)) {
    ev.preventDefault();
    quiz.toggle(key);
    return;
  }
  if (ev.key === "Enter") {
    ev.preventDefault();
    if (quiz.submitted) void advance();
    else void onSubmit();
  }
}

async function quitEarly() {
  if (!confirm("提前交卷？未作答的题按错误计分。")) return;
  await finish(false);
}
</script>

<template>
  <div v-if="starting" class="card empty">正在组卷…</div>
  <div v-else-if="startError" class="card empty">
    <p>{{ startError }}</p>
    <button type="button" class="btn" style="margin-top: 12px" @click="router.push({ name: 'dashboard' })">
      回仪表盘
    </button>
  </div>
  <div v-else-if="quiz.current" class="stack">
    <div class="runbar">
      <div class="dots">
        <span
          v-for="(q, i) in quiz.paper!.questions"
          :key="q.id"
          class="dot"
          :class="{ done: !!quiz.answers[q.id], now: i === quiz.index }"
        />
      </div>
      <span class="counter mono">{{ quiz.index + 1 }} / {{ quiz.total }}</span>
      <span v-if="timeLabel" class="counter mono" :class="{ low: lowTime }">{{ timeLabel }}</span>
    </div>

    <QuestionCard
      :question="quiz.current"
      :picked="quiz.picked"
      :revealed="quiz.submitted"
      :correct="quiz.lastOutcome?.correct"
      :domain-number="DOMAIN_NO[quiz.current.domain]"
      :hint="quiz.kind === 'exam' ? '考试中不显示答案' : undefined"
      :disabled="quiz.busy"
      @toggle="quiz.toggle"
    >
      <template #actions>
        <div class="actions">
          <template v-if="quiz.submitted">
            <button type="button" class="btn btn-primary" @click="advance">
              {{ quiz.isLast ? "交卷看成绩" : "下一题" }}
            </button>
          </template>
          <template v-else>
            <button
              type="button" class="btn btn-primary"
              :disabled="quiz.picked.length === 0 || quiz.busy"
              @click="onSubmit"
            >
              {{ quiz.kind === "exam" ? "确定并进入下一题" : "提交答案" }}
            </button>
            <button type="button" class="btn" :disabled="quiz.busy" @click="onSkip">跳过</button>
            <button v-if="quiz.kind === 'exam'" type="button" class="btn" @click="quitEarly">
              提前交卷
            </button>
          </template>
          <span class="hint mono">键盘：{{ Object.keys(quiz.current.options).sort().join("/") }} 选项　Enter 继续</span>
        </div>
      </template>
    </QuestionCard>

    <p v-if="quiz.error" class="lede err">{{ quiz.error }}</p>
  </div>
  <div v-else class="card empty">正在交卷…</div>
</template>

<style scoped>
.runbar {
  position: sticky; top: 0; z-index: 10;
  background: var(--surface); border: 1px solid var(--line); border-radius: var(--r);
  padding: 9px 13px; display: flex; align-items: center; gap: 12px; flex-wrap: wrap;
}
.dots { display: flex; gap: 3px; flex-wrap: wrap; flex: 1; min-width: 120px; }
.dot { width: 7px; height: 7px; border-radius: 2px; background: var(--line-strong); }
.dot.done { background: var(--accent); }
.dot.now { background: var(--ink); outline: 2px solid var(--accent-soft); }
.counter { font-size: 14px; font-weight: 600; }
.counter.low { color: var(--st-crit); }
.actions { display: flex; gap: 10px; flex-wrap: wrap; align-items: center; margin-top: 18px; }
.hint { font-size: 11.5px; color: var(--faint); }
.err { color: var(--st-crit); }
</style>
