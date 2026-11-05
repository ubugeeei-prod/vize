import { existsSync } from "node:fs";
import { posix, resolve } from "node:path";
import { ruleGroups, type RuleLocale } from "./rule-groups.ts";

const generatedRoutes = new RegExp(
  `^rules/(?:reference/|project/|(?:all|cross-file|${ruleGroups.join("|")})\\.md$)`,
);

/**
 * A localized page links to its own locale when that page exists and to the
 * English page otherwise, so a smaller locale never publishes a broken link.
 */
export function localizeLinks(root: string, locale: RuleLocale, page: string, content: string) {
  if (locale === "en") return content;
  const content_ = resolve(root, "docs/content");
  let fence = false;
  return content
    .split("\n")
    .map((line) => {
      if (/^(`{3,}|~{3,})/.test(line)) fence = !fence;
      if (fence || line.startsWith("```")) return line;
      return line.replace(/\]\(([^)#:\s]+\.md)(#[^)]*)?\)/g, (match, target: string, hash = "") => {
        if (target.startsWith("/")) return match;
        const path = posix.normalize(posix.join(posix.dirname(page), target));
        if (generatedRoutes.test(path) || existsSync(resolve(content_, locale, path))) return match;
        if (!existsSync(resolve(content_, path)))
          throw new Error(`${locale}/${page}: link target missing in every locale: ${target}`);
        return `](/${path}${hash})`;
      });
    })
    .join("\n");
}
