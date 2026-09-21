<script setup lang="ts">
/** 错题本：掌握度分布 + 待复习清单（可展开看解析和出处）。 */
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { api } from "@/api";
import type { BoxBucket, WrongRow } from "@/types";
import { boxDots, dueLabel } from "@/format";
import BoxDistribution from "@/components/BoxDistribution.vue";
import FeedbackPanel from "@/components/FeedbackPanel.vue";

const router = useRouter();
const rows = ref<WrongRow[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const expanded = ref<Set<string>>(new Set());


const pending = computed(() => rows.value.filter((r) => !r.entry.mastered));
const mastered = computed(() => rows.value.filter((r) => r.entry.mastered));
const dueCount = computed(() => rows.value.filter((r) => r.dueNow).length);

const buckets = computed<BoxBucket[]>(() =>
  [1, 2, 3, 4, 5].map((level) => ({
    boxLevel: level,
    count: rows.value.filter((r) => r.entry.boxLevel === level).length,
  })),
);

/** 最薄弱的考点：按累计错误次数汇总。 */
const weakTopics = computed(() => {
  const agg = new Map<string, number>();
  for (const r of pending.value) {
    agg.set(r.question.topic, (agg.get(r.question.topic) ?? 0) + r.entry.wrongCount);
  }
  return [...agg.entries()].sort((a, b) => b[1] - a[1]).slice(0, 6);
});

async function load() {
  loading.value = true;
  try {
    rows.value = await api.wrongbook(true);
    error.value = null;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}
onMounted(load);

function toggle(id: string) {
  const next = new Set(expanded.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  expanded.value = next;
}

async function clearAll() {
  if (!confirm("确定清空错题本？这会删掉所有错题记录和掌握度。")) return;
  await api.clearWrongbook();
  await load();
}
</script>

<template>
  <div v-if="loading" class="card empty">载入中…</div>
  <div v-else-if="error" class="card empty">读取失败：{{ error }}</div>
  <div v-else class="stack">
    <div class="card">
      <div class="card-head">
        <h4>
          错题本 ·
          {{ rows.length ? `${pending.length} 题待攻克 · ${mastered.length} 题已攻克` : "还没有错题" }}
        </h4>
        <span v-if="rows.length" class="sub">今天到期 {{ dueCount }} 题</span>
      </div>
      <BoxDistribution v-if="rows.length" :buckets="buckets" />
      <p class="lede rule">
        答对一次掌握度升一级并推迟（3 小时 → 12 小时 → 1 天 → 2 天），答错立刻打回 1 级；
        掌握度到 4 级且连续答对 2 次才算攻克。
      </p>
      <div class="row acts">
        <button
          v-if="pending.length" type="button" class="btn btn-primary"
          @click="router.push({ name: 'quiz', query: { mode: 'review' } })"
        >
          {{ dueCount ? `复习到期的 ${dueCount} 题` : `复习全部 ${pending.length} 题` }}
        </button>
        <button v-if="rows.length" type="button" class="btn" @click="clearAll">清空错题本</button>
      </div>
    </div>

    <div v-if="!rows.length" class="card empty">
      做一套模拟考，错的题会自动收录到这里，并标出你为什么错。
    </div>

    <div v-if="weakTopics.length" class="card">
      <div class="card-head"><h4>最薄弱的考点</h4></div>
      <div class="row">
        <button
          v-for="[topic, n] in weakTopics" :key="topic" type="button" class="btn btn-sm"
          @click="router.push({ name: 'quiz', query: { mode: 'practice', topic } })"
        >
          {{ topic }} · 错 {{ n }} 次
        </button>
      </div>
    </div>

    <div v-if="rows.length" class="list">
      <div v-for="r in rows" :key="r.question.id" class="entry">
        <div class="head">
          <span class="chip mono">{{ r.question.id }}</span>
          <span class="chip chip-accent">{{ r.question.topic }}</span>
          <span class="chip" :style="{ color: r.entry.mastered ? 'var(--st-good)' : 'var(--st-crit)' }">
            {{ r.entry.mastered ? "已攻克" : `错 ${r.entry.wrongCount} 次` }}
          </span>
          <span class="dots mono">{{ boxDots(r.entry.boxLevel) }}</span>
          <span class="chip" :style="{ color: r.dueNow ? 'var(--st-warn)' : undefined }">
            {{ r.entry.mastered ? "不再出现" : dueLabel(r.entry.dueAt) }}
          </span>
        </div>
        <p class="q">{{ r.question.question }}</p>
        <p class="lede ans">
          正确答案 {{ r.question.answer.join("、") }}　你曾选
          {{ r.entry.lastPicked.length ? r.entry.lastPicked.join("、") : "—" }}
        </p>
        <button type="button" class="toggle" @click="toggle(r.question.id)">
          {{ expanded.has(r.question.id) ? "收起" : "看解析和出处" }}
        </button>
        <FeedbackPanel
          v-if="expanded.has(r.question.id)"
          :question="r.question"
          :picked="r.entry.lastPicked"
          :correct="false"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.rule { margin-top: 12px; }
.acts { margin-top: 14px; }
.list { display: flex; flex-direction: column; gap: 10px; }
.entry { background: var(--surface); border: 1px solid var(--line); border-radius: 11px; padding: 14px 15px; }
.head { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin-bottom: 7px; }
.dots { font-size: 12px; letter-spacing: 1px; color: var(--series-1); }
.q { font-size: 14.5px; line-height: 1.55; }
.ans { font-size: 13px; margin-top: 4px; }
.toggle {
  background: none; border: 0; padding: 0; margin-top: 7px;
  color: var(--accent); font-size: 13px; text-decoration: underline;
}
</style>
