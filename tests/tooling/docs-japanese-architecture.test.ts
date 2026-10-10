import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

function content(name: string, locale = "") {
  return readFileSync(
    new URL(`../../docs/content/${locale}architecture/${name}.md`, import.meta.url),
    "utf8",
  );
}

test("complete Japanese architecture reviews retain source commands, tables and diagrams", () => {
  for (const name of ["overview", "crates", "source-guide"]) {
    const source = content(name);
    const translation = content(name, "ja/");
    assert.ok(/Reviewed translation;[^\n]+scope: complete document/.test(translation), name);
    const code = (text: string) =>
      [...text.matchAll(/(?<!`)`([^`\n]+)`(?!`)/g)].map((m) => m[1]).sort();
    assert.deepEqual(code(translation), code(source), `${name}: commands and paths`);
    const tables = (text: string) =>
      (text.match(/(?:^\|[^\n]+\n)+/gm) ?? []).map((table) =>
        table
          .trim()
          .split("\n")
          .map((row) => row.split("|").length - 2),
      );
    assert.deepEqual(tables(translation), tables(source), `${name}: complete table cells`);
    const diagrams = (text: string) =>
      [...text.matchAll(/```mermaid\n([\s\S]*?)```/g)].map((m) => m[1]);
    assert.deepEqual(diagrams(translation), diagrams(source), `${name}: diagram contract`);
  }
  const overview = content("overview", "ja/");
  for (let step = 1; step <= 6; step++) assert.match(overview, new RegExp(`^${step}\\. `, "m"));
  assert.doesNotMatch(content("source-guide", "ja/"), /MoonBit|moon run|moonScript/);
});

test("historical TypeScript research links identify the verified JS implementation revision", () => {
  const revision = "c63de15a992d37f0d6cec03ac7631872838602cb";
  for (const locale of ["", "ja/"]) {
    const practices = content("language-engineering-practices", locale);
    assert.ok(
      practices.includes(
        `https://github.com/microsoft/TypeScript/blob/${revision}/CONTRIBUTING%2Emd`,
      ),
    );
    assert.ok(
      practices.includes(
        `https://github.com/microsoft/TypeScript/tree/${revision}/tests/cases/fourslash`,
      ),
    );
    assert.ok(practices.includes("v5.9.3"));
    assert.ok(
      practices.includes("https://github.com/microsoft/TypeScript/blob/main/CONTRIBUTING%2Emd"),
    );
    assert.ok(
      !practices.includes(
        "https://github.com/microsoft/TypeScript/tree/main/tests/cases/fourslash",
      ),
    );
  }
});
