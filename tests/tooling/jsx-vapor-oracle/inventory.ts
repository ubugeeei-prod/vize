// Custody of the vue-jsx-vapor semantic corpus: verdicts, divergence pins,
// documented surfaces and the value-sensitive coverage matrix (#6884, #6890).
import assert from "node:assert/strict";

type Spec = Record<string, unknown> & {
  id: string;
  source: string;
  steps: unknown[];
  babelCorpusId?: string;
  valueMatrix?: boolean;
};
type Verdict = Record<string, unknown> & {
  verdict: string;
  basis: string;
  referenceConflicts?: number[];
};
interface Inputs {
  corpus: { oracle: { packageVersion: string; vueVersion: string }; cases: Spec[] };
  verdicts: { oraclePackageVersion: string; vueVersion: string; cases: Record<string, Verdict> };
  inventory: string;
  babelCorpus: { cases: { id: string; source: string }[] };
  babelVerdicts: string;
  coverage: { babelExempt: Record<string, string>; vapor: Record<string, string> };
  installed: { packageVersion: string; vueVersion: string };
}

const traceFields = ["dom", "text", "elementIdentities", "warnings", "events"];
const capitalize = (field: string) => field[0]!.toUpperCase() + field.slice(1);

// `side` selects `expected<Field>`, `expectedReference<Field>`, `expectedVdom<Field>`
// or `expectedBabel<Field>`, falling back to the shared expectation.
function expectation(spec: Record<string, unknown>, side: string, field: string): unknown {
  const key = `expected${capitalize(field)}`;
  const sided = side ? `expected${side}${capitalize(field)}` : key;
  return Object.hasOwn(spec, sided) ? spec[sided] : spec[key];
}

function assertDivergences(spec: Spec, verdict: Verdict): boolean {
  // Each side's divergence names its tracking issue and exactly the trace
  // fields that differ; the reference side of each differing field is pinned.
  let divergent = false;
  for (const [differsKey, issueKey, reference, own] of [
    ["differs", "issue", "Reference", ""],
    ["vdomDiffers", "vdomIssue", "Babel", "Vdom"],
  ] as const) {
    const differs = verdict[differsKey] as string[] | undefined;
    assert.equal(
      Number.isInteger(verdict[issueKey]),
      Array.isArray(differs),
      `${spec.id}: ${issueKey}`,
    );
    if (own && differs) assert.ok(spec.babelCorpusId, `${spec.id}: VDOM divergence needs Babel`);
    if (differs) {
      divergent = true;
      assert.ok(differs.length > 0, `${spec.id}: empty divergence`);
      assert.deepEqual(
        differs,
        traceFields.filter((field) => differs.includes(field)),
      );
    }
    const prefix = `expected${reference}`;
    for (const key of Object.keys(spec).filter((key) => key.startsWith(prefix))) {
      assert.ok(
        differs?.some((field) => key === prefix + capitalize(field)),
        `${spec.id}: ${key}`,
      );
    }
    for (const field of differs ?? []) {
      assert.ok(prefix + capitalize(field) in spec, `${spec.id}: pinned ${reference} ${field}`);
    }
    if (differs?.includes("dom")) assert.ok(expectation(spec, own, "dom"), `${spec.id}: own DOM`);
  }
  assert.equal(verdict.verdict, divergent ? "divergent" : "equivalent", `${spec.id}: verdict`);
  // Phases where Babel and vue-jsx-vapor disagree with each other; each
  // backend still matches its own reference there.
  const conflicts = verdict.referenceConflicts;
  if (conflicts !== undefined) {
    assert.ok(spec.babelCorpusId, `${spec.id}: reference conflicts need both references`);
    assert.ok(!divergent, `${spec.id}: reference conflicts belong to equivalent rows`);
    assert.ok(conflicts.length > 0, `${spec.id}: empty reference conflict list`);
    conflicts.forEach((phase, index) => {
      assert.ok(Number.isInteger(phase) && phase >= 0 && phase <= spec.steps.length, spec.id);
      assert.ok(index === 0 || conflicts[index - 1]! < phase, `${spec.id}: ordered conflicts`);
    });
  }
  for (const key of ["expectedVdomText", "expectedVdomElementIdentities"]) {
    assert.ok(!(key in spec) || conflicts || divergent, `${spec.id}: ${key} needs a difference`);
  }
  return divergent;
}

