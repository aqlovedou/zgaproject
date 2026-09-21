<script setup lang="ts">
/** 仪表盘：四个关键数字 + 三张图 + 准备度清单。 */
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { api } from "@/api";
import type { Dashboard } from "@/types";
import { percent, rate } from "@/format";
import StatTile from "@/components/StatTile.vue";
import ScoreTrend from "@/components/ScoreTrend.vue";
import DomainBars from "@/components/DomainBars.vue";
import BoxDistribution from "@/components/BoxDistribution.vue";

const router = useRouter();
const data = ref<Dashboard | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    data.value = await api.dashboard();
  } catch (e) {
    error.value = String(e);
  }
});

function practiceTopic(topic: string) {
  router.push({ name: "quiz", query: { mode: "practice", topic } });
}
</script>

<template>
  <div v-if="error" class="card empty">读取数据失败：{{ error }}</div>
  <div v-else-if="!data" class="card empty">载入中…</div>
  <div v-else class="stack">
    <div class="tiles">
      <StatTile
        label="最近一次模拟"
        :value="data.overview.lastExamScore !== null ? String(data.overview.lastExamScore) : '—'"
        :sub="
          data.overview.lastExamScore !== null
            ? `分 · ${data.overview.lastExamScore >= data.passScore ? '达标 ✓' : '未达标'}`
            : '还没考过'
        "
        :tone="
          data.overview.lastExamScore === null
            ? undefined
            : data.overview.lastExamScore >= data.passScore
              ? 'good'
              : 'crit'
        "
      />
      <StatTile
        label="已练题数"
        :value="String(data.overview.seenCount)"
        :sub="`/ ${data.overview.bankSize} 题 · ${percent(rate(data.overview.seenCount, data.overview.bankSize))}`"
      />
      <StatTile
        label="错题待攻克"
        :value="String(data.overview.wrongPending)"
        :sub="`题 · 今天到期 ${data.overview.wrongDueNow} 题`"
        :tone="data.overview.wrongPending > 0 ? 'warn' : 'good'"
      />
      <StatTile
        label="平均每题"
        :value="data.overview.avgSeconds !== null ? String(Math.round(data.overview.avgSeconds)) : '—'"
        sub="秒 · 限 54 秒"
      />
    </div>

    <div class="card">
      <div class="card-head">
        <h4>上考场准备度</h4>
        <span class="sub">{{ data.readiness.ready ? "四项已满足" : "尚未满足" }}</span>
      </div>
      <h2 class="verdict" :class="data.readiness.ready ? 'good' : 'warn'">
        {{ data.readiness.ready ? "可以约考了" : "还没到稳定达标" }}
      </h2>
      <ul class="checks">
        <li :class="{ on: data.readiness.twoExamsOver800 }">
          <span aria-hidden="true">{{ data.readiness.twoExamsOver800 ? "✓" : "○" }}</span>
          连续两次模拟 ≥ 800 分
        </li>
        <li :class="{ on: data.readiness.allDomainsOver80 }">
          <span aria-hidden="true">{{ data.readiness.allDomainsOver80 ? "✓" : "○" }}</span>
          三个领域正确率都 ≥ 80%
        </li>
        <li :class="{ on: data.readiness.wrongbookClear }">
          <span aria-hidden="true">{{ data.readiness.wrongbookClear ? "✓" : "○" }}</span>
          错题本清空
        </li>
        <li :class="{ on: data.readiness.coverageOver90 }">
          <span aria-hidden="true">{{ data.readiness.coverageOver90 ? "✓" : "○" }}</span>
          题库覆盖 ≥ 90%
        </li>
      </ul>
      <p class="lede next">下一步：{{ data.readiness.nextStep }}</p>
    </div>

    <div class="card">
      <div class="card-head">
        <h4>模拟考成绩趋势</h4>
        <span class="sub">{{ data.exams.length }} 次 · {{ data.maxScore }} 分制</span>
      </div>
      <ScoreTrend :points="data.exams" :pass-score="data.passScore" :max-score="data.maxScore" />
    </div>

    <div class="card">
      <div class="card-head">
        <h4>三领域累计正确率</h4>
        <span class="sub">目标 ≥ 80%</span>
      </div>
      <DomainBars :rows="data.domains" show-range />
    </div>

    <div class="card">
      <div class="card-head">
        <h4>错题掌握度分布</h4>
        <span class="sub">{{ data.overview.wrongPending + data.overview.wrongMastered }} 题在册</span>
      </div>
      <BoxDistribution :buckets="data.boxes" />
    </div>

    <div v-if="data.weakTopics.length" class="card">
      <div class="card-head">
        <h4>最该补的考点</h4>
        <span class="sub">点考点名直接练</span>
      </div>
      <div class="weak">
        <div v-for="t in data.weakTopics" :key="t.topic" class="weak-row">
          <button type="button" class="topic" @click="practiceTopic(t.topic)">{{ t.topic }}</button>
          <span class="mono num">{{ t.rights }}/{{ t.attempts }}　{{ percent(rate(t.rights, t.attempts)) }}</span>
          <div class="track">
            <div
              class="fill"
              :style="{
                width: `${rate(t.rights, t.attempts) * 100}%`,
                background:
                  rate(t.rights, t.attempts) >= 0.8
                    ? 'var(--st-good)'
                    : rate(t.rights, t.attempts) >= 0.6
                      ? 'var(--st-warn)'
                      : 'var(--st-crit)',
              }"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tiles { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 12px; }
.verdict { font-size: 22px; margin: 2px 0 12px; }
.verdict.good { color: var(--st-good); }
.verdict.warn { color: var(--st-warn); }
.checks { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 5px; }
.checks li { font-size: 13.5px; color: var(--muted); display: flex; gap: 8px; }
.checks li.on { color: var(--st-good); }
.next { margin-top: 13px; }
.weak { display: flex; flex-direction: column; gap: 12px; }
.weak-row { display: grid; grid-template-columns: 1fr auto; gap: 4px 12px; align-items: baseline; }
.topic {
  background: none; border: 0; padding: 0; text-align: left; font-size: 13.5px;
  color: var(--ink); text-decoration: underline; text-decoration-color: var(--line-strong);
}
.topic:hover { color: var(--accent); }
.num { font-size: 12px; color: var(--muted); }
.track { grid-column: 1 / -1; height: 7px; border-radius: 4px; background: var(--surface-3); overflow: hidden; }
.fill { height: 100%; border-radius: 4px; }
</style>
