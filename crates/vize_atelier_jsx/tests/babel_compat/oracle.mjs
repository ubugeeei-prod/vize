/**
 * Differential oracle for `@vue/babel-plugin-jsx` compatibility.
 *
 * Runs every case in `fixtures/corpus.json` through the real
 * `@vue/babel-plugin-jsx` and records babel's output in
 * `fixtures/babel-output.json`. That recorded file is the committed ground
 * truth: the Rust oracle test (`tests/babel_compat_oracle.rs`) reads it with no
 * Node/babel dependency at all, so the Rust side stays offline and fast, while
 * `tests/tooling/babel-jsx-oracle.test.ts` re-runs this script in CI and fails
 * if the recording has drifted from the installed plugin.
 *
 * Usage:
 *   node crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs --check   # verify
 *   node crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs --write   # re-record
 */
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
export const corpusPath = join(here, "fixtures", "corpus.json");
export const recordedPath = join(here, "fixtures", "babel-output.json");

export function readCorpus() {
  return JSON.parse(readFileSync(corpusPath, "utf8"));
}

export function readRecorded() {
  return JSON.parse(readFileSync(recordedPath, "utf8"));
}

/**
 * `isCustomElement` is a function option, but the corpus is plain JSON. Cases
 * declare it as the *list of tags* that should be treated as custom elements
 * and the predicate is built here, so the corpus stays declarative, readable
 * from Rust, and free of executable strings.
 */
function materializeOptions(babelOptions) {
  const options = { ...babelOptions };
  if (Array.isArray(options.isCustomElement)) {
    const customTags = new Set(options.isCustomElement);
    options.isCustomElement = (tag) => customTags.has(tag);
  }
  return options;
}

/** Transform one corpus case, returning babel's emitted code or its error. */
export async function transformCase(babel, plugin, tsSyntaxPlugin, entry) {
  const plugins = [[plugin, materializeOptions(entry.babelOptions)]];
  if (entry.lang === "tsx") {
    plugins.push([tsSyntaxPlugin, { isTSX: true }]);
  }
  try {
    const result = babel.transformSync(entry.source, {
      plugins,
      configFile: false,
      babelrc: false,
      filename: entry.lang === "tsx" ? "input.tsx" : "input.jsx",
    });
    return { status: "ok", code: result.code };
  } catch (error) {
    // Babel prefixes messages with the (temp-dir dependent) filename, so only
    // the first line after the prefix is recorded — the message itself is the
    // observable contract, the path is not.
    const message = String(error.message).split("\n")[0];
    const colon = message.indexOf(": ");
    return { status: "error", message: colon === -1 ? message : message.slice(colon + 2) };
  }
}

/** Run the whole corpus, returning the recordable `{ pluginVersion, cases }`. */
export async function runOracle() {
  const [{ default: babel }, { default: plugin }, { default: tsSyntaxPlugin }, pluginPkg] =
    await Promise.all([
      import("@babel/core"),
      import("@vue/babel-plugin-jsx"),
      import("@babel/plugin-syntax-typescript"),
      import("@vue/babel-plugin-jsx/package.json", { with: { type: "json" } }).then(
        (m) => m.default,
      ),
    ]);
  const corpus = readCorpus();
  const cases = {};
  for (const entry of corpus.cases) {
    cases[entry.id] = await transformCase(babel, plugin, tsSyntaxPlugin, entry);
  }
  return {
    schemaVersion: 1,
    description:
      "Recorded @vue/babel-plugin-jsx output for every case in corpus.json. Generated — do not hand-edit; re-record with oracle.mjs --write.",
    pluginVersion: pluginPkg.version,
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
