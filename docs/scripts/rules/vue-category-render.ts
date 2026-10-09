import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import type { RuleMetadata } from "./types.ts";
import { inlineReference } from "./inline-reference.ts";
import { annotateExampleChanges } from "./inline-code-diff.ts";
import {
  vueCategoryLabels,
  vueCategoryLegacyAnchors,
  vueLocales,
  type VueLocale,
} from "./vue-category-labels.ts";
import {
  localizeVueExampleLabels,
  localizeVueReference,
  translatedVuePurpose,
} from "./vue-category-translations.ts";

const slug = (id: string) => id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
const cell = (value: string) =>
  value
    .replaceAll("|", "\\|")
    .replace(/(`+)([\s\S]*?)\1|[<>]/g, (token) =>
      token === "<" ? "&lt;" : token === ">" ? "&gt;" : token,
    );

function referencePurpose(reference: string) {
  const purpose = reference.match(/^# `[^`]+`\n\n([^\n]+)/m)?.[1];
  if (!purpose) throw new Error("Vue reference requires its actual purpose");
  return purpose;
}

function exampleSource(reference: string, heading: string) {
  const section = reference.split(`## ${heading}\n`)[1]?.split("\n## ")[0];
  const source = section?.match(/```(?:vue|ts|html)\n([\s\S]*?)\n```/)?.[1];
  if (source === undefined) throw new Error(`Vue reference requires complete ${heading} source`);
  return source;
}

/** A missing localized prerequisite links to its real English page, never a 404. */
function supportedLinks(root: string, locale: VueLocale, inline: string) {
  const prefix = locale === "en" ? "" : `${locale}/`;
  const directory = resolve(root, `docs/content/${prefix}rules`);
  let inFence = false;
  return inline
    .split("\n")
    .map((line) => {
      if (line.startsWith("```")) {
        inFence = !inFence;
        return line;
      }
      if (inFence) return line;
      return line.replace(/\]\(([^)]+\.md)(#[^)]*)?\)/g, (match, target: string, fragment = "") => {
        if (
          target.startsWith("/") ||
          /^[a-z]+:/i.test(target) ||
          existsSync(resolve(directory, target))
        )
          return match;
        const englishDirectory = resolve(root, "docs/content/rules");
        const englishTarget = resolve(englishDirectory, target);
        if (!existsSync(englishTarget))
          throw new Error(`Missing Vue prerequisite ${locale}: ${target}`);
        const content = resolve(root, "docs/content");
        return `](/${englishTarget.slice(content.length + 1)}${fragment})`;
      });
    })
    .join("\n");
}

export function generateVueCategoryPages(
  root: string,
  rules: readonly RuleMetadata[],
  checking: boolean,
) {
  const selected = rules
    .filter((rule) => rule.name.startsWith("vue/"))
    .sort((left, right) => left.name.localeCompare(right.name));
  const references = new Map(
    selected.map((rule) => {
      const file = `${slug(rule.name)}.md`;
      const en = readFileSync(resolve(root, "docs/content/rules/reference", file), "utf8");
      const ja = readFileSync(resolve(root, "docs/content/ja/rules/reference", file), "utf8");
      return [
        rule.name,
        { en, ja, bad: exampleSource(en, "Bad"), good: exampleSource(en, "Good") },
      ];
    }),
  );
  for (const locale of vueLocales) {
    const labels = vueCategoryLabels[locale];
    const entries = selected.map((rule) => {
      const reference = references.get(rule.name)!;
      const authored = locale === "ja" ? reference.ja : reference.en;
      const localized =
        locale === "en" || locale === "ja"
          ? authored
          : localizeVueReference(authored, rule.name, locale);
      const annotated = annotateExampleChanges(localized, reference.bad, reference.good);
      const inline = supportedLinks(
        root,
        locale,
        localizeVueExampleLabels(inlineReference(annotated, rule.name), locale),
      ).replace(/^<span id="[^"\n]+"><\/span>\n\n/, "");
      // Native heading IDs provide the rule anchor without a duplicate span ID.
      return {
        id: slug(rule.name),
        name: rule.name,
        inline,
        purpose:
          locale === "en" || locale === "ja"
            ? referencePurpose(authored)
            : translatedVuePurpose(rule.name, locale),
      };
    });
    const lines = [
      "---",
      `title: ${labels.title}`,
      "---",
      "",
      `# ${labels.title}`,
      "",
      labels.intro,
      "",
      ...(vueCategoryLegacyAnchors[locale] ?? []).map((id) => `<span id="${id}"></span>`),
      "",
      `| ${labels.rule} | ${labels.examples} | ${labels.purpose} |`,
      "| --- | --- | --- |",
      ...entries.map(
        ({ id, name, purpose }) =>
          `| [\`${name}\`](#${id}) | [${labels.bad}](#${id}-bad) · [${labels.good}](#${id}-good) | ${cell(purpose)} |`,
      ),
      "",
      supportedLinks(root, locale, labels.related),
    ];
    const prefix = locale === "en" ? "" : `${locale}/`;
    const path = resolve(root, `docs/content/${prefix}rules/vue.md`);
    const publicRoute = `https://vizejs.dev/${prefix}rules/vue.html`;
    const index = `${lines.map((line) => line.replaceAll("](#", `](${publicRoute}#`)).join("\n")}\n`;
    const text = `${[...lines, "", ...entries.map(({ inline }) => inline)].join("\n").trimEnd()}\n`;
    const generated = resolve(root, `docs/content/generated/rules/${locale}`);
    if (checking) {
      if (readFileSync(path, "utf8") !== index)
        throw new Error(`Stale inline Vue category: ${path}`);
      if (readFileSync(resolve(generated, "vue.md"), "utf8") !== text)
        throw new Error(`Stale generated inline Vue category: ${locale}`);
    } else {
      writeFileSync(path, index);
      mkdirSync(generated, { recursive: true });
      writeFileSync(resolve(generated, "vue.md"), text);
    }
  }
}
