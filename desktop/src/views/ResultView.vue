<script setup lang="ts">
/** 成绩单：大分数 + 领域拆分 + 逐题讲评。 */
import { computed } from "vue";
import { useRouter } from "vue-router";
import { useResultStore } from "@/stores/result";
import { duration } from "@/format";
import type { DomainKey } from "@/types";
import DomainBars from "@/components/DomainBars.vue";
import QuestionCard from "@/components/QuestionCard.vue";

const router = useRouter();
const store = useResultStore();
const result = computed(() => store.result);

const DOMAIN_NO: Record<DomainKey, number> = {
  "cloud-concepts": 1,
  "architecture-services": 2,
  "management-governance": 3,
};

const PASS = 700;
const KIND_LABEL = { exam: "全真模拟考", practice: "专项练习", review: "错题复习" } as const;

const bars = computed(() =>
  (result.value?.domains ?? []).map((d) => ({
    number: d.number,
    nameCn: d.nameCn,
    rights: d.correct,
    attempts: d.total,
  })),
);
</script>

<template>
  <div v-if="!result" class="card empty">
    <p>没有可显示的成绩。</p>
    <button type="button" class="btn" style="margin-top: 12px" @click="router.push({ name: 'dashboard' })">
      回仪表盘
    </button>
  </div>
  <div v-else class="stack">
    <div class="card">
      <div class="eyebrow">{{ KIND_LABEL[result.session.kind] }} · 成绩单</div>
      <div class="score" :class="result.passed ? 'good' : 'crit'">{{ result.session.score }}</div>
      <p class="verdict" :class="result.passed ? 'good' : 'crit'">
        <span aria-hidden="true">{{ result.passed ? "✓" : "✕" }}</span>
        {{ result.passed ? `达标（及格线 ${PASS}）` : `未达标，及格线 ${PASS}` }}
      </p>
      <p class="lede meta">
        答对 {{ result.session.correct }}/{{ result.session.total }} 题 ·
        用时 {{ duration(result.session.elapsedSec) }}
        <template v-if="result.session.total > 0">
          · 平均每题 {{ Math.round(result.session.elapsedSec / result.session.total) }} 秒
        </template>
        <template v-if="result.session.timedOut">（时间到，未作答按错误计分）</template>
      </p>
    </div>

    <div class="card">
      <div class="card-head">
        <h4>本卷各领域得分</h4>
        <span class="sub">按官方权重组卷</span>
      </div>
      <DomainBars :rows="bars" />
    </div>

    <div v-if="result.wrongQuestions.length" class="card">
      <div class="card-head">
        <h4>逐题讲评 · {{ result.wrongQuestions.length }} 道错题</h4>
        <span class="sub">已收入错题本</span>
      </div>
      <div class="reviews">
        <QuestionCard
          v-for="w in result.wrongQuestions"
          :key="w.question.id"
          class="review"
          :question="w.question"
          :picked="w.picked"
          :revealed="true"
          :correct="false"
          :domain-number="DOMAIN_NO[w.question.domain]"
        />
      </div>
    </div>
    <div v-else class="card empty">全对，这套题没有留下错题。</div>

    <div class="row">
      <button
        type="button" class="btn btn-primary"
        @click="router.push({ name: 'quiz', query: { mode: 'review' } })"
      >
        去复习错题
      </button>
      <button
        type="button" class="btn"
        @click="router.push({ name: 'quiz', query: { mode: 'exam', count: result.session.total } })"
      >
        再来一套
      </button>
      <button type="button" class="btn" @click="router.push({ name: 'dashboard' })">看进度</button>
    </div>
  </div>
</template>

<style scoped>
.score { font-size: 48px; font-weight: 600; letter-spacing: -0.03em; line-height: 1.1; margin: 8px 0 2px; }
.score.good, .verdict.good { color: var(--st-good); }
.score.crit, .verdict.crit { color: var(--st-crit); }
.verdict { font-weight: 600; font-size: 14.5px; display: flex; gap: 7px; align-items: center; }
.meta { margin-top: 9px; }
.reviews { display: flex; flex-direction: column; gap: 14px; }
.review { border-color: var(--line); }
</style>
