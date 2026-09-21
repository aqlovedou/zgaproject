<script setup lang="ts">
/**
 * 模拟考成绩趋势。单系列 → 不需要图例（标题已点明画的是什么），
 * 只直标端点而不是每个点都写数字；700 及格线用虚线，和实线网格区分开。
 */
import { computed, ref } from "vue";
import type { ExamPoint } from "@/types";
import { stamp } from "@/format";

const props = defineProps<{ points: ExamPoint[]; passScore: number; maxScore: number }>();

const W = 640, H = 250, L = 46, R = 22, T = 16, B = 42;
const pw = W - L - R;
const ph = H - T - B;

const x = (i: number) =>
  props.points.length <= 1 ? L + pw / 2 : L + (i * pw) / (props.points.length - 1);
const y = (score: number) => T + ph - (score / props.maxScore) * ph;

const line = computed(() =>
  props.points.map((p, i) => `${i ? "L" : "M"}${x(i)} ${y(p.score)}`).join(" "),
);
const ticks = [0, 250, 500, 1000];
const last = computed(() => props.points[props.points.length - 1] ?? null);

const hover = ref<number | null>(null);
const hoverPoint = computed(() =>
  hover.value === null ? null : props.points[hover.value] ?? null,
);
/** 悬停热区宽度：相邻两点间距，首尾各取一半。 */
const hitWidth = computed(() =>
  props.points.length <= 1 ? pw : pw / (props.points.length - 1),
);
</script>

<template>
  <div v-if="points.length === 0" class="empty">
    还没有模拟考记录。做完第一套，这里会出现成绩趋势。
  </div>
  <div v-else class="wrap">
    <svg
      class="chart"
      :viewBox="`0 0 ${W} ${H}`"
      role="img"
      :aria-label="`模拟考成绩趋势：共 ${points.length} 次，最近一次 ${last?.score ?? 0} 分，及格线 ${passScore} 分`"
    >
      <!-- 网格：实线发丝，一档浅于表面 -->
      <g>
        <template v-for="t in ticks" :key="t">
          <line :x1="L" :y1="y(t)" :x2="W - R" :y2="y(t)" stroke="var(--grid)" stroke-width="1" />
          <text
            :x="L - 9" :y="y(t) + 4" text-anchor="end"
            font-family="var(--mono)" font-size="11" fill="var(--muted)"
          >{{ t }}</text>
        </template>
      </g>

      <!-- 及格线：这是真正的阈值，用虚线与网格区分 -->
      <line
        :x1="L" :y1="y(passScore)" :x2="W - R" :y2="y(passScore)"
        stroke="var(--st-crit)" stroke-width="1.5" stroke-dasharray="5 4" opacity="0.75"
      />
      <text
        :x="L + 5" :y="y(passScore) - 8" text-anchor="start"
        font-family="var(--mono)" font-size="11" fill="var(--st-crit)"
      >及格线 {{ passScore }}</text>

      <line :x1="L" :y1="y(0)" :x2="W - R" :y2="y(0)" stroke="var(--axis)" stroke-width="1" />

      <path :d="line" fill="none" stroke="var(--series-1)" stroke-width="2"
            stroke-linejoin="round" stroke-linecap="round" />

      <g v-for="(p, i) in points" :key="p.sessionId">
        <circle :cx="x(i)" :cy="y(p.score)" r="4.5" fill="var(--series-1)"
                stroke="var(--surface)" stroke-width="2" />
        <text
          v-if="points.length <= 12"
          :x="x(i)" :y="H - B + 20" text-anchor="middle"
          font-family="var(--mono)" font-size="11" fill="var(--muted)"
        >{{ stamp(p.at).slice(0, 5) }}</text>
      </g>

      <!-- 只标端点 -->
      <text
        v-if="last"
        :x="x(points.length - 1) - 6" :y="y(last.score) - 13" text-anchor="end"
        font-size="13" font-weight="600" fill="var(--ink)"
      >{{ last.score }} 分</text>

      <line
        v-if="hover !== null"
        :x1="x(hover)" :y1="T" :x2="x(hover)" :y2="T + ph"
        stroke="var(--axis)" stroke-width="1" opacity="0.6"
      />

      <rect
        v-for="(p, i) in points" :key="`hit-${p.sessionId}`"
        :x="Math.max(L, x(i) - hitWidth / 2)" :y="T"
        :width="hitWidth" :height="ph" fill="transparent" style="cursor: crosshair"
        @mouseenter="hover = i" @mouseleave="hover = null"
      />
    </svg>

    <div
      v-if="hoverPoint && hover !== null"
      class="tip mono"
      :style="{ left: `${(x(hover) / W) * 100}%`, top: `${(y(hoverPoint.score) / H) * 100}%` }"
    >
      {{ stamp(hoverPoint.at) }} · {{ hoverPoint.correct }}/{{ hoverPoint.total }} ·
      {{ hoverPoint.score }} 分 · {{ hoverPoint.score >= passScore ? "达标" : "未达标" }}
    </div>
  </div>
</template>

<style scoped>
.wrap { position: relative; }
.chart { display: block; width: 100%; height: auto; overflow: visible; }
.tip {
  position: absolute; pointer-events: none; transform: translate(-50%, -125%);
  background: var(--ink); color: var(--surface); border-radius: 7px;
  padding: 6px 9px; font-size: 11.5px; white-space: nowrap; z-index: 5;
}
</style>
