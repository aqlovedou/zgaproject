<script setup lang="ts">
/**
 * 错题掌握度分布。1→5 级是有序的，所以用单色序数阶梯（浅→深），
 * 段之间留 2px 表面色缝隙而不是描边。图例带数值，兼作表格视图。
 */
import { computed } from "vue";
import type { BoxBucket } from "@/types";

const props = defineProps<{ buckets: BoxBucket[] }>();
const LABELS = ["1 级 刚错", "2 级", "3 级", "4 级", "5 级 已攻克"];
const total = computed(() => props.buckets.reduce((s, b) => s + b.count, 0));
const pct = (n: number) => (total.value ? (n / total.value) * 100 : 0);
</script>

<template>
  <div v-if="total === 0" class="empty">错题本是空的。</div>
  <div v-else>
    <div class="stackbar">
      <i
        v-for="b in buckets.filter((x) => x.count > 0)"
        :key="b.boxLevel"
        :style="{ width: `${pct(b.count)}%`, background: `var(--ord-${b.boxLevel})` }"
      />
    </div>
    <div class="legend">
      <span v-for="b in buckets" :key="b.boxLevel">
        <i :style="{ background: `var(--ord-${b.boxLevel})` }" />
        <span>{{ LABELS[b.boxLevel - 1] }} </span>
        <b class="mono">{{ b.count }} 题</b>
      </span>
    </div>
  </div>
</template>

<style scoped>
.stackbar {
  display: flex; gap: 2px; height: 13px; border-radius: 4px;
  overflow: hidden; margin-bottom: 12px;
}
.stackbar i { display: block; height: 100%; }
.legend { display: flex; gap: 14px; flex-wrap: wrap; font-size: 12px; color: var(--muted); }
.legend > span { display: inline-flex; align-items: center; gap: 6px; }
.legend i { width: 9px; height: 9px; border-radius: 2px; display: block; flex: none; }
.legend b { color: var(--ink); font-weight: 500; }
</style>
