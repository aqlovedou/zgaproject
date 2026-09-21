/** 界面上反复用到的小格式化函数，单独放一处便于测试。 */

/** 秒 → "33:41"。负数按 0 处理。 */
export function clock(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

/** 秒 → "31 分 12 秒"。 */
export function duration(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const m = Math.floor(s / 60);
  return m > 0 ? `${m} 分 ${s % 60} 秒` : `${s} 秒`;
}

/** 0.713 → "71%"。 */
export function percent(rate: number): string {
  return `${Math.round(rate * 100)}%`;
}

export function rate(rights: number, attempts: number): number {
  return attempts > 0 ? rights / attempts : 0;
}

/** epoch 毫秒 → "09/21 15:04"。 */
export function stamp(ms: number): string {
  const d = new Date(ms);
    const p = (n: number) => String(n).padStart(2, "0");
  return `${p(d.getMonth() + 1)}/${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

/** epoch 毫秒 → "已到期" / "3 小时后" / "明天"。 */
export function dueLabel(dueAt: number, now = Date.now()): string {
  const diff = dueAt - now;
  if (diff <= 0) return "已到期";
  const hours = diff / 3_600_000;
  if (hours < 1) return `${Math.ceil(diff / 60_000)} 分钟后`;
  if (hours < 24) return `${Math.ceil(hours)} 小时后`;
  // 用 floor 而不是 ceil：25 小时后是「明天」，不是「2 天后」
  const days = Math.floor(hours / 24);
  return days === 1 ? "明天" : `${days} 天后`;
}

/** 掌握度 3 → "●●●○○"。 */
export function boxDots(level: number): string {
  const n = Math.min(5, Math.max(0, Math.round(level)));
  return "●".repeat(n) + "○".repeat(5 - n);
}

/**
 * 把输入的字母解析成选项。"ac" / "A, C" / "AC" 都接受；
 * 有重复、有不存在的选项、或单选题选了多个，一律返回 null。
 */
export function parseChoice(
  raw: string,
  optionKeys: string[],
  type: "single" | "multi" | "truefalse",
): string[] | null {
  const keys = [...raw.toUpperCase()].filter((c) => /[A-Z]/.test(c));
  if (keys.length === 0) return null;
  if (new Set(keys).size !== keys.length) return null;
  if (keys.some((k) => !optionKeys.includes(k))) return null;
  if (type !== "multi" && keys.length !== 1) return null;
  return keys;
}

/** 两个选项集合是否相同（判分用，顺序无关）。 */
export function sameSet(a: string[], b: string[]): boolean {
  if (a.length !== b.length) return false;
  const x = [...a].sort();
  const y = [...b].sort();
  return x.every((v, i) => v === y[i]);
}
