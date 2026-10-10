import { portugueseRules0 } from "./pt-0.ts";
import { portugueseRules1 } from "./pt-1.ts";
import { portugueseRules2 } from "./pt-2.ts";
import { portugueseRules3 } from "./pt-3.ts";
import { portugueseRules4 } from "./pt-4.ts";
import { portugueseRules5 } from "./pt-5.ts";

export const portuguesePackets: Record<string, readonly [string, string, string]> = {
  ...portugueseRules0,
  ...portugueseRules1,
  ...portugueseRules2,
  ...portugueseRules3,
  ...portugueseRules4,
  ...portugueseRules5,
};

export { portugueseShared } from "./pt-shared.ts";
export {
  portugueseCategoryTitles,
  portugueseCategoryIntro,
  portugueseCrossFileIntro,
} from "./pt-labels.ts";
