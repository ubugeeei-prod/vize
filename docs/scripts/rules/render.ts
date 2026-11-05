import type { RulePageOptions } from "./types.ts";
import { resolve } from "node:path";
import { generateProjectPages } from "./project-render.ts";
import { ruleExamples } from "./examples.ts";
import { validateExampleExplanations } from "./explanations.ts";
import { migrationPage } from "./migration.ts";
import { purposeJa } from "./purpose-ja.ts";
import { crossMetadata } from "./project-metadata.ts";
import { localizeCatalogueReference } from "./catalogue-translations.ts";
import { ruleLocales, type RuleLocale } from "./rule-groups.ts";
import { renderRulePage, ruleSlug } from "./rule-page.ts";
import { ruleTranslator } from "./rule-translations.ts";
import { generateRuleIndexes, writeOutput, type IndexedRule } from "./rule-index.ts";
import { localizeLinks } from "./localized-links.ts";

const prefix = (locale: RuleLocale) => (locale === "en" ? "" : `${locale}/`);
const sourcePurpose = (page: string) => {
  const purpose = page.match(/^# `[^`]+`\n\n([^\n]+)/m)?.[1];
  if (!purpose) throw new Error("Rule page requires its purpose sentence");
  return purpose;
};

export function generateRulePages({ workspaceRoot: root, rules }: RulePageOptions) {
  const checking = process.argv.includes("--check");
  validateExampleExplanations(rules);
  const indexed: IndexedRule[] = [];
  const page = (locale: RuleLocale, path: string, content: string) =>
    writeOutput(
      resolve(root, `docs/content/${prefix(locale)}rules`, path),
      localizeLinks(root, locale, `rules/${path}`, content),
      checking,
    );

  for (const rule of rules) {
    if (!purposeJa[rule.name]) throw new Error(`Missing Japanese purpose for ${rule.name}`);
    const example = ruleExamples(root, rule);
    const purpose = {} as Record<RuleLocale, string>;
    for (const locale of ruleLocales) {
      const text = renderRulePage(rule, example, locale);
      purpose[locale] = sourcePurpose(text);
      page(locale, `reference/${ruleSlug(rule.name)}.md`, text);
    }
    indexed.push({
      id: rule.name,
      project: false,
      fixable: rule.fixable,
      presets: rule.presets,
      purpose,
    });
  }

  const status = new Map(crossMetadata(root).map((rule) => [rule.code, rule.status]));
  const projectPages = generateProjectPages(root, checking);
  const japanese = new Map(projectPages.get("ja/")!.map(({ id, text }) => [id, text]));
  for (const { id, text } of projectPages.get("")!) {
    const purpose = { en: sourcePurpose(text), ja: sourcePurpose(japanese.get(id)!) } as Record<
      RuleLocale,
      string
    >;
    for (const locale of ["zh-CN", "pt-BR", "fr"] as const) {
      purpose[locale] = ruleTranslator(id, locale, true).purpose;
      page(
        locale,
        `project/${ruleSlug(id)}.md`,
        localizeCatalogueReference(text, id, locale, true),
      );
    }
    indexed.push({
      id,
      project: true,
      fixable: false,
      presets: [],
      status: status.get(id) ?? "cli",
      purpose,
    });
  }

  const names = new Set(rules.map((rule) => rule.name));
  for (const locale of ["en", "ja"] as const)
    page(locale, "migration.md", migrationPage(root, names, locale === "ja"));
  generateRuleIndexes(root, indexed, checking, page);
}
