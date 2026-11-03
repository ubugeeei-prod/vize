import { frenchPackets, frenchShared } from "./category-locales/fr-index.ts";
import { portuguesePackets, portugueseShared } from "./category-locales/pt-index.ts";
import { chinesePackets, chineseShared } from "./category-locales/zh-index.ts";
import { commonVueText } from "./rule-phrases.ts";
import { ruleHeadings } from "./rule-page.ts";
import type { TranslatedLocale } from "./rule-translations.ts";

const locales: Record<
  TranslatedLocale,
  {
    packets: Readonly<Record<string, readonly [string, string, string]>>;
    shared: Readonly<Record<string, string>>;
  }
> = {
  fr: { packets: frenchPackets, shared: frenchShared },
  "pt-BR": { packets: portuguesePackets, shared: portugueseShared },
  "zh-CN": { packets: chinesePackets, shared: chineseShared },
};

/** Translate a project rule page's prose, leaving code blocks, paths and IDs exact. */
export function localizeCatalogueReference(
  reference: string,
  id: string,
  locale: TranslatedLocale,
  _project: true,
) {
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
  const labels = ruleHeadings[locale];
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
        /^`[^`]+`$/.test(line)
      )
        return line;
      if (line === "## Bad") return `## ${labels.bad}`;
      if (line === "## Good") return `## ${labels.good}`;
      const localized = prose.get(line) ?? common.get(line);
      if (localized) return localized;
      if (line.startsWith("## ")) return `## ${value(line.slice(3))}`;
      const metadata = line.match(/^([^:]+): (.*?)(\s*)$/);
      if (metadata && common.has(metadata[1]))
        return `${common.get(metadata[1])}: ${value(metadata[2])}${metadata[3]}`;
      if (line.startsWith("["))
        return line.replace(
          /\[([^\]]+)\]\(([^)]+)\)/g,
          (_, label: string, target: string) => `[${value(label)}](${target})`,
        );
      throw new Error(`${locale} ${id}: untranslated prose ${line}`);
    })
    .join("\n");
}
