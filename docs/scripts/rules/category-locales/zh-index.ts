import { chineseRules0 } from "./zh-0.ts";
import { chineseRules1 } from "./zh-1.ts";
import { chineseRules2 } from "./zh-2.ts";
import { chineseRules3 } from "./zh-3.ts";
import { chineseRules4 } from "./zh-4.ts";
import { chineseRules5 } from "./zh-5.ts";
import { chineseRules6 } from "./zh-6.ts";

export const chinesePackets: Record<string, readonly [string, string, string]> = {
  ...chineseRules0,
  ...chineseRules1,
  ...chineseRules2,
  ...chineseRules3,
  ...chineseRules4,
  ...chineseRules5,
  ...chineseRules6,
};

export { chineseShared } from "./zh-shared.ts";
