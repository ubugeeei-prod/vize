// Registers historical compiler fixes (#6880) as current public API oracles.
//
//   node tools/commands/fixtures/compiler-fix-history-case.ts register [--errors FILE] SPEC.json...
//
// A spec names one indexed fix commit and either its cases or, when the fix
// has no compiled-output effect, its disposition:
//
//   { "commit": "<40 hex>", "cases": [{ "id", "source", "filename"?, "entry": "sfc"|"template",
//       "target": "dom"|"ssr"|"vapor", "options"?: {...}, "errors": N }] }
//   { "commit": "<40 hex>", "noBehavior": { "reason": "...", "coveredBy": ["path::test", ...] } }
//
// Cases are written to the compile-history cohort, the runner's cohort-size
// pin is updated, and the compiler fix-history index is regenerated. `--errors`
// takes the `id -> observed` object the runner prints for mismatched counts.
// Snapshots come from `INSTA_UPDATE=always cargo test -p vize_atelier_sfc --test compile_history`.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { COHORT, DISPOSITIONS, collectHistory, renderHistory } from "./compiler-fix-history.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const runner = "crates/vize_atelier_sfc/tests/compile_history.rs";
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
]);

type Case = {
  id: string;
  history: string;
  source: string;
  filename: string;
  entry: "sfc" | "template";
  target: "dom" | "ssr" | "vapor";
  options?: Record<string, boolean | string>;
  errors: number;
};
type Spec = {
  commit: string;
  cases?: Omit<Case, "history">[];
  noBehavior?: { reason: string; coveredBy: string[] };
};

const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");
const write = (file: string, bytes: string) => fs.writeFileSync(path.join(root, file), bytes);

function normalize(spec: Spec, item: Omit<Case, "history">, errors: Record<string, number>): Case {
  assert.match(item.id, /^[a-z0-9]+(?:-[a-z0-9]+)*$/, item.id);
  assert(typeof item.source === "string", `${item.id}: source`);
  assert(["sfc", "template"].includes(item.entry), `${item.id}: entry ${item.entry}`);
  assert(["dom", "ssr", "vapor"].includes(item.target), `${item.id}: target ${item.target}`);
  assert(item.entry === "sfc" || item.target === "dom", `${item.id}: template entries are DOM`);
  for (const key of Object.keys(item.options ?? {}))
    assert(OPTIONS.has(key), `${item.id}: unknown option ${key}`);
  const observed = errors[item.id] ?? item.errors;
  assert(Number.isInteger(observed) && observed >= 0, `${item.id}: errors`);
  const normalized: Case = {
    id: item.id,
    history: spec.commit,
    source: item.source,
    filename: item.filename ?? "History.vue",
    entry: item.entry,
    target: item.target,
    errors: observed,
  };
  if (item.options && Object.keys(item.options).length) {
    const { errors: count, ...rest } = normalized;
    return { ...rest, options: item.options, errors: count };
  }
  return normalized;
}

function register(files: string[], errors: Record<string, number>) {
  const cohort: Case[] = JSON.parse(read(COHORT));
  const dispositions = JSON.parse(read(DISPOSITIONS));
  for (const file of files) {
    const spec: Spec = JSON.parse(fs.readFileSync(file, "utf8"));
    assert.match(spec.commit, /^[a-f0-9]{40}$/, file);
    assert(Boolean(spec.cases) !== Boolean(spec.noBehavior), `${file}: cases xor noBehavior`);
    // Re-registering a commit replaces its earlier cases or disposition.
    for (let index = cohort.length - 1; index >= 0; index -= 1)
      if (cohort[index].history === spec.commit) cohort.splice(index, 1);
    dispositions.rows = dispositions.rows.filter(
      (row: { fixCommit: string }) => row.fixCommit !== spec.commit,
    );
    if (spec.cases) {
      assert(spec.cases.length > 0, `${file}: no cases`);
      for (const item of spec.cases) {
        assert(!cohort.some((existing) => existing.id === item.id), `duplicate id ${item.id}`);
        cohort.push(normalize(spec, item, errors));
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
  write(DISPOSITIONS, `${JSON.stringify(dispositions, null, 2)}\n`);
  const source = read(runner);
  // rustfmt may break the pin over several lines; keep its whitespace.
  const pin = /(assert_eq!\(\s*cases\.len\(\),\s*)\d+,/;
  assert.match(source, pin);
  write(runner, source.replace(pin, `$1${cohort.length},`));
  const rendered = renderHistory(collectHistory(root));
  write(INDEX, rendered.index);
  write(RECEIPT, rendered.receipt);
  const receipt = JSON.parse(rendered.receipt);
  console.log(
    `${cohort.length} cases; ${receipt.currentApiPinnedFixSubjects} pinned and ` +
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
