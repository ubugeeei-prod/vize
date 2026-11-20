// Registers historical compiler fixes (#6880) as current public API oracles.
//
//   node tools/commands/fixtures/compiler-fix-history-case.ts register [--errors FILE] SPEC.json...
//
// A spec names one indexed fix commit and either its cases or, when the fix
// has no compiled-output effect, its disposition:
//
//   { "commit": "<40 hex>", "cases": [{ "id", "source", "filename"?, "entry": "sfc"|"template",
//       "target": "dom"|"ssr"|"vapor", "options"?: {...}, "files"?: {...}, "errors": N }] }
//   { "commit": "<40 hex>", "cases": [{ "id", "source", "filename"?, "entry": "jsx",
//       "lang"?: "tsx"|"jsx", "mode"?: "vdom"|"vapor"|"ssr", "options"?: {...}, "errors": N }] }
//   { "commit": "<40 hex>", "noBehavior": { "reason": "...", "coveredBy": ["path::test", ...] } }
//
// Cases are written to the compile-history cohort, the runner's cohort-size
// pin is updated, and the compiler fix-history index is regenerated. `--errors`
// takes the `id -> observed` object the runner prints for mismatched counts.
// Snapshots come from `INSTA_UPDATE=always cargo test -p vize_atelier_sfc --test compile_history`
// and, for JSX entries, `INSTA_UPDATE=always cargo test -p vize_atelier_jsx --test jsx_history`.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  COHORT,
  DISPOSITIONS,
  JSX_COHORT,
  collectHistory,
  renderHistory,
} from "./compiler-fix-history.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const runner = "crates/vize_atelier_sfc/tests/compile_history.rs";
const jsxRunner = "crates/vize_atelier_jsx/tests/jsx_history.rs";
const INDEX = "docs/davinci/plan/compiler-fix-history.tsv";
const RECEIPT = "docs/davinci/plan/compiler-fix-history-index.json";
const OPTIONS = new Set([
  "prod",
  "preserveTs",
  "standalone",
  "mode",
  "prefixIdentifiers",
  "hoistStatic",
  "cacheHandlers",
  "comments",
  "sourceMap",
  "templateSyntax",
  "customRenderer",
  "vueVersion",
  "scriptOutput",
  "customElements",
  "patternedTemplate",
  "inTagComments",
  "scoped",
  "minify",
  "viteHelper",
  "noProjectConfig",
]);
const JSX_OPTIONS = new Set([
  "compat",
  "pragma",
  "mergeProps",
  "enableObjectSlots",
  "transformOn",
  "customElementPrefix",
  "hoistStatic",
  "cacheHandlers",
  "sourceMap",
]);

type Case = {
  id: string;
  history: string;
  source: string;
  filename: string;
  entry: "sfc" | "template" | "css" | "vite-css" | "vite";
  target: "dom" | "ssr" | "vapor";
  options?: Record<string, boolean | string | string[]>;
  files?: Record<string, string>;
  errors: number;
};
type JsxCase = {
  id: string;
  history: string;
  source: string;
  filename: string;
  lang: "tsx" | "jsx";
  mode?: "vdom" | "vapor" | "ssr";
  options?: Record<string, boolean | string>;
  errors: number;
};
type SpecCase = Omit<Case, "history" | "entry" | "target"> & {
  entry: Case["entry"] | "jsx";
  target?: "dom" | "ssr" | "vapor";
  lang?: "tsx" | "jsx";
  mode?: "vdom" | "vapor" | "ssr";
};
type Spec = {
  commit: string;
  cases?: SpecCase[];
  noBehavior?: { reason: string; coveredBy: string[] };
};

const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");
const write = (file: string, bytes: string) => fs.writeFileSync(path.join(root, file), bytes);

function normalizeJsx(spec: Spec, item: SpecCase, errors: Record<string, number>): JsxCase {
  assert.match(item.id, /^[a-z0-9]+(?:-[a-z0-9]+)*$/, item.id);
  assert(typeof item.source === "string", `${item.id}: source`);
  const lang = item.lang ?? "tsx";
  assert(["tsx", "jsx"].includes(lang), `${item.id}: lang ${lang}`);
  assert(!item.mode || ["vdom", "vapor", "ssr"].includes(item.mode), `${item.id}: mode`);
  for (const key of Object.keys(item.options ?? {}))
    assert(JSX_OPTIONS.has(key), `${item.id}: unknown JSX option ${key}`);
  const observed = errors[item.id] ?? item.errors;
  assert(Number.isInteger(observed) && observed >= 0, `${item.id}: errors`);
  return {
    id: item.id,
    history: spec.commit,
    source: item.source,
    filename: item.filename ?? `History.${lang}`,
    lang,
    ...(item.mode ? { mode: item.mode } : {}),
    ...(item.options && Object.keys(item.options).length
      ? { options: item.options as Record<string, boolean | string> }
      : {}),
    errors: observed,
  };
}

