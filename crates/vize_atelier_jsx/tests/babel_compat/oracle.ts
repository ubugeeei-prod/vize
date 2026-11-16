/**
 * Differential oracle for `@vue/babel-plugin-jsx` compatibility.
 *
 * Runs every case in `fixtures/corpus.json` through the real
 * `@vue/babel-plugin-jsx` and records Babel's output in
 * `fixtures/babel-output.json`. The Rust oracle reads that committed output
 * without a Node or Babel dependency, while the tooling test re-runs this
 * source and refuses drift from the installed plugin.
 *
 * Usage:
 *   node crates/vize_atelier_jsx/tests/babel_compat/oracle.ts --check
 *   node crates/vize_atelier_jsx/tests/babel_compat/oracle.ts --write
 */
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

type JsonObject = Record<string, unknown>;
type BabelPlugin = unknown;
type BabelPluginEntry = [BabelPlugin, JsonObject];

interface BabelCore {
  transformSync(
    source: string,
    options: {
      plugins: BabelPluginEntry[];
      configFile: false;
      babelrc: false;
      filename: string;
    },
  ): { code: string | null } | null;
}

export interface BabelCorpusCase {
  id: string;
  category: string;
  lang: "jsx" | "tsx";
  source: string;
  babelOptions: JsonObject;
}

export interface BabelCorpus {
  schemaVersion: number;
  description: string;
  oracle: {
    plugin: string;
    pluginVersion: string;
    babelCore: string;
    regenerate: string;
    verify: string;
  };
  dialectCoverage: unknown;
  cases: BabelCorpusCase[];
}

export type BabelCaseResult = { status: "ok"; code: string } | { status: "error"; message: string };

export interface BabelOracleRecord {
  schemaVersion: number;
  description: string;
  pluginVersion: string;
  cases: Record<string, BabelCaseResult>;
}

const here = dirname(fileURLToPath(import.meta.url));
const testsRequire = createRequire(new URL("../../../../tests/package.json", import.meta.url));
export const corpusPath = join(here, "fixtures", "corpus.json");
export const recordedPath = join(here, "fixtures", "babel-output.json");

function isObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringField(value: JsonObject, field: string, owner: string): string {
  const found = value[field];
  if (typeof found !== "string") throw new TypeError(`${owner}.${field} must be a string`);
  return found;
}

function numberField(value: JsonObject, field: string, owner: string): number {
  const found = value[field];
  if (typeof found !== "number") throw new TypeError(`${owner}.${field} must be a number`);
  return found;
}

function jsonFile(path: string): unknown {
  return JSON.parse(readFileSync(path, "utf8")) as unknown;
}

function parseCase(value: unknown, index: number): BabelCorpusCase {
  if (!isObject(value)) throw new TypeError(`corpus.cases[${index}] must be an object`);
  const lang = value.lang;
  if (lang !== "jsx" && lang !== "tsx") {
    throw new TypeError(`corpus.cases[${index}].lang must be jsx or tsx`);
  }
  const babelOptions = value.babelOptions;
  if (babelOptions !== undefined && !isObject(babelOptions)) {
    throw new TypeError(`corpus.cases[${index}].babelOptions must be an object`);
  }
  return {
    id: stringField(value, "id", `corpus.cases[${index}]`),
    category: stringField(value, "category", `corpus.cases[${index}]`),
    lang,
    source: stringField(value, "source", `corpus.cases[${index}]`),
    babelOptions: babelOptions ?? {},
  };
}

export function readCorpus(): BabelCorpus {
  const value = jsonFile(corpusPath);
  if (!isObject(value)) throw new TypeError("corpus must be an object");
  if (!isObject(value.oracle)) throw new TypeError("corpus.oracle must be an object");
  if (!Array.isArray(value.cases)) throw new TypeError("corpus.cases must be an array");
  return {
    schemaVersion: numberField(value, "schemaVersion", "corpus"),
    description: stringField(value, "description", "corpus"),
    oracle: {
      plugin: stringField(value.oracle, "plugin", "corpus.oracle"),
      pluginVersion: stringField(value.oracle, "pluginVersion", "corpus.oracle"),
      babelCore: stringField(value.oracle, "babelCore", "corpus.oracle"),
      regenerate: stringField(value.oracle, "regenerate", "corpus.oracle"),
      verify: stringField(value.oracle, "verify", "corpus.oracle"),
    },
    dialectCoverage: value.dialectCoverage,
    cases: value.cases.map(parseCase),
  };
}

function parseResult(value: unknown, id: string): BabelCaseResult {
  if (!isObject(value)) throw new TypeError(`recorded.cases[${id}] must be an object`);
  if (value.status === "ok") return { status: "ok", code: stringField(value, "code", id) };
  if (value.status === "error") {
    return { status: "error", message: stringField(value, "message", id) };
  }
  throw new TypeError(`recorded.cases[${id}].status must be ok or error`);
}

