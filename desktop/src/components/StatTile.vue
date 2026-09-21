<script setup lang="ts">
/** 一个关键数字。数字本身是主角，所以不给它画图表。 */
defineProps<{ label: string; value: string; sub?: string; tone?: "good" | "warn" | "crit" }>();
</script>

<template>
  <div class="tile">
    <div class="k">{{ label }}</div>
    <div class="v" :class="tone ? `tone-${tone}` : ''">{{ value }}</div>
    <div v-if="sub" class="s mono">{{ sub }}</div>
  </div>
</template>

<style scoped>
.tile { border: 1px solid var(--line); border-radius: 10px; padding: 13px 15px; background: var(--surface); }
.k { font-size: 11.5px; color: var(--muted); margin-bottom: 4px; }
.v { font-size: 27px; font-weight: 600; line-height: 1.15; letter-spacing: -0.02em; }
.s { font-size: 11.5px; color: var(--muted); }
.tone-good { color: var(--st-good); }
.tone-warn { color: var(--st-warn); }
.tone-crit { color: var(--st-crit); }
</style>
