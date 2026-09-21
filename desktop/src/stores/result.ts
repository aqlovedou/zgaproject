/** 交卷结果的临时存放处：答题页交卷后写入，成绩单页读取。 */
import { defineStore } from "pinia";
import type { SessionResult } from "@/types";

export const useResultStore = defineStore("result", {
  state: () => ({ result: null as SessionResult | null }),
  actions: {
    set(r: SessionResult) {
      this.result = r;
    },
    clear() {
      this.result = null;
    },
  },
});
