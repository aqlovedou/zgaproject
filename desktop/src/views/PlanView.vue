<script setup lang="ts">
/** 五天冲刺计划。每条任务都是一个按钮，点了直接开练。 */
import { useRouter } from "vue-router";

const router = useRouter();

interface Task { label: string; query: Record<string, string> }
interface Day { day: number; title: string; focus: string; tasks: Task[]; target: string }

const PLAN: Day[] = [
  {
    day: 1,
    title: "云概念打底 + Azure 架构骨架",
    focus:
      "共享责任模型、云部署模型、消费模型与 CapEx/OpEx、云的优势、IaaS/PaaS/SaaS、区域与可用性区域、资源组织层级",
    tasks: [
      { label: "刷完领域 1", query: { mode: "practice", domain: "cloud-concepts" } },
      { label: "练「区域与可用性区域」", query: { mode: "practice", topic: "核心架构：区域与可用性区域" } },
      { label: "练「资源组织层级」", query: { mode: "practice", topic: "核心架构：资源组织层级" } },
      { label: "20 题小测热身", query: { mode: "exam", count: "20" } },
    ],
    target: "领域 1 正确率 ≥ 80%",
  },
  {
    day: 2,
    title: "计算 + 网络（领域 2 的最大失分区）",
    focus:
      "虚拟机与规模集、可用性集、虚拟桌面、容器与 AKS、Functions、App Service、虚拟网络与子网、VPN 网关、ExpressRoute、专用终结点",
    tasks: [
      { label: "练所有「计算服务」题", query: { mode: "practice", topicLike: "计算服务" } },
      { label: "练所有「网络服务」题", query: { mode: "practice", topicLike: "网络服务" } },
      { label: "清昨天的错题", query: { mode: "review" } },
      { label: "30 题模拟", query: { mode: "exam", count: "30" } },
    ],
    target: "计算/网络类题目正确率 ≥ 75%",
  },
  {
    day: 3,
    title: "存储 + 身份与安全",
    focus:
      "存储账户与冗余（LRS/ZRS/GRS/GZRS）、访问层、迁移工具、Entra ID、MFA 与条件访问、RBAC、零信任、纵深防御、Defender for Cloud",
    tasks: [
      { label: "练所有「存储服务」题", query: { mode: "practice", topicLike: "存储服务" } },
      { label: "练所有「身份与访问」题", query: { mode: "practice", topicLike: "身份与访问" } },
      { label: "练所有「安全」题", query: { mode: "practice", topicLike: "安全：" } },
      { label: "错题复习", query: { mode: "review" } },
      { label: "40 题模拟", query: { mode: "exam", count: "40" } },
    ],
    target: "领域 2 整体 ≥ 75%，模拟考 ≥ 650 分",
  },
  {
    day: 4,
    title: "管理与治理 + 第一次全真模拟",
    focus:
      "成本因素与优化、定价计算器 vs TCO、Cost Management 与预算、标记、Azure Policy 与资源锁、Purview、服务信任门户、门户/CLI/PowerShell/Cloud Shell、ARM 与 Bicep、Monitor / Advisor / Service Health",
    tasks: [
      { label: "刷完领域 3", query: { mode: "practice", domain: "management-governance" } },
      { label: "错题复习", query: { mode: "review" } },
      { label: "50 题全真模拟", query: { mode: "exam", count: "50" } },
    ],
    target: "全真模拟 ≥ 700 分（首次达标）",
  },
  {
    day: 5,
    title: "错题清零 + 两次全真模拟稳定达标",
    focus: "错题本全部攻克、三大领域均衡、考试节奏（平均每题 ≤ 54 秒）",
    tasks: [
      { label: "反复刷到错题本清空", query: { mode: "review" } },
      { label: "第二次全真模拟", query: { mode: "exam", count: "50" } },
      { label: "第三次全真模拟", query: { mode: "exam", count: "50" } },
    ],
    target: "连续两次模拟 ≥ 800 分，且错题本已攻克 ≥ 90%",
  },
];
</script>

<template>
  <div class="stack">
    <div class="card">
      <div class="eyebrow">五天冲刺</div>
      <h2 class="title">5 天把 AZ-900 刷到稳定达标</h2>
      <p class="lede">
        题库按官方技能大纲的三大领域和权重编写。每题都有解析、每个错误选项为什么错、
        以及 Microsoft Learn 官方出处。答错自动进错题本，按间隔重复复习到攻克为止。
      </p>
      <p class="lede note">
        考试规则：满分 1000 分，<b>700 分及格</b>，约 40–60 题，答题时间 45 分钟。
        本应用模拟考按官方权重组卷（领域1 25–30% / 领域2 35–40% / 领域3 30–35%）。
      </p>
      <div class="row acts">
        <button
          type="button" class="btn btn-primary"
          @click="router.push({ name: 'quiz', query: { mode: 'exam', count: '50' } })"
        >
          开始 50 题全真模拟
        </button>
        <button
          type="button" class="btn"
          @click="router.push({ name: 'quiz', query: { mode: 'exam', count: '20' } })"
        >
          20 题快速摸底
        </button>
      </div>
    </div>

    <div v-for="d in PLAN" :key="d.day" class="card">
      <div class="day">
        <h3>Day {{ d.day }}　{{ d.title }}</h3>
        <p class="focus">{{ d.focus }}</p>
        <div class="row">
          <button
            v-for="t in d.tasks" :key="t.label" type="button" class="btn btn-sm"
            @click="router.push({ name: 'quiz', query: t.query })"
          >
            {{ t.label }}
          </button>
        </div>
        <p class="target">达标线：{{ d.target }}</p>
      </div>
    </div>

    <p class="lede src">
      题目依据微软公开的 AZ-900 技能大纲原创编写，不含任何考试泄题内容。
    </p>
  </div>
</template>

<style scoped>
.title { font-size: 21px; margin: 6px 0 10px; }
.note { margin-top: 10px; }
.acts { margin-top: 16px; }
.day { border-left: 3px solid var(--line-strong); padding-left: 15px; }
.day h3 { font-size: 15.5px; margin-bottom: 5px; }
.focus { font-size: 13.5px; color: var(--muted); margin: 5px 0 11px; }
.target {
  font-size: 13px; color: var(--st-warn); background: color-mix(in srgb, var(--st-warn) 12%, transparent);
  padding: 5px 10px; border-radius: 7px; display: inline-block; margin-top: 11px;
}
.src { font-size: 12.5px; }
</style>
