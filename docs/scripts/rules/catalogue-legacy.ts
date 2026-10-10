import { legacyen } from "./catalogue-legacy-en.ts";
import { legacyja } from "./catalogue-legacy-ja.ts";
import { legacyzh_CN } from "./catalogue-legacy-zh-CN.ts";
import { legacypt_BR } from "./catalogue-legacy-pt-BR.ts";
import { legacyfr } from "./catalogue-legacy-fr.ts";
import type { VueLocale } from "./vue-category-labels.ts";
const legacy: Record<VueLocale, Record<string, readonly string[]>> = {
  en: legacyen,
  ja: legacyja,
  "zh-CN": legacyzh_CN,
  "pt-BR": legacypt_BR,
  fr: legacyfr,
};

export function retainedCategoryFragments(
  locale: VueLocale,
  file: string,
  ownedIds: ReadonlySet<string>,
) {
  return (legacy[locale][file] ?? [])
    .filter((id) => !ownedIds.has(id))
    .map((id) => `<span id="${id}"></span>`);
}
