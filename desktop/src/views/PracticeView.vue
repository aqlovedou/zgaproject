<script setup lang="ts">
/** 专项练习入口：按领域、按考点、或只练没做过的。 */
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { api } from "@/api";
import type { DomainKey, TopicRow } from "@/types";

const router = useRouter();
const topics = ref<TopicRow[]>([]);
const bankSize = ref(0);
const keyword = ref("");

const DOMAINS: { key: DomainKey; no: number; cn: string }[] = [
  { key: "cloud-concepts", no: 1, cn: "云概念" },
  { key: "architecture-services", no: 2, cn: "Azure 架构与服务" },
  { key: "management-governance", no: 3, cn: "Azure 管理与治理" },
];

onMounted(async () => {
  [topics.value, bankSize.value] = await Promise.all([api.topics(), api.bankSize()]);
});

const byDomain = computed(() =>
  DOMAINS.map((d) => ({
    ...d,
    rows: topics.value.filter((t) => t.domain === d.key),
    count: topics.value.filter((t) => t.domain === d.key).reduce((s, t) => s + t.count, 0),
    range: topics.value.find((t) => t.domain === d.key)?.officialRange ?? "",
  })),
);

const go = (query: Record<string, string>) =>
  router.push({ name: "quiz", query: { mode: "practice", ...query } });
</script>

<template>
  <div class="stack">
    <div class="card">
      <div class="eyebrow">专项练习</div>
      <h2 class="title">按领域或考点刷，答一题讲一题</h2>
      <p class="lede">
        专项练习会立刻给出解析和出处，适合打基础和补弱项。答错同样自动进错题本。
      </p>
      <div class="row acts">
        <button
          v-for="d in byDomain" :key="d.key" type="button" class="btn"
          @click="go({ domain: d.key })"
        >
          领域{{ d.no }} {{ d.cn }}（{{ d.count }} 题）
        </button>
        <button type="button" class="btn btn-primary" @click="go({ onlyUnseen: '1' })">
          只练没做过的
        </button>
      </div>
      <form class="search" @submit.prevent="keyword.trim() && go({ keyword: keyword.trim() })">
        <input
          v-model="keyword" type="search" class="input"
          placeholder="搜题干关键词，比如 ExpressRoute、GRS、条件访问"
          aria-label="按关键词搜题"
        />
        <button type="submit" class="btn btn-sm" :disabled="!keyword.trim()">搜并开练</button>
      </form>
    </div>

    <div v-for="d in byDomain" :key="d.key" class="card">
      <div class="card-head">
        <h4>领域{{ d.no }} {{ d.cn }}</h4>
        <span class="sub">官方权重 {{ d.range }} · 本题库 {{ d.count }} 题</span>
      </div>
      <div class="row">
        <button
          v-for="t in d.rows" :key="t.topic" type="button" class="btn btn-sm"
          @click="go({ topic: t.topic })"
        >
          {{ t.topic }}（{{ t.count }}）
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.title { font-size: 20px; margin: 6px 0 10px; }
.acts { margin-top: 14px; }
.search { display: flex; gap: 9px; margin-top: 14px; flex-wrap: wrap; }
.input {
  flex: 1; min-width: 200px; font: inherit; font-size: 14px; color: var(--ink);
  background: var(--surface); border: 1px solid var(--line-strong);
  border-radius: 9px; padding: 8px 12px;
}
.input:focus { outline: 2px solid var(--accent); outline-offset: 1px; }
</style>
