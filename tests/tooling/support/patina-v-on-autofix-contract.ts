import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  parseJson,
} from "../../../tools/support/json-value.ts";

export const root = fileURLToPath(new URL("../../../", import.meta.url));
export const directory = join(root, "tests/_fixtures/differential/linter/v-on-event-autofix-6097");
export const rule = "vue/v-on-event-hyphenation";
export const hash = (raw: Buffer | string) => createHash("sha256").update(raw).digest("hex");
export interface AutofixCase {
  id: string;
  file: string;
  options: unknown[];
  sourceSha256: string;
  reference: { file: string; sha256: string };
}
const corpusBytes = readFileSync(join(directory, "cases.json"));
assert.equal(hash(corpusBytes), "c077afcd6f6cacc74a3f18cc6fefe3b2c5ec4182706fbf15d2eeec839df39662");
export const corpus = asRecord(parseJson(corpusBytes.toString()));
export const cases: AutofixCase[] = asArray(corpus.cases).map((value) => {
  const row = asRecord(value),
    reference = asRecord(row.reference);
  return {
    id: asString(row.id),
    file: asString(row.file),
    options: asArray(row.options),
    sourceSha256: asString(row.sourceSha256),
    reference: { file: asString(reference.file), sha256: asString(reference.sha256) },
  };
});
assert.equal(cases.length, 22);
assert.equal(new Set(cases.map((row) => row.id)).size, 22);

export function sourceFor(row: AutofixCase): string {
  assert(
    ["LocaleMenu.vue.txt", "Modifiers.vue.txt", "Unicode.vue.txt", "FinalSigma.vue.txt"].includes(
      row.file,
    ),
  );
  const bytes = readFileSync(join(directory, row.file));
  assert.equal(hash(bytes), row.sourceSha256);
  if (row.file === "LocaleMenu.vue.txt") {
    assert.equal(hash(bytes), "431cdc5f87aa51a64da55275db5fe94fbb8d623190855469d8dd2a94100ae78a");
    assert(
      bytes.equals(
        readFileSync(
          join(
            root,
            "tests/_fixtures/differential/linter/v-on-hyphenation-suggestion-10822/LocaleMenu.vue.txt",
          ),
        ),
      ),
    );
  }
  return bytes.toString();
}
export function referenceFor(row: AutofixCase) {
  assert.match(
    row.reference.file,
    /^reference\/(?:LocaleMenu|Modifiers|Unicode|FinalSigma)\.vue\.[a-z-]+\.json$/u,
  );
  const bytes = readFileSync(join(directory, row.reference.file));
  assert.equal(hash(bytes), row.reference.sha256);
  return asRecord(parseJson(bytes.toString()));
}
export function policy(row: AutofixCase): unknown {
  if (row.options.length === 0) return undefined;
  const style = asString(row.options[0]);
  if (row.options.length === 1) return style;
  return { style, ...asRecord(row.options[1]) };
}
export function flat(row: AutofixCase) {
  const value = policy(row);
  if (value === undefined) return {};
  if (typeof value === "string") return { vOnEventHyphenation: value };
  const options = asRecord(value);
  return {
    vOnEventHyphenation: options.style,
    vOnEventHyphenationIgnore: options.ignore ?? [],
    ...(options.autofix === undefined ? {} : { vOnEventHyphenationAutofix: options.autofix }),
  };
}

interface ReferenceLinter {
  verify(source: string, config: unknown[], options: { filename: string }): unknown[];
  verifyAndFix(source: string, config: unknown[], options: { filename: string }): unknown;
}
export function provider() {
  const require = createRequire(join(root, "tools/benchmarks/scripts/package.json"));
  const versions = {
    eslint: "10.4.1",
    "eslint-plugin-vue": "10.9.2",
    "vue-eslint-parser": "10.4.1",
    "@typescript-eslint/parser": "8.65.0",
  };
  for (const [name, version] of Object.entries(versions))
    assert.equal(asRecord(require(name + "/package.json")).version, version);
  const pluginRoot = dirname(require.resolve("eslint-plugin-vue/package.json"));
  assert.equal(
    hash(readFileSync(join(pluginRoot, "dist/rules/v-on-event-hyphenation.js"))),
    "f91e547b8e0275dac03081fab804190e619fe87ee376302b50814f929c503849",
  );
  assert.equal(
    hash(readFileSync(join(pluginRoot, "dist/utils/casing.js"))),
    "911df4eed3c7d75afac7c6df4a13527d104afdbbd43821993c2b4ece6108f44c",
  );
  const Linter = asRecord(require("eslint")).Linter as new () => ReferenceLinter;
  const plugin: unknown = require("eslint-plugin-vue"),
    parser: unknown = require("vue-eslint-parser"),
    scriptParser: unknown = require("@typescript-eslint/parser");
  const casing = asRecord(require(join(pluginRoot, "dist/utils/casing.js")));
  assert.equal(typeof casing.kebabCase, "function");
  const kebab = casing.kebabCase as (name: string) => string;
  const config = (row: AutofixCase) => [
    {
      files: ["**/*.vue"],
      languageOptions: {
        parser,
        parserOptions: { parser: scriptParser, ecmaVersion: 2022, sourceType: "module" },
      },
      plugins: { vue: plugin },
      rules: { [rule]: ["warn", ...row.options] },
    },
  ];
  return { linter: new Linter(), config, kebab };
}

