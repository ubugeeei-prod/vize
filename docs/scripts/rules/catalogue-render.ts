import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import type { CrossRuleMetadata, RuleMetadata } from "./types.ts";
import { nativeInlineReference } from "./annotate-reference.ts";
import {
  categoryFiles,
  categoryIncludes,
  englishCategoryTitles,
  japaneseCategoryTitles,
} from "./catalogue-routes.ts";
import { catalogueSubgroups } from "./catalogue-subgroups.ts";
import {
  accessibilityIntroductions,
  japaneseAccessibilityIntroductions,
} from "./accessibility-introductions.ts";
import {
  actualPrerequisiteLinks,
  catalogueCell,
  ruleSlug,
  writeCatalogue,
} from "./catalogue-page.ts";
import {
  cataloguePurpose,
  catalogueTranslation,
  localizeCatalogueReference,
} from "./catalogue-translations.ts";
import { retainedCategoryFragments } from "./catalogue-legacy.ts";
import {
  commonVueText,
  vueCategoryLabels,
  vueLocales,
  type VueLocale,
} from "./vue-category-labels.ts";
import { localizeVueExampleLabels } from "./vue-category-translations.ts";
import { crossMetadata } from "./project-metadata.ts";
import {
  overviewIntroduction,
  overviewProjectIntroduction,
  ruleOverview,
} from "./all-rules-overview.ts";

interface Reference {
  id: string;
  en: string;
  ja: string;
  project: boolean;
}
const intro = {
  en: "Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show what changes; copying an example gives you the complete source. Each rule also states its current support limits.",
  ja: "このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。",
};
const crossIntro = {
  en: "This page shows all 66 project examples with their complete Bad and Good versions and the files both versions share. Use the shared files with both. Of the 60 diagnostic codes, the CLI reports 19 today (18 verified against source examples and one shown with an illustrative project), 16 exist only in the experimental Rust analyzer and are not reported individually by the CLI, and 25 are specified but nothing produces them yet. Configuring an ID does not turn on a check that is not available.",
  ja: "全 66 件のプロジェクトの例について、共通ファイルと、省略のない悪い例・良い例をこのページにまとめています。共通ファイルは両方で使用します。解析器の 60 件の診断コードには、CLI の 19 件（検証済みのソース例 18 件と、参照関係を保持したリアクティブなグラフを含む説明用プロジェクト 1 件）、CLI が個別には生成しない実験的な Rust 解析器の 16 件、現在は診断を生成する処理のない契約 25 件があります。ID を設定しても、未対応の診断処理は有効になりません。",
};
const cliInstructions = {
  en: "The public CLI exposes the same pass with `vize lint --cross-file`. Displayed `vize:croquis/cf/*` codes use `croquis/cf/*` in `vue.linter.rules` (omit `vize:`). Information/hint diagnostics become CLI warnings. Related locations explain the source/consumer relationship.",
  ja: "公開 CLI では `vize lint --cross-file` で同じ処理を使用できます。表示される `vize:croquis/cf/*` コードは、`vue.linter.rules` では `croquis/cf/*` と設定します（`vize:` を省略）。information/hint の診断は CLI では warning になります。関連箇所は生成元と利用側の関係を説明します。",
};
const supportLabels = {
  cli: "CLI",
  library: "Experimental Rust analyzer; not an individual CLI code",
  contract: "Contract; no current producer",
  project: "Project-specific lint ID",
};
const japaneseSupportLabels = {
  cli: "CLI",
  library: "実験的な Rust 解析器（CLI の個別コードではありません）",
  contract: "契約（現在の生成元なし）",
  project: "プロジェクト固有の lint ID",
};
const titleFragment = (title: string) =>
  title
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, "-")
    .replace(/^-+|-+$/g, "");

function supportText(locale: VueLocale, status: CrossRuleMetadata["status"] | "project") {
  if (locale === "ja") return japaneseSupportLabels[status];
  const text = supportLabels[status];
  if (locale === "en" || status === "cli") return text;
  const translated = catalogueTranslation(locale).shared[text];
  if (!translated) throw new Error(`${locale}: missing catalogue support label ${text}`);
  return translated;
}

