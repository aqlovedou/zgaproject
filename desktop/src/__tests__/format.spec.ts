import { describe, expect, it } from "vitest";
import {
  boxDots,
  clock,
  dueLabel,
  duration,
  parseChoice,
  percent,
  rate,
  sameSet,
} from "@/format";

describe("倒计时格式", () => {
  it("补零到两位", () => {
    expect(clock(2021)).toBe("33:41");
    expect(clock(65)).toBe("1:05");
    expect(clock(0)).toBe("0:00");
  });
  it("负数按 0 处理，不显示 -1:-1", () => {
    expect(clock(-30)).toBe("0:00");
  });
});

describe("用时格式", () => {
  it("超过一分钟显示分和秒", () => {
    expect(duration(1872)).toBe("31 分 12 秒");
  });
  it("不足一分钟只显示秒", () => {
    expect(duration(45)).toBe("45 秒");
  });
});

describe("正确率", () => {
  it("四舍五入成百分数", () => {
    expect(percent(0.713)).toBe("71%");
    expect(percent(1)).toBe("100%");
  });
  it("分母为零不产生 NaN", () => {
    expect(rate(0, 0)).toBe(0);
    expect(percent(rate(0, 0))).toBe("0%");
  });
});

describe("下次复习时间", () => {
  const now = 1_700_000_000_000;
  it("到点了就说已到期", () => {
    expect(dueLabel(now - 1, now)).toBe("已到期");
    expect(dueLabel(now, now)).toBe("已到期");
  });
  it("按小时和天换算", () => {
    expect(dueLabel(now + 30 * 60_000, now)).toBe("30 分钟后");
    expect(dueLabel(now + 3 * 3_600_000, now)).toBe("3 小时后");
    expect(dueLabel(now + 25 * 3_600_000, now)).toBe("明天");
    expect(dueLabel(now + 24 * 3_600_000, now)).toBe("明天");
    expect(dueLabel(now + 50 * 3_600_000, now)).toBe("2 天后");
  });
});

describe("掌握度圆点", () => {
  it("总是五个字符", () => {
    expect(boxDots(3)).toBe("●●●○○");
    expect(boxDots(1)).toBe("●○○○○");
    expect(boxDots(5)).toBe("●●●●●");
  });
  it("越界也不会画歪", () => {
    expect(boxDots(0)).toBe("○○○○○");
    expect(boxDots(9)).toBe("●●●●●");
  });
});

describe("选项解析", () => {
  const keys = ["A", "B", "C", "D"];
  it("接受各种写法", () => {
    expect(parseChoice("a", keys, "single")).toEqual(["A"]);
    expect(parseChoice("AC", keys, "multi")).toEqual(["A", "C"]);
    expect(parseChoice("a, c", keys, "multi")).toEqual(["A", "C"]);
  });
  it("拒绝重复选项", () => {
    expect(parseChoice("AA", keys, "multi")).toBeNull();
  });
  it("拒绝不存在的选项", () => {
    expect(parseChoice("Z", keys, "single")).toBeNull();
  });
  it("单选题不能多选", () => {
    expect(parseChoice("AB", keys, "single")).toBeNull();
    expect(parseChoice("AB", keys, "truefalse")).toBeNull();
  });
  it("空输入返回 null", () => {
    expect(parseChoice("", keys, "single")).toBeNull();
    expect(parseChoice("  ", keys, "single")).toBeNull();
  });
});

describe("判分：集合相等", () => {
  it("顺序无关", () => {
    expect(sameSet(["A", "C"], ["C", "A"])).toBe(true);
  });
  it("少选算错", () => {
    expect(sameSet(["A"], ["A", "C"])).toBe(false);
  });
  it("多选算错", () => {
    expect(sameSet(["A", "B", "C"], ["A", "C"])).toBe(false);
  });
  it("空对空", () => {
    expect(sameSet([], [])).toBe(true);
  });
});
