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
import { accessibilityIntroductions } from "./accessibility-introductions.ts";
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

interface Reference {
  id: string;
  en: string;
  ja: string;
  project: boolean;
}
const intro = {
  en: "Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.",
  ja: "このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。",
};
const crossIntro = {
  en: "All 66 project packets include shared files and complete Bad/Good examples here. Apply shared files to both sides. The 60 analyzer codes retain their actual boundaries: 19 CLI codes (18 qualified source pairs and one illustrative project with its retained reactive graph), 16 experimental Rust analyzer codes not emitted individually by that CLI pass, and 25 contracts without a current producer. Configuring an ID does not enable an unavailable producer.",
  ja: "全 66 のプロジェクトの例について、共通ファイルと完全な悪い例・良い例をこのページにまとめています。共通ファイルは両方で使用します。60 の analyzer コードの対応範囲は、CLI の 19 コード（18 の検証済みソースの例と、保持された参照構成を示す 1 つの具体例）、CLI が個別には生成しない実験的な Rust analyzer の 16 コード、現在の生成元がない 25 の契約です。ID を設定しても未対応の生成元は有効になりません。",
};
const cliInstructions = {
  en: "The public CLI exposes the same pass with `vize lint --cross-file`. Displayed `vize:croquis/cf/*` codes use `croquis/cf/*` in `lint.vize.rules` (omit `vize:`). Information/hint diagnostics become CLI warnings. Related locations explain the source/consumer relationship.",
  ja: "公開 CLI では `vize lint --cross-file` で同じ処理を使用できます。表示される `vize:croquis/cf/*` コードは、`lint.vize.rules` では `croquis/cf/*` と設定します（`vize:` を省略）。information/hint の診断は CLI では warning になります。関連箇所は生成元と利用側の関係を説明します。",
};
const supportLabels = {
  cli: "CLI",
  library: "Experimental Rust analyzer; not an individual CLI code",
  contract: "Contract; no current producer",
  project: "Project-specific lint ID",
};
const japaneseSupportLabels = {
  cli: "CLI",
  library: "実験的な Rust analyzer（CLI の個別コードではありません）",
  contract: "契約（現在の生成元なし）",
  project: "プロジェクト固有の lint ID",
};

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
  if (references.size !== 317)
    throw new Error("Catalogue must retain 251 source rules and 66 project packets");
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
  const page = (locale: VueLocale, file: string, title: string, selected: readonly Reference[]) => {
    const accessibilityGroup = file.match(
      /^accessibility-(core|integrity|interactions|structure)$/u,
    )?.[1];
    const labels = vueCategoryLabels[locale];
    const text =
      locale === "en" || locale === "ja" ? intro[locale] : catalogueTranslation(locale).intro;
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
    const titleId = title
      .toLowerCase()
      .replace(/[^\p{L}\p{N}]+/gu, "-")
      .replace(/^-+|-+$/g, "");
    owned.add(titleId);
    const lines = [
      "---",
      `title: ${JSON.stringify(title)}`,
      "---",
      "",
      `# ${title}`,
      "",
      ...(locale === "en" && accessibilityGroup
        ? [...accessibilityIntroductions[accessibilityGroup], ""]
        : []),
      text,
      "",
      ...(entries.some((entry) => entry.project) ? [projectText, "", cliText, ""] : []),
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
      entries.map((entry) => packet(entry, locale)),
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
    page(
      "en",
      file,
      subgroup.title,
      subgroup.rules.map((id) => references.get(id)!),
    );
  }
}
