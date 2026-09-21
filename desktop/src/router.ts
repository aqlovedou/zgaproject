import { createRouter, createWebHashHistory } from "vue-router";

/**
 * 桌面应用用 hash 路由：Tauri 加载的是本地文件，history 模式刷新会 404。
 */
export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/plan" },
    { path: "/plan", name: "plan", component: () => import("@/views/PlanView.vue") },
    { path: "/dashboard", name: "dashboard", component: () => import("@/views/DashboardView.vue") },
    { path: "/practice", name: "practice", component: () => import("@/views/PracticeView.vue") },
    { path: "/wrongbook", name: "wrongbook", component: () => import("@/views/WrongbookView.vue") },
    { path: "/quiz", name: "quiz", component: () => import("@/views/QuizView.vue") },
    { path: "/result", name: "result", component: () => import("@/views/ResultView.vue") },
    { path: "/:pathMatch(.*)*", redirect: "/plan" },
  ],
});