// Every expression container holds a literal, a comment or nothing.
const literalOnly = (source: string) =>
  [...source.matchAll(/\{([^{}]*)\}/gu)].every(([, body]) =>
    /^\s*(?:-?\d+(?:\.\d+)?|"[^"]*"|'[^']*'|\/\*[^*]*\*\/)?\s*$/u.test(body!),
  );

// Each equivalent Babel row has a value-sensitive mounted row or an exemption
// the oracle can check; each equivalent vue-jsx-vapor row is value-sensitive
// itself or names the value-sensitive row that exercises its value position.
function assertValueCoverage({ corpus, verdicts, babelCorpus, babelVerdicts, coverage }: Inputs) {
  const linked = new Set(
    corpus.cases.filter((spec) => spec.valueMatrix).map((spec) => spec.babelCorpusId),
  );
  const same = babelCorpus.cases.filter(({ id }) => babelVerdicts.includes(`("${id}", Same)`));
  for (const { id, source } of same) {
    const exemption = coverage.babelExempt[id];
    assert.notEqual(linked.has(id), exemption !== undefined, `${id}: value row xor exemption`);
    if (exemption === "literal") assert.ok(literalOnly(source), `${id}: literal-only source`);
    else if (exemption !== undefined) {
      assert.equal(exemption, "compile-error", `${id}: exemption reason`);
      assert.ok(id.startsWith("errors/"), `${id}: compile-error rows are error cases`);
    }
  }
  const sameIds = new Set(same.map(({ id }) => id));
  for (const id of Object.keys(coverage.babelExempt)) assert.ok(sameIds.has(id), id);
  const valueRows = new Set(corpus.cases.filter((spec) => spec.valueMatrix).map(({ id }) => id));
  for (const spec of corpus.cases) {
    const equivalent = verdicts.cases[spec.id]!.verdict === "equivalent";
    const target = coverage.vapor[spec.id];
    if (spec.valueMatrix || !equivalent) assert.equal(target, undefined, `${spec.id}: unmapped`);
    else assert.ok(target && valueRows.has(target), `${spec.id}: value-sensitive counterpart`);
  }
  const ids = new Set(corpus.cases.map(({ id }) => id));
  for (const id of Object.keys(coverage.vapor)) assert.ok(ids.has(id), id);
}

function assertInventory(inputs: Inputs) {
  const { corpus, verdicts, inventory, babelCorpus, babelVerdicts, installed } = inputs;
  assert.deepEqual(installed, {
    packageVersion: corpus.oracle.packageVersion,
    vueVersion: corpus.oracle.vueVersion,
  });
  assert.equal(verdicts.oraclePackageVersion, installed.packageVersion);
  assert.equal(verdicts.vueVersion, installed.vueVersion);
  const ids = corpus.cases.map(({ id }) => id).sort();
  assert.deepEqual(Object.keys(verdicts.cases).sort(), ids);
  for (const id of ids) assert.equal(typeof verdicts.cases[id]!.basis, "string", id);
  const babelIds = new Set(babelCorpus.cases.map(({ id }) => id));
  for (const spec of corpus.cases.filter(({ babelCorpusId }) => babelCorpusId)) {
    assert.ok(babelIds.has(spec.babelCorpusId!), `${spec.id}: registered Babel corpus row`);
    assert.ok(
      babelVerdicts.includes(`("${spec.babelCorpusId}", Same)`),
      `${spec.id}: equivalent Babel verdict`,
    );
  }
  for (const spec of corpus.cases) assertDivergences(spec, verdicts.cases[spec.id]!);
  const documented = [
    ...inventory.matchAll(/^\| `([^`]+)`\s+\| (?:equivalent|divergent|deferred)\s+\|/gmu),
  ]
    .map((match) => match[1])
    .sort();
  assert.deepEqual(documented, ids);
  // Every dialect surface row names its mounted cases, and every case serves a surface.
  const surfaces = inventory.slice(inventory.indexOf("## Dialect surface"));
  const surfaceRows = [...surfaces.matchAll(/^\| ([^|`]+?)\s+\| ((?:`[^`]+`(?:, )?)+)\s+\|$/gmu)];
  assert.ok(surfaceRows.length > 0, "dialect surface table");
  const served = new Set(
    surfaceRows.flatMap((row) => [...row[2]!.matchAll(/`([^`]+)`/gu)].map((match) => match[1])),
  );
  for (const id of served) assert.ok(ids.includes(id!), `surface names unknown case ${id}`);
  for (const id of ids) assert.ok(served.has(id), `${id}: no dialect surface`);
  assertValueCoverage(inputs);
}

export { assertInventory, expectation, traceFields };
