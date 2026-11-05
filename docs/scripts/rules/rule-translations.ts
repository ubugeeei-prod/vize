import { chinesePackets, chineseShared } from "./category-locales/zh-index.ts";
import { frenchPackets, frenchShared } from "./category-locales/fr-index.ts";
import { portuguesePackets, portugueseShared } from "./category-locales/pt-index.ts";
import { chineseVue0 } from "./vue-locales/zh-0.ts";
import { chineseVue1 } from "./vue-locales/zh-1.ts";
import { chineseVue2, chineseVueNotes } from "./vue-locales/zh-2.ts";
import { frenchVue0 } from "./vue-locales/fr-0.ts";
import { frenchVue1 } from "./vue-locales/fr-1.ts";
import { frenchVue2, frenchVueNotes } from "./vue-locales/fr-2.ts";
import { portugueseVue0 } from "./vue-locales/pt-0.ts";
import { portugueseVue1 } from "./vue-locales/pt-1.ts";
import { portugueseVue2, portugueseVueNotes } from "./vue-locales/pt-2.ts";
import { commonVueText } from "./rule-phrases.ts";
import type { RuleLocale } from "./rule-groups.ts";

export type TranslatedLocale = Exclude<RuleLocale, "en" | "ja">;
type Packet = readonly [purpose: string, bad: string, good: string];

const vuePackets: Record<TranslatedLocale, Record<string, Packet>> = {
  "zh-CN": { ...chineseVue0, ...chineseVue1, ...chineseVue2 },
  "pt-BR": { ...portugueseVue0, ...portugueseVue1, ...portugueseVue2 },
  fr: { ...frenchVue0, ...frenchVue1, ...frenchVue2 },
};
const otherPackets: Record<TranslatedLocale, Record<string, Packet>> = {
  "zh-CN": chinesePackets,
  "pt-BR": portuguesePackets,
  fr: frenchPackets,
};
const phrases: Record<TranslatedLocale, Map<string, string>> = {
  "zh-CN": new Map([
    ...commonVueText("zh-CN"),
    ...Object.entries(chineseShared),
    ...Object.entries(chineseVueNotes),
  ]),
  "pt-BR": new Map([
    ...commonVueText("pt-BR"),
    ...Object.entries(portugueseShared),
    ...Object.entries(portugueseVueNotes),
  ]),
  fr: new Map([
    ...commonVueText("fr"),
    ...Object.entries(frenchShared),
    ...Object.entries(frenchVueNotes),
  ]),
};

/** Translated prose for one rule. Missing text fails generation instead of shipping English. */
export function ruleTranslator(id: string, locale: TranslatedLocale, project: boolean) {
  const packet = (!project && id.startsWith("vue/") ? vuePackets : otherPackets)[locale][id];
  if (!packet) throw new Error(`${locale}: missing translated explanation for ${id}`);
  const known = phrases[locale];
  const line = (text: string) => {
    const translated = known.get(text);
    if (translated === undefined) throw new Error(`${locale} ${id}: untranslated text: ${text}`);
    return translated;
  };
  /** Translate prose lines; fenced code, headings' code and blank lines stay exact. */
  const block = (text: string) => {
    let fence = false;
    return text
      .split("\n")
      .map((source) => {
        if (source.startsWith("```")) {
          fence = !fence;
          return source;
        }
        if (fence || !source.trim() || /^`[^`]+`$/.test(source)) return source;
        if (known.has(source)) return line(source);
        const heading = source.match(/^(#+ )(.+)$/);
        return heading ? `${heading[1]}${line(heading[2])}` : line(source);
      })
      .join("\n");
  };
  return { purpose: packet[0], bad: packet[1], good: packet[2], line, block };
}

export function hasTranslatedPhrase(locale: TranslatedLocale, text: string) {
  return phrases[locale].has(text);
}
