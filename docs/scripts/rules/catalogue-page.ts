import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import type { VueLocale } from "./vue-category-labels.ts";

export const ruleSlug = (id: string) => id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
export const catalogueCell = (value: string) =>
  value
    .replaceAll("|", "\\|")
    .replace(/(`+)([\s\S]*?)\1|[<>]/g, (token) =>
      token === "<" ? "&lt;" : token === ">" ? "&gt;" : token,
    );

export function actualPrerequisiteLinks(root: string, locale: VueLocale, inline: string) {
  const directory = resolve(root, `docs/content/${locale === "en" ? "" : `${locale}/`}rules`);
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
        const english = resolve(root, "docs/content/rules", target);
        if (!existsSync(english))
          throw new Error(`Missing catalogue prerequisite ${locale}: ${target}`);
        return `](/${english.slice(resolve(root, "docs/content").length + 1)}${fragment})`;
      });
    })
    .join("\n");
}

export function writeCatalogue(
  root: string,
  locale: VueLocale,
  file: string,
  index: string,
  examples: readonly string[],
  checking: boolean,
) {
  const prefix = locale === "en" ? "" : `${locale}/`;
  const source = resolve(root, `docs/content/${prefix}rules/${file}.md`);
  const derivative = resolve(root, `docs/content/generated/rules/${locale}/${file}.md`);
  const sourceText = `${index.replaceAll("](#", `](https://vizejs.dev/${prefix}rules/${file}.html#`).trimEnd()}\n`;
  const fullText = `${[index.trimEnd(), "", ...examples].join("\n").trimEnd()}\n`;
  for (const [path, text] of [
    [source, sourceText],
    [derivative, fullText],
  ]) {
    if (checking) {
      if (readFileSync(path, "utf8") !== text) throw new Error(`Stale catalogue ${path}`);
    } else {
      mkdirSync(resolve(path, ".."), { recursive: true });
      writeFileSync(path, text);
    }
  }
}
