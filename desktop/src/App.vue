<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { api } from "@/api";
import { useQuizStore } from "@/stores/quiz";

const route = useRoute();
const router = useRouter();
const quiz = useQuizStore();

const bankSize = ref<number | null>(null);
const dbPath = ref("");

onMounted(async () => {
  try {
    [bankSize.value, dbPath.value] = await Promise.all([api.bankSize(), api.dbPath()]);
  } catch {
    /* 后端还没起来时不阻塞界面 */
  }
});

const NAV = [
  { name: "plan", label: "计划" },
  { name: "dashboard", label: "仪表盘" },
  { name: "practice", label: "专项练习" },
  { name: "wrongbook", label: "错题本" },
] as const;

/** 答题过程中切换导航要确认，避免误点丢掉半套题。 */
function navigate(name: string) {
  if (quiz.active && !confirm("离开会中断当前这套题，已作答的部分不会计分。确定离开？")) return;
  quiz.reset();
  router.push({ name });
}

const activeName = computed(() => route.name);
</script>

<template>
  <div class="shell">
    <header class="topbar">
      <div class="inner">
        <div class="brand">
          <b>AZ-900 刷题台</b>
          <span v-if="bankSize" class="mono">{{ bankSize }} 题</span>
        </div>
        <nav class="tabs" role="tablist">
          <button
            v-for="n in NAV" :key="n.name" type="button" role="tab"
            :aria-selected="activeName === n.name" @click="navigate(n.name)"
          >
            {{ n.label }}
          </button>
        </nav>
      </div>
    </header>

    <main>
      <router-view />
    </main>

    <footer v-if="dbPath" class="foot mono">成绩与错题本保存在 {{ dbPath }}</footer>
  </div>
</template>

<style scoped>
.shell { min-height: 100%; display: flex; flex-direction: column; }
.topbar {
  position: sticky; top: 0; z-index: 20;
  background: color-mix(in srgb, var(--ground) 88%, transparent);
  backdrop-filter: blur(12px); border-bottom: 1px solid var(--line);
}
.inner {
  max-width: 820px; margin: 0 auto; padding: 10px 20px;
  display: flex; align-items: center; gap: 14px; flex-wrap: wrap;
}
.brand { display: flex; align-items: baseline; gap: 8px; margin-right: auto; }
.brand b { font-size: 15px; font-weight: 600; letter-spacing: -0.02em; }
.brand span { font-size: 11px; color: var(--muted); letter-spacing: 0.06em; }
.tabs { display: flex; gap: 2px; background: var(--surface-2); padding: 3px; border-radius: var(--r); flex-wrap: wrap; }
.tabs button {
  border: 0; background: transparent; padding: 5px 12px; border-radius: 7px;
  font-size: 13px; color: var(--muted); font-weight: 500;
}
.tabs button[aria-selected="true"] {
  background: var(--surface); color: var(--ink); box-shadow: 0 1px 2px rgba(15, 23, 42, 0.1);
}
.tabs button:hover[aria-selected="false"] { color: var(--ink); }
main { flex: 1; max-width: 820px; width: 100%; margin: 0 auto; padding: 22px 20px 60px; }
.foot { max-width: 820px; margin: 0 auto; padding: 0 20px 20px; font-size: 11px; color: var(--faint); word-break: break-all; }
</style>
