// P5-6a / P5-6b / P5-6c: Maestro's request paths read the resident tier's
// memoized SFC parse instead of calling `parse_sfc` per request.
//
// Hover, completion, definition, template scope, references, diagnostics,
// semantic tokens and inlay hints have no `parse_sfc` call left. The
// crate-wide count is pinned exactly: a new call site fails, and a removed
// one fails until this ceiling is lowered to match, so the count only falls.
//
// P5-6c acceptance is not met. After the inlay-hints slice, 30 request-path
// sites remain (document structure, ecosystem, code actions, rename, links,
// lenses, colors, formatting, virtual documents, importers, the type service,
// musea, template refs and SFC regions) plus 15 test-only sites.
// `with_content` is not deleted.

import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const maestroSrc = path.join(repoRoot, "crates/vize_maestro/src");

/** `parse_sfc` call sites left in `crates/vize_maestro/src` (81 before P5-6a, 50 before P5-6b, 47 before semantic tokens, 46 before inlay hints). */
const CEILING = 45;

/**
 * Request-path `parse_sfc(` sites after inlay hints moved onto the resident
 * descriptor. The other `CEILING - REQUEST_PATH` sites are tests.
 */
const REQUEST_PATH = 30;

/** Files whose every `parse_sfc(` is a test, including inline `#[cfg(test)]` modules. */
function isTestOnly(file: string): boolean {
  const base = file.slice(file.lastIndexOf("/") + 1);
  return (
    base === "tests.rs" ||
    base.startsWith("tests_") ||
    base.endsWith("_tests.rs") ||
    file === "ide/ecosystem/i18n.rs" ||
    file === "ide/ecosystem/router.rs"
  );
}

/** The P5-6a wave: every path here reads `IdeContext::descriptor` instead. */
const WAVE = ["hover", "completion", "definition", "template_scope", "references"].flatMap(
  (feature) => [`ide/${feature}.rs`, `ide/${feature}/`],
);

/** Every `parse_sfc(` on a non-comment line, as `path:line`. */
function parseSfcSites(files: Map<string, string>): string[] {
  const sites: string[] = [];
  for (const [file, source] of files) {
    source.split("\n").forEach((line, index) => {
      if (!line.trimStart().startsWith("//") && line.includes("parse_sfc(")) {
        sites.push(`${file}:${index + 1}`);
      }
    });
  }
  return sites.sort();
}

function maestroSources(): Map<string, string> {
  const files = new Map<string, string>();
  for (const entry of readdirSync(maestroSrc, { recursive: true, encoding: "utf8" })) {
    if (entry.endsWith(".rs")) {
      const file = entry.split(path.sep).join("/");
      files.set(file, readFileSync(path.join(maestroSrc, entry), "utf8"));
    }
  }
  return files;
}

const sites = parseSfcSites(maestroSources());

test("the P5-6a wave calls parse_sfc nowhere", () => {
  const inWave = sites.filter((site) => WAVE.some((prefix) => site.startsWith(prefix)));
  assert.deepEqual(inWave, []);
});

test("the P5-6b diagnostics wave calls parse_sfc nowhere", () => {
  const inWave = sites.filter(
    (site) => site.startsWith("ide/diagnostics.rs:") || site.startsWith("ide/diagnostics/"),
  );
  assert.deepEqual(inWave, []);
});

test("the P5-6c semantic-tokens slice calls parse_sfc nowhere", () => {
  const inWave = sites.filter(
    (site) => site.startsWith("ide/semantic_tokens.rs:") || site.startsWith("ide/semantic_tokens/"),
  );
  assert.deepEqual(inWave, []);
});

test("the P5-6c inlay-hints slice calls parse_sfc nowhere", () => {
  const inWave = sites.filter(
    (site) => site.startsWith("ide/inlay_hint.rs:") || site.startsWith("ide/inlay_hint/"),
  );
  assert.deepEqual(inWave, []);
});

test("request-path parse_sfc sites remaining after inlay hints", () => {
  const requestPath = sites.filter((site) => !isTestOnly(site.slice(0, site.lastIndexOf(":"))));
  assert.equal(
    requestPath.length,
    REQUEST_PATH,
    `found ${requestPath.length} request-path sites; P5-6c acceptance is 0\n${requestPath.join("\n")}`,
  );
});

test("the crate-wide parse_sfc count equals its ceiling", () => {
  assert.equal(
    sites.length,
    CEILING,
    `found ${sites.length} parse_sfc call sites; a new site is a regression, and a removed one lowers CEILING`,
  );
});

test("the count fails on an injected call and ignores comments", () => {
  const injected = new Map([
    ["ide/hover/injected.rs", "let d = vize_atelier_sfc::parse_sfc(&ctx.content, options);"],
    ["ide/hover/comment.rs", "    // re-parsing with `parse_sfc(` is gone"],
  ]);
  assert.deepEqual(parseSfcSites(injected), ["ide/hover/injected.rs:1"]);
});
