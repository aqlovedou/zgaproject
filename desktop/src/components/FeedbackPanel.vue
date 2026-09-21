<script setup lang="ts">
/** 讲评：解析 / 你选的为什么错 / 你漏掉的正确项 / 官方出处。 */
import { computed } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Question } from "@/types";

const props = defineProps<{ question: Question; picked: string[]; correct: boolean }>();

const wrongKeys = computed(() => props.picked.filter((k) => !props.question.answer.includes(k)));
const missedKeys = computed(() =>
  props.question.answer.filter((k) => !props.picked.includes(k)),
);
const reason = (k: string) =>
  props.question.optionReasons[k] ?? "该选项与题干要求的场景不符。";

/** 在系统默认浏览器里打开官方文档，而不是在应用窗口里跳转。 */
async function open() {
  try {
    await openUrl(props.question.referenceUrl);
  } catch {
    /* 打不开就算了，链接文本仍然可见可复制 */
  }
}
</script>

<template>
  <div class="feedback">
    <div class="verdict" :class="correct ? 'ok' : 'no'">
      <span aria-hidden="true">{{ correct ? "✓" : "✕" }}</span>
      <span>{{ correct ? "回答正确" : "回答错误" }}</span>
    </div>

    <p v-if="!correct" class="lede mine">
      你选了 {{ picked.join("、") || "（未作答）" }}，正确答案是 {{ question.answer.join("、") }}。
    </p>

    <div class="note">
      <h5>解析</h5>
      <p>{{ question.explanation }}</p>
    </div>

    <div v-if="wrongKeys.length" class="note note-why">
      <h5>你选的为什么错</h5>
      <div v-for="k in wrongKeys" :key="k" class="wrong-opt">
        <b>{{ k }}. {{ question.options[k] }}</b>
        <p>{{ reason(k) }}</p>
      </div>
    </div>

    <div v-if="missedKeys.length && question.type === 'multi' && picked.length" class="note">
      <h5>你漏掉的正确项</h5>
      <p v-for="k in missedKeys" :key="k">{{ k }}. {{ question.options[k] }}</p>
    </div>

    <div class="note note-ref">
      <h5>官方出处</h5>
      <p>{{ question.referenceTitle }}</p>
      <button type="button" class="link mono" @click="open">{{ question.referenceUrl }}</button>
    </div>
  </div>
</template>

<style scoped>
.feedback { display: flex; flex-direction: column; gap: 13px; margin-top: 16px; }
.verdict { display: flex; align-items: center; gap: 8px; font-weight: 600; font-size: 15px; }
.verdict.ok { color: var(--st-good); }
.verdict.no { color: var(--st-crit); }
.mine { margin: 0; }
.note { border-left: 3px solid var(--line-strong); padding-left: 13px; }
.note h5 {
  font-family: var(--mono); font-size: 10.5px; letter-spacing: 0.09em;
  text-transform: uppercase; color: var(--muted); margin: 0 0 5px; font-weight: 600;
}
.note p { font-size: 14px; line-height: 1.7; }
.note-why { border-color: var(--st-crit); }
.note-why h5 { color: var(--st-crit); }
.note-ref { border-color: var(--accent); }
.note-ref h5 { color: var(--accent); }
.wrong-opt { margin-bottom: 9px; }
.wrong-opt:last-child { margin-bottom: 0; }
.wrong-opt b { font-weight: 600; font-size: 14px; }
.link {
  background: none; border: 0; padding: 0; text-align: left;
  color: var(--accent); font-size: 12.5px; text-decoration: underline; word-break: break-all;
}
</style>
