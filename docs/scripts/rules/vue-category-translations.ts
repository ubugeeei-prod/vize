import { frenchVue0 } from "./vue-locales/fr-0.ts";
import { frenchVue1 } from "./vue-locales/fr-1.ts";
import { frenchVue2, frenchVueNotes } from "./vue-locales/fr-2.ts";
import { portugueseVue0 } from "./vue-locales/pt-0.ts";
import { portugueseVue1 } from "./vue-locales/pt-1.ts";
import { portugueseVue2, portugueseVueNotes } from "./vue-locales/pt-2.ts";
import { chineseVue0 } from "./vue-locales/zh-0.ts";
import { chineseVue1 } from "./vue-locales/zh-1.ts";
import { chineseVue2, chineseVueNotes } from "./vue-locales/zh-2.ts";
import { commonVueText, type VueLocale, vueCategoryLabels } from "./vue-category-labels.ts";

type TranslatedLocale = Exclude<VueLocale, "en" | "ja">;
const packets: Record<TranslatedLocale, Record<string, readonly [string, string, string]>> = {
  fr: { ...frenchVue0, ...frenchVue1, ...frenchVue2 },
  "pt-BR": { ...portugueseVue0, ...portugueseVue1, ...portugueseVue2 },
  "zh-CN": { ...chineseVue0, ...chineseVue1, ...chineseVue2 },
};
const notes: Record<TranslatedLocale, Record<string, string>> = {
  fr: frenchVueNotes,
  "pt-BR": portugueseVueNotes,
  "zh-CN": chineseVueNotes,
};

/** Fail rather than publish incomplete translated explanations for a new rule. */
export function translatedVuePurpose(id: string, locale: TranslatedLocale) {
  const packet = packets[locale][id];
  if (!packet) throw new Error(`${locale}: missing complete Vue explanation for ${id}`);
  return packet[0];
}

export function localizeVueReference(reference: string, id: string, locale: TranslatedLocale) {
  const translated = packets[locale][id];
  if (!translated) throw new Error(`${locale}: missing complete Vue explanation for ${id}`);
  const authored = [
    reference.match(/^# `[^`]+`\n\n([^\n]+)/m)?.[1],
    reference.match(/^## Bad\n\n([^\n]+)/m)?.[1],
    reference.match(/^## Good\n\n([^\n]+)/m)?.[1],
  ];
  if (authored.some((text) => text === undefined)) throw new Error(`${id}: incomplete reference`);
  const prose = new Map(authored.map((source, index) => [source, translated[index]]));
  const common = commonVueText(locale);
  const labels = vueCategoryLabels[locale];
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
      const translatedProse = prose.get(line) ?? notes[locale][line] ?? common.get(line);
      if (translatedProse) return translatedProse;
      if (line.startsWith("## ")) {
        const title = common.get(line.slice(3));
        if (title) return `## ${title}`;
      }
      const metadata = line.match(/^([^:]+): (.*?)(\s*)$/);
      if (metadata && common.has(metadata[1])) {
        const [, key, value, spacing] = metadata;
        const translatedValue = common.get(value) ?? value;
        if (!common.has(value) && !/^(?:`[^`]+`(?:, `[^`]+`)*|_none_)$/.test(value))
          throw new Error(`${locale} ${id}: untranslated metadata ${value}`);
        return `${common.get(key)}: ${translatedValue}${spacing}`;
      }
      if (line === "[Bad](#bad) · [Good](#good)")
        return `[${labels.bad}](#bad) · [${labels.good}](#good)`;
      if (line.startsWith("[Implementation]("))
        return line
          .replace("[Implementation]", `[${common.get("Implementation")}]`)
          .replace("[All rules]", `[${common.get("All rules")}]`);
      throw new Error(`${locale} ${id}: untranslated prose ${line}`);
    })
    .join("\n");
}

export function localizeVueExampleLabels(inline: string, locale: VueLocale) {
  if (locale === "en" || locale === "ja") return inline;
  const labels = vueCategoryLabels[locale];
  let inFence = false;
  return inline
    .split("\n")
    .map((line) => {
      if (line.startsWith("```")) {
        inFence = !inFence;
        return line;
      }
      if (inFence) return line;
      if (line === "**Bad**") return `**${labels.bad}**`;
      if (line === "**Good**") return `**${labels.good}**`;
      return line;
    })
    .join("\n");
}