function position(source: string, line: number, column: number) {
  const lines = source.split("\n");
  assert(line >= 1 && line <= lines.length && column >= 1);
  const prefix = lines.slice(0, line - 1).join("\n");
  const utf16 = prefix.length + (line > 1 ? 1 : 0) + column - 1;
  return { line, column, offset: Buffer.byteLength(source.slice(0, utf16)) };
}

/** Preserve Vize's existing English presentation; edits come from whole official findings. */
export function expected(
  row: AutofixCase,
  source: string,
  findings: unknown,
  includeFixes = true,
  helpLevel = "full",
) {
  const { kebab } = provider();
  const diagnostics = asArray(findings).map((value) => {
    const finding = asRecord(value);
    assert.equal(finding.ruleId, rule);
    assert.equal(finding.severity, 1);
    const caption = /^v-on event '([^']+)' (?:must be hyphenated\.|can't be hyphenated\.)$/u.exec(
      asString(finding.message),
    );
    assert(caption);
    const key = caption[1]!;
    const name = (
      key.startsWith("@") ? key.slice(1) : key.startsWith("v-on:") ? key.slice(5) : ""
    ).split(".")[0]!;
    assert(name);
    const always = finding.messageId === "mustBeHyphenated";
    assert(always || finding.messageId === "cannotBeHyphenated");
    const help = always
      ? "Rename the listener to kebab-case so it matches the emitted event name."
      : "Use camelCase or a single-word event name instead of kebab-case.";
    let fix: unknown = null;
    if (finding.fix !== undefined) {
      const originalFix = asRecord(finding.fix),
        range = asArray(originalFix.range);
      assert.equal(range.length, 2);
      fix = {
        message: help,
        edits: [
          {
            start: Buffer.byteLength(source.slice(0, asNumber(range[0]))),
            end: Buffer.byteLength(source.slice(0, asNumber(range[1]))),
            newText: asString(originalFix.text),
          },
        ],
      };
    }
    return {
      rule,
      severity: "warning",
      message: always
        ? `Custom event listeners on components should be hyphenated: use '${kebab(name)}'`
        : `Custom event listeners on components should not be hyphenated: '${name}'`,
      location: {
        start: position(source, asNumber(finding.line), asNumber(finding.column)),
        end: position(source, asNumber(finding.endLine), asNumber(finding.endColumn)),
      },
      help: helpLevel === "none" ? null : help,
      ...(includeFixes ? { fix } : {}),
    };
  });
  return {
    filename: row.file.replace(/\.txt$/u, ""),
    errorCount: 0,
    warningCount: diagnostics.length,
    diagnostics,
  };
}

export function apply(source: string, packet: unknown): string {
  const edits = asArray(asRecord(packet).diagnostics)
    .flatMap((value) => {
      const fix = asRecord(value).fix;
      return fix === null || fix === undefined
        ? []
        : asArray(asRecord(fix).edits).map((edit) => asRecord(edit));
    })
    .sort((left, right) => asNumber(right.start) - asNumber(left.start));
  let bytes = Buffer.from(source);
  for (const edit of edits) {
    const start = asNumber(edit.start),
      end = asNumber(edit.end);
    assert(start >= 0 && start <= end && end <= bytes.length);
    bytes = Buffer.concat([
      bytes.subarray(0, start),
      Buffer.from(asString(edit.newText)),
      bytes.subarray(end),
    ]);
  }
  return bytes.toString();
}
