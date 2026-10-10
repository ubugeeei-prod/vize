import {
  frenchPackets,
  frenchShared,
  frenchCategoryTitles,
  frenchCategoryIntro,
  frenchCrossFileIntro,
} from "./category-locales/fr-index.ts";
import {
  portuguesePackets,
  portugueseShared,
  portugueseCategoryTitles,
  portugueseCategoryIntro,
  portugueseCrossFileIntro,
} from "./category-locales/pt-index.ts";
import {
  chinesePackets,
  chineseShared,
  chineseCategoryTitles,
  chineseCategoryIntro,
  chineseCrossFileIntro,
} from "./category-locales/zh-index.ts";
import { commonVueText, vueCategoryLabels, type VueLocale } from "./vue-category-labels.ts";
import { localizeVueReference, translatedVuePurpose } from "./vue-category-translations.ts";

type TranslatedLocale = Exclude<VueLocale, "en" | "ja">;
interface CatalogueTranslation {
  packets: Readonly<Record<string, readonly [string, string, string]>>;
  shared: Readonly<Record<string, string>>;
  titles: Record<keyof typeof frenchCategoryTitles, string>;
  intro: string;
  crossIntro: string;
}
const locales: Record<TranslatedLocale, CatalogueTranslation> = {
  fr: {
    packets: frenchPackets,
    shared: frenchShared,
    titles: frenchCategoryTitles,
    intro: frenchCategoryIntro,
    crossIntro: frenchCrossFileIntro,
  },
  "pt-BR": {
    packets: portuguesePackets,
    shared: portugueseShared,
    titles: portugueseCategoryTitles,
    intro: portugueseCategoryIntro,
    crossIntro: portugueseCrossFileIntro,
  },
  "zh-CN": {
    packets: chinesePackets,
    shared: chineseShared,
    titles: chineseCategoryTitles,
    intro: chineseCategoryIntro,
    crossIntro: chineseCrossFileIntro,
  },
};
export const catalogueTranslation = (locale: TranslatedLocale) => locales[locale];

export function cataloguePurpose(
  reference: string,
  id: string,
  locale: VueLocale,
  project: boolean,
) {
  if (locale === "en" || locale === "ja") {
    const purpose = reference.match(/^# `[^`]+`\n\n([^\n]+)/m)?.[1];
    if (!purpose) throw new Error(`${id}: missing source purpose`);
    return purpose;
  }
  if (!project && id.startsWith("vue/")) return translatedVuePurpose(id, locale);
  const packet = locales[locale].packets[id];
  if (!packet) throw new Error(`${locale}: missing complete catalogue explanation for ${id}`);
  return packet[0];
}

/** Translate actual prose, leaving every code block, file path and diagnostic ID exact. */
export function localizeCatalogueReference(
  reference: string,
  id: string,
  locale: TranslatedLocale,
  project: boolean,
) {
  if (!project && id.startsWith("vue/")) return localizeVueReference(reference, id, locale);
  const { packets, shared } = locales[locale];
  const translated = packets[id];
  if (!translated) throw new Error(`${locale}: missing complete catalogue explanation for ${id}`);
  const authored = [
    reference.match(/^# `[^`]+`\n\n([^\n]+)/m)?.[1],
    reference.match(/^## Bad\n\n([^\n]+)/m)?.[1],
    reference.match(/^## Good\n\n([^\n]+)/m)?.[1],
  ];
  if (authored.some((text) => text === undefined))
    throw new Error(`${id}: incomplete authored reference`);
  const prose = new Map(authored.map((source, index) => [source, translated[index]]));
  const common = new Map([...commonVueText(locale), ...Object.entries(shared)]);
  const value = (text: string) => {
    const localized = common.get(text);
    if (localized) return localized;
    if (/^(?:`[^`]+`(?:, `[^`]+`)*|_none_|warning|error|info|information|hint)$/.test(text))
      return text;
    throw new Error(`${locale} ${id}: untranslated metadata ${text}`);
  };
  let inFence = false;
  return reference
    .split("\n")
    .map((line) => {
      if (line.startsWith("```")) {
        inFence = !inFence;
        return line;
      }
      if (
        inFence ||
        !line ||
        line === "---" ||
        line.startsWith("title:") ||
        line.startsWith("# ") ||
        line === "## Bad" ||
        line === "## Good" ||
        /^`[^`]+`$/.test(line)
      )
        return line;
      const localized = prose.get(line) ?? common.get(line);
      if (localized) return localized;
      if (line.startsWith("## ")) return `## ${value(line.slice(3))}`;
      const metadata = line.match(/^([^:]+): (.*?)(\s*)$/);
      if (metadata && common.has(metadata[1]))
        return `${common.get(metadata[1])}: ${value(metadata[2])}${metadata[3]}`;
      if (line === "[Bad](#bad) · [Good](#good)")
        return `[${vueCategoryLabels[locale].bad}](#bad) · [${vueCategoryLabels[locale].good}](#good)`;
      if (line.startsWith("["))
        return line.replace(
          /\[([^\]]+)\]\(([^)]+)\)/g,
          (_, label: string, target: string) => `[${value(label)}](${target})`,
        );
      throw new Error(`${locale} ${id}: untranslated prose ${line}`);
    })
    .join("\n");
}
