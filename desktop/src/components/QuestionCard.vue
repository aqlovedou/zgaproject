<script setup lang="ts">
/** 一道题：题头信息 + 选项 + （可选）讲评。 */
import { TYPE_LABEL, type Question } from "@/types";
import FeedbackPanel from "./FeedbackPanel.vue";

defineProps<{
  question: Question;
  picked: string[];
  /** 已提交：高亮对错并显示讲评 */
  revealed: boolean;
  correct?: boolean;
  domainNumber: number;
  hint?: string;
  disabled?: boolean;
}>();
const emit = defineEmits<{ (e: "toggle", key: string): void }>();

function mark(key: string, question: Question, picked: string[], revealed: boolean) {
  if (!revealed) return undefined;
  if (question.answer.includes(key)) return "right";
  if (picked.includes(key)) return "wrong";
  return undefined;
}
</script>

<template>
  <div class="card">
    <div class="meta">
      <span class="chip mono">{{ question.id }}</span>
      <span class="chip chip-accent">领域{{ domainNumber }} · {{ question.topic }}</span>
      <span class="chip">{{ TYPE_LABEL[question.type] }}</span>
      <span v-if="hint" class="chip">{{ hint }}</span>
    </div>

    <p class="qtext">{{ question.question }}</p>

    <div class="opts">
      <button
        v-for="key in Object.keys(question.options).sort()"
        :key="key"
        type="button"
        class="opt"
        :data-mark="mark(key, question, picked, revealed)"
        :aria-pressed="picked.includes(key)"
        :disabled="revealed || disabled"
        @click="emit('toggle', key)"
      >
        <span class="key mono">{{ key }}</span>
        <span>{{ question.options[key] }}</span>
      </button>
    </div>

    <p v-if="question.type === 'multi' && !revealed" class="lede multi-hint">
      多选题：本题需要选 {{ question.answer.length }} 项。
    </p>

    <FeedbackPanel
      v-if="revealed"
      :question="question"
      :picked="picked"
      :correct="correct ?? false"
    />

    <slot name="actions" />
  </div>
</template>

<style scoped>
.meta { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin-bottom: 13px; }
.qtext { font-size: 17px; line-height: 1.6; font-weight: 500; margin-bottom: 15px; }
.opts { display: flex; flex-direction: column; gap: 8px; }
.opt {
  display: flex; gap: 11px; align-items: flex-start; text-align: left; width: 100%;
  border: 1px solid var(--line); background: var(--surface); border-radius: var(--r);
  padding: 11px 13px; font-size: 14.5px; line-height: 1.55;
  transition: border-color 0.12s, background 0.12s;
}
.opt:hover:not(:disabled) { border-color: var(--accent); }
.opt:disabled { cursor: default; }
.opt .key {
  font-size: 12px; font-weight: 600; flex: none; width: 22px; height: 22px;
  border-radius: 6px; display: grid; place-items: center;
  background: var(--surface-2); color: var(--muted); margin-top: 1px;
}
.opt[aria-pressed="true"]:not([data-mark]) { border-color: var(--accent); background: var(--accent-soft); }
.opt[aria-pressed="true"]:not([data-mark]) .key { background: var(--accent); color: #fff; }
.opt[data-mark="right"] { border-color: var(--st-good); background: color-mix(in srgb, var(--st-good) 8%, transparent); }
.opt[data-mark="right"] .key { background: var(--st-good); color: #fff; }
.opt[data-mark="wrong"] { border-color: var(--st-crit); background: color-mix(in srgb, var(--st-crit) 8%, transparent); }
.opt[data-mark="wrong"] .key { background: var(--st-crit); color: #fff; }
.multi-hint { margin-top: 10px; }
</style>
