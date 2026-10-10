import { frenchRules0 } from "./fr-0.ts";
import { frenchRules1 } from "./fr-1.ts";
import { frenchRules2 } from "./fr-2.ts";
import { frenchRules3 } from "./fr-3.ts";
import { frenchRules4 } from "./fr-4.ts";
import { frenchRules5 } from "./fr-5.ts";
import { frenchRules6 } from "./fr-6.ts";

export const frenchPackets = {
  ...frenchRules0,
  ...frenchRules1,
  ...frenchRules2,
  ...frenchRules3,
  ...frenchRules4,
  ...frenchRules5,
  ...frenchRules6,
} satisfies Record<string, readonly [string, string, string]>;

export { frenchShared } from "./fr-shared.ts";
export { frenchCategoryTitles, frenchCategoryIntro, frenchCrossFileIntro } from "./fr-labels.ts";