export function readRecorded(): BabelOracleRecord {
  const value = jsonFile(recordedPath);
  if (!isObject(value)) throw new TypeError("recorded oracle must be an object");
  if (!isObject(value.cases)) throw new TypeError("recorded.cases must be an object");
  return {
    schemaVersion: numberField(value, "schemaVersion", "recorded"),
    description: stringField(value, "description", "recorded"),
    pluginVersion: stringField(value, "pluginVersion", "recorded"),
    cases: Object.fromEntries(
      Object.entries(value.cases).map(([id, result]) => [id, parseResult(result, id)]),
    ),
  };
}

/** Materialize the JSON representation of Babel's function-valued option. */
function materializeOptions(babelOptions: JsonObject): JsonObject {
  const options = { ...babelOptions };
  if (Array.isArray(options.isCustomElement)) {
    if (!options.isCustomElement.every((tag): tag is string => typeof tag === "string")) {
      throw new TypeError("isCustomElement tags must be strings");
    }
    const customTags = new Set(options.isCustomElement);
    options.isCustomElement = (tag: string): boolean => customTags.has(tag);
  }
  return options;
}

/** Transform one corpus case, returning Babel's emitted code or its error. */
export async function transformCase(
  babel: BabelCore,
  plugin: BabelPlugin,
  tsSyntaxPlugin: BabelPlugin,
  entry: BabelCorpusCase,
): Promise<BabelCaseResult> {
  const plugins: BabelPluginEntry[] = [[plugin, materializeOptions(entry.babelOptions)]];
  if (entry.lang === "tsx") plugins.push([tsSyntaxPlugin, { isTSX: true }]);
  try {
    const result = babel.transformSync(entry.source, {
      plugins,
      configFile: false,
      babelrc: false,
      filename: entry.lang === "tsx" ? "input.tsx" : "input.jsx",
    });
    if (typeof result?.code !== "string") throw new TypeError("Babel returned no emitted code");
    return { status: "ok", code: result.code };
  } catch (error: unknown) {
    const message = (error instanceof Error ? error.message : String(error)).split("\n")[0];
    const colon = message.indexOf(": ");
    return { status: "error", message: colon === -1 ? message : message.slice(colon + 2) };
  }
}

function defaultExport(module: unknown, name: string): object | ((...args: never[]) => unknown) {
  const value = isObject(module) && "default" in module ? module.default : module;
  if ((typeof value !== "object" && typeof value !== "function") || value === null) {
    throw new TypeError(`${name} did not export a module value`);
  }
  return value;
}

function babelCore(module: unknown): BabelCore {
  const value = defaultExport(module, "@babel/core");
  if (!("transformSync" in value) || typeof value.transformSync !== "function") {
    throw new TypeError("@babel/core did not export transformSync");
  }
  return value as BabelCore;
}

/** Run the whole corpus, returning the recordable plugin version and cases. */
export async function runOracle(): Promise<BabelOracleRecord> {
  const babelModule: unknown = testsRequire("@babel/core");
  const pluginModule: unknown = testsRequire("@vue/babel-plugin-jsx");
  const tsSyntaxModule: unknown = testsRequire("@babel/plugin-syntax-typescript");
  const pluginPackage: unknown = testsRequire("@vue/babel-plugin-jsx/package.json");
  if (!isObject(pluginPackage) || typeof pluginPackage.version !== "string") {
    throw new TypeError("@vue/babel-plugin-jsx package version is unavailable");
  }
  const corpus = readCorpus();
  const cases: Record<string, BabelCaseResult> = {};
  const babel = babelCore(babelModule);
  const plugin = defaultExport(pluginModule, "@vue/babel-plugin-jsx");
  const tsSyntaxPlugin = defaultExport(tsSyntaxModule, "@babel/plugin-syntax-typescript");
  for (const entry of corpus.cases) {
    cases[entry.id] = await transformCase(babel, plugin, tsSyntaxPlugin, entry);
  }
  return {
    schemaVersion: 1,
    description:
      "Recorded @vue/babel-plugin-jsx output for every case in corpus.json. Generated — do not hand-edit; re-record with oracle.ts --write.",
    pluginVersion: pluginPackage.version,
    cases,
  };
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const write = process.argv.includes("--write");
  const recorded = await runOracle();
  if (write) {
    writeFileSync(recordedPath, `${JSON.stringify(recorded, null, 2)}\n`);
    console.log(`recorded ${Object.keys(recorded.cases).length} cases -> ${recordedPath}`);
  } else {
    const expected = readRecorded();
    const actual = JSON.stringify(recorded, null, 2);
    if (JSON.stringify(expected, null, 2) !== actual) {
      console.error("babel-output.json is out of date; re-record with --write");
      process.exitCode = 1;
    } else {
      console.log(`oracle matches ${Object.keys(recorded.cases).length} recorded cases`);
    }
  }
}