export function catalogueReferences(root: string) {
  const references = new Map<string, Reference>();
  for (const section of ["reference", "project"]) {
    for (const file of readdirSync(resolve(root, `docs/content/rules/${section}`)).sort()) {
      const read = (prefix: string) =>
        readFileSync(resolve(root, `docs/content/${prefix}rules/${section}/${file}`), "utf8");
      const en = read("");
      const id = en.match(/^# `([^`]+)`/m)?.[1];
      if (!id || references.has(id))
        throw new Error(`Ambiguous catalogue source ${section}/${file}`);
      references.set(id, { id, en, ja: read("ja/"), project: section === "project" });
    }
  }
  if (references.size !== 332)
    throw new Error("Catalogue must retain 266 source rules and 66 project packets");
  return references;
}

export function generateRemainingCatalogues(
  root: string,
  rules: readonly RuleMetadata[],
  checking: boolean,
) {
  const references = catalogueReferences(root);
  const analyzerStatus = new Map(crossMetadata(root).map((rule) => [rule.code, rule.status]));
  const cache = new Map<string, string>();
  const packet = (reference: Reference, locale: VueLocale) => {
    const key = `${locale}:${reference.id}`;
    const cached = cache.get(key);
    if (cached !== undefined) return cached;
    const authored = locale === "ja" ? reference.ja : reference.en;
    const translated =
      locale === "en" || locale === "ja"
        ? authored
        : localizeCatalogueReference(authored, reference.id, locale, reference.project);
    const result = actualPrerequisiteLinks(
      root,
      locale,
      localizeVueExampleLabels(
        nativeInlineReference(translated, reference.id, reference.project),
        locale,
      ),
    );
    cache.set(key, result);
    return result;
  };
  const page = (
    locale: VueLocale,
    file: string,
    title: string,
    selected: readonly Reference[],
    originalTitle?: string,
  ) => {
    const accessibilityGroup = file.match(
      /^accessibility-(core|integrity|interactions|structure)$/u,
    )?.[1];
    const groupIntroduction = accessibilityGroup
      ? locale === "en"
        ? accessibilityIntroductions[accessibilityGroup]
        : locale === "ja"
          ? japaneseAccessibilityIntroductions[accessibilityGroup]
          : undefined
      : locale === "ja"
        ? catalogueSubgroups[file]?.jaIntroduction
        : undefined;
    if (locale === "ja" && accessibilityGroup && !groupIntroduction)
      throw new Error(`Missing Japanese accessibility introduction: ${accessibilityGroup}`);
    const labels = vueCategoryLabels[locale];
    const text =
      file === "all"
        ? overviewIntroduction[locale]
        : locale === "en" || locale === "ja"
          ? intro[locale]
          : catalogueTranslation(locale).intro;
    const projectText =
      locale === "en" || locale === "ja"
        ? crossIntro[locale]
        : catalogueTranslation(locale).crossIntro;
    const cliText =
      locale === "en" || locale === "ja"
        ? cliInstructions[locale]
        : catalogueTranslation(locale).shared[cliInstructions.en];
    if (!cliText) throw new Error(`${locale}: missing catalogue CLI instructions`);
    const withStatus = file === "cross-file";
    const supportHeading =
      locale === "en"
        ? "Current support"
        : locale === "ja"
          ? "現在の対応"
          : commonVueText(locale).get("Current support");
    const entries = [...selected].sort((left, right) => left.id.localeCompare(right.id));
    const owned = new Set(
      entries.flatMap(({ id }) => [ruleSlug(id), `${ruleSlug(id)}-bad`, `${ruleSlug(id)}-good`]),
    );
    // The title fragment is emitted by native headings, not a second explicit span.
    const titleId = titleFragment(title);
    owned.add(titleId);
    const originalTitleId = originalTitle && titleFragment(originalTitle);
    if (originalTitleId) owned.add(originalTitleId);
    const lines = [
      "---",
      `title: ${JSON.stringify(title)}`,
      "---",
      "",
      ...(originalTitleId && originalTitleId !== titleId
        ? [`<span id="${originalTitleId}"></span>`, ""]
        : []),
      `# ${title}`,
      "",
      ...(groupIntroduction ? [...groupIntroduction, ""] : []),
      text,
      "",
      ...(entries.some((entry) => entry.project)
        ? [file === "all" ? overviewProjectIntroduction[locale] : projectText, "", cliText, ""]
        : []),
      ...retainedCategoryFragments(locale, file, owned),
      "",
      `| ${labels.rule} | ${labels.examples} | ${labels.purpose}${withStatus ? ` | ${supportHeading}` : ""} |`,
      `| --- | --- | ---${withStatus ? " | ---" : ""} |`,
      ...entries.map((entry) => {
        const id = ruleSlug(entry.id);
        const purpose = cataloguePurpose(
          locale === "ja" ? entry.ja : entry.en,
          entry.id,
          locale,
          entry.project,
        );
        const status = withStatus
          ? ` | ${supportText(locale, analyzerStatus.get(entry.id) ?? "project")}`
          : "";
        return `| [\`${entry.id}\`](#${id}) | [${labels.bad}](#${id}-bad) · [${labels.good}](#${id}-good) | ${catalogueCell(purpose)}${status} |`;
      }),
      "",
      actualPrerequisiteLinks(root, locale, labels.related),
      "",
    ];
    writeCatalogue(
      root,
      locale,
      file,
      lines.join("\n"),
      entries.map((entry) =>
        file === "all"
          ? ruleOverview(locale === "ja" ? entry.ja : entry.en, entry.id, locale, entry.project)
          : packet(entry, locale),
      ),
      checking,
    );
  };
  for (const locale of vueLocales) {
    const titles =
      locale === "en"
        ? englishCategoryTitles
        : locale === "ja"
          ? japaneseCategoryTitles
          : catalogueTranslation(locale).titles;
    for (const file of categoryFiles) {
      const selected = rules
        .filter((rule) => categoryIncludes(file, rule.name))
        .map((rule) => references.get(rule.name)!);
      page(locale, file, titles[file], selected);
    }
    page(
      locale,
      "cross-file",
      titles["cross-file"],
      [...references.values()].filter((entry) => entry.project),
    );
    if (locale !== "en" && locale !== "ja")
      page(locale, "all", titles.all, [...references.values()]);
  }
  for (const [file, subgroup] of Object.entries(catalogueSubgroups)) {
    const selected = subgroup.rules.map((id) => references.get(id)!);
    page("en", file, subgroup.title, selected);
    if (subgroup.jaTitle) page("ja", file, subgroup.jaTitle, selected, subgroup.title);
  }
}
