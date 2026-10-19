import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "playwright";
import { owningRuleGroup } from "./rules/all-rules-overview.ts";

const root = resolve(import.meta.dirname, "../..");

/** All Rules is an index; complete examples belong to its unchanged group/reference pages. */
export function expectedRuleOverview(route: string) {
  const prefix = route.match(/^\/(ja\/|zh-CN\/|pt-BR\/|fr\/)?rules\/all$/)?.[1] ?? "";
  const locale = prefix.slice(0, -1) || "en";
  const source = readFileSync(
    resolve(root, `docs/content/generated/rules/${locale}/all.md`),
    "utf8",
  );
  const projects = new Set(
    readdirSync(resolve(root, "docs/content/rules/project")).map((file) => {
      const source = readFileSync(resolve(root, "docs/content/rules/project", file), "utf8");
      const id = source.match(/^# `([^`]+)`$/m)?.[1];
      assert(id, `${file}: project source identity`);
      return id;
    }),
  );
  const expected = [...source.matchAll(/^### `([^`]+)`$/gm)].map((match) => ({
    name: match[1],
    id: match[1].replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase(),
    group: owningRuleGroup(match[1], projects.has(match[1])),
  }));
  assert.equal(expected.length, 324, "overview retains all 324 rule headings");
  return { prefix, locale, expected };
}

export async function verifyRenderedRuleOverview(page: Page, route: string) {
  const { prefix, locale, expected } = expectedRuleOverview(route);
  const actual = await page.evaluate(
    ({ expected, prefix }) => {
      const content = document.querySelector(".content");
      if (!content) throw new Error("Rule overview requires its actual content authority");
      const ids = [...content.querySelectorAll("[id]")].map((element) => element.id);
      if (new Set(ids).size !== ids.length) throw new Error("Rule overview contains duplicate IDs");
      if (content.querySelectorAll("pre code").length)
        throw new Error("All Rules must not collect complete example code packets");
      const headings = [...content.querySelectorAll("h3[id]")];
      if (headings.length !== expected.length) throw new Error("Rule overview heading coverage");
      return expected.map(({ name, id, group }) => {
        const heading = document.getElementById(id);
        if (
          !heading ||
          heading.tagName !== "H3" ||
          heading.textContent?.trim() !== name ||
          !heading.checkVisibility()
        )
          throw new Error(`Missing overview rule heading ${name}`);
        const summary = heading.nextElementSibling;
        if (
          !summary ||
          summary.tagName !== "P" ||
          !summary.textContent?.trim() ||
          !summary.checkVisibility()
        )
          throw new Error(`Missing overview purpose ${name}`);
        const links: HTMLAnchorElement[] = [];
        for (let element = summary; element && element.tagName !== "H3";) {
          links.push(...element.querySelectorAll<HTMLAnchorElement>("a[href]"));
          const next = element.nextElementSibling;
          if (!next || /^H[1-3]$/.test(next.tagName)) break;
          element = next;
        }
        const destination = `/${prefix}rules/${group}/index.html`;
        for (const kind of ["bad", "good"]) {
          const alias = document.getElementById(`${id}-${kind}`);
          if (!alias || !content.contains(alias))
            throw new Error(`Missing retained overview ${kind} alias ${name}`);
          if (
            !links.some((link) => {
              const url = new URL(link.href);
              return (
                url.origin === location.origin &&
                url.pathname === destination &&
                url.hash === `#${id}-${kind}`
              );
            })
          )
            throw new Error(`Missing owning-group ${kind} link ${name}`);
        }
        return { id: name, summary: summary.textContent.trim(), destination, links: links.length };
      });
    },
    { expected, prefix },
  );
  assert.equal(actual.length, 324, `${route}: complete overview reading flow`);
  return { locale, entries: actual };
}