function normalize(spec: Spec, item: SpecCase, errors: Record<string, number>): Case {
  assert.match(item.id, /^[a-z0-9]+(?:-[a-z0-9]+)*$/, item.id);
  assert(typeof item.source === "string", `${item.id}: source`);
  assert(
    ["sfc", "template", "css", "vite-css", "vite"].includes(item.entry),
    `${item.id}: entry ${item.entry}`,
  );
  assert(["dom", "ssr", "vapor"].includes(item.target ?? ""), `${item.id}: target ${item.target}`);
  assert(item.entry === "sfc" || item.target === "dom", `${item.id}: non-SFC entries are DOM`);
  for (const key of Object.keys(item.options ?? {}))
    assert(OPTIONS.has(key), `${item.id}: unknown option ${key}`);
  const observed = errors[item.id] ?? item.errors;
  assert(Number.isInteger(observed) && observed >= 0, `${item.id}: errors`);
  assert(!item.files || item.entry === "sfc", `${item.id}: project files need an SFC entry`);
  return {
    id: item.id,
    history: spec.commit,
    source: item.source,
    filename: item.filename ?? "History.vue",
    entry: item.entry as Case["entry"],
    target: item.target as Case["target"],
    ...(item.options && Object.keys(item.options).length ? { options: item.options } : {}),
    ...(item.files && Object.keys(item.files).length ? { files: item.files } : {}),
    errors: observed,
  };
}

// The cohort lives under `crates/**/tests/fixtures/**`, which `vp fmt` ignores
// (it exits non-zero when every named file is ignored). Format byte copies at
// an unignored scratch path and write the formatted bytes back.
function formatIgnored(files: string[]) {
  const scratch = fs.mkdtempSync(path.join(root, ".compile-history-fmt-"));
  try {
    const copies = files.map((file, index) => {
      const copy = path.join(scratch, `${index}.json`);
      fs.copyFileSync(path.join(root, file), copy);
      return copy;
    });
    execFileSync("vp", ["fmt", ...copies.map((copy) => path.relative(root, copy))], {
      cwd: root,
      stdio: "ignore",
    });
    files.forEach((file, index) => fs.copyFileSync(copies[index], path.join(root, file)));
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
}

function register(files: string[], errors: Record<string, number>) {
  const cohort: Case[] = JSON.parse(read(COHORT));
  const jsxCohort: JsxCase[] = JSON.parse(read(JSX_COHORT));
  const dispositions = JSON.parse(read(DISPOSITIONS));
  for (const file of files) {
    const spec: Spec = JSON.parse(fs.readFileSync(file, "utf8"));
    assert.match(spec.commit, /^[a-f0-9]{40}$/, file);
    assert(Boolean(spec.cases) !== Boolean(spec.noBehavior), `${file}: cases xor noBehavior`);
    // Re-registering a commit replaces its earlier cases or disposition.
    for (const cases of [cohort, jsxCohort] as { history: string }[][])
      for (let index = cases.length - 1; index >= 0; index -= 1)
        if (cases[index].history === spec.commit) cases.splice(index, 1);
    dispositions.rows = dispositions.rows.filter(
      (row: { fixCommit: string }) => row.fixCommit !== spec.commit,
    );
    if (spec.cases) {
      assert(spec.cases.length > 0, `${file}: no cases`);
      for (const item of spec.cases) {
        assert(
          ![...cohort, ...jsxCohort].some((existing) => existing.id === item.id),
          `duplicate id ${item.id}`,
        );
        if (item.entry === "jsx") jsxCohort.push(normalizeJsx(spec, item, errors));
        else cohort.push(normalize(spec, item, errors));
      }
    } else {
      const { reason, coveredBy } = spec.noBehavior!;
      assert(typeof reason === "string" && reason.length > 0, `${file}: reason`);
      assert(Array.isArray(coveredBy), `${file}: coveredBy`);
      dispositions.rows.push({
        fixCommit: spec.commit,
        state: "no-compile-behavior",
        reason,
        coveredBy,
      });
    }
  }
  dispositions.rows.sort((left: { fixCommit: string }, right: { fixCommit: string }) =>
    left.fixCommit.localeCompare(right.fixCommit),
  );
  write(COHORT, `${JSON.stringify(cohort, null, 2)}\n`);
  write(JSX_COHORT, `${JSON.stringify(jsxCohort, null, 2)}\n`);
  write(DISPOSITIONS, `${JSON.stringify(dispositions, null, 2)}\n`);
  // rustfmt may break the pin over several lines; keep its whitespace.
  const pin = /(assert_eq!\(\s*cases\.len\(\),\s*)\d+,/;
  for (const [file, count] of [
    [runner, cohort.length],
    [jsxRunner, jsxCohort.length],
  ] as const) {
    const source = read(file);
    assert.match(source, pin);
    write(file, source.replace(pin, `$1${count},`));
  }
  // The receipt pins the formatted bytes, so format before regenerating it.
  formatIgnored([COHORT, JSX_COHORT, DISPOSITIONS]);
  const rendered = renderHistory(collectHistory(root));
  write(INDEX, rendered.index);
  write(RECEIPT, rendered.receipt);
  execFileSync("vp", ["fmt", RECEIPT], { cwd: root, stdio: "ignore" });
  const receipt = JSON.parse(rendered.receipt);
  console.log(
    `${cohort.length} SFC/template and ${jsxCohort.length} JSX cases; ` +
      `${receipt.currentApiPinnedFixSubjects} pinned and ` +
      `${receipt.noCompileBehaviorFixSubjects} no-behavior of ${receipt.nonMergeFixSubjects} fixes`,
  );
}

const [command, ...rest] = process.argv.slice(2);
assert.equal(
  command,
  "register",
  "usage: compiler-fix-history-case.ts register [--errors FILE] SPEC...",
);
const at = rest.indexOf("--errors");
const errors = at >= 0 ? JSON.parse(fs.readFileSync(rest.splice(at, 2)[1], "utf8")) : {};
register(rest, errors);
