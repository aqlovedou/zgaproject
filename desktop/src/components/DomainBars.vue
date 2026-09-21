<script setup lang="ts">
/**
 * 领域正确率。领域之间没有大小顺序 → 所有条同一个系列色，
 * 达标与否用「图标 + 文字」的状态标签表达，不靠颜色单独承载信息。
 */
import { percent, rate } from "@/format";

interface Row {
  number: number;
  nameCn: string;
  officialRange?: string;
  rights: number;
  attempts: number;
}
const props = defineProps<{ rows: Row[]; target?: number; showRange?: boolean }>();
const target = props.target ?? 0.8;
</script>

<template>
  <div class="bars">
    <div v-for="row in rows" :key="row.number" class="bar">
      <div class="top">
        <span class="name">
          领域{{ row.number }} {{ row.nameCn }}
          <span v-if="showRange && row.officialRange" class="range mono">
            官方权重 {{ row.officialRange }}
          </span>
        </span>
        <span class="right">
          <template v-if="row.attempts > 0">
            <span
              class="pill"
              :class="rate(row.rights, row.attempts) >= target ? 'pill-good' : 'pill-warn'"
            >
              <span aria-hidden="true">{{ rate(row.rights, row.attempts) >= target ? "✓" : "!" }}</span>
              <span>{{ rate(row.rights, row.attempts) >= target ? "达标" : "待加强" }}</span>
            </span>
            <span class="num mono">
              {{ row.rights }}/{{ row.attempts }}　{{ percent(rate(row.rights, row.attempts)) }}
            </span>
          </template>
          <span v-else class="num mono">未练</span>
        </span>
      </div>
      <div class="track">
        <div
          class="fill"
          :style="{ width: `${rate(row.rights, row.attempts) * 100}%` }"
        />
        <div class="target" :style="{ left: `${target * 100}%` }" />
      </div>
    </div>
    <p class="caption mono">竖线为 {{ percent(target) }} 目标线</p>
  </div>
</template>

<style scoped>
.bars { display: flex; flex-direction: column; gap: 14px; }
.top {
  display: flex; justify-content: space-between; align-items: baseline;
  gap: 10px; margin-bottom: 5px; flex-wrap: wrap;
}
.name { font-size: 13.5px; }
.range { font-size: 11px; color: var(--faint); margin-left: 6px; }
.num { font-size: 12px; color: var(--muted); }
.track { position: relative; height: 9px; border-radius: 5px; background: var(--surface-3); }
/* 领域之间没有大小顺序，所以每条都是同一个系列色；
   达标与否由「图标 + 文字」的状态标签承载，不靠颜色单独表达 */
.fill { height: 100%; border-radius: 5px; background: var(--series-1); }
.target { position: absolute; top: -4px; bottom: -4px; width: 2px; background: var(--axis); }
.caption { font-size: 11.5px; color: var(--muted); margin-top: 2px; }
</style>
