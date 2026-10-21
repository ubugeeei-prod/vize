import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

type Dialect = "pug" | "jade";
type State = "partial" | "preservation";

type Audit = {
  version: number;
  issue: number;
  auditedRevision: string;
  dialects: Dialect[];
  fixtures: Array<{
    path: string;
    dialect: Dialect;
    role: "accepted-inline-declarative" | "known-executable-gap";
    sha256: string;
  }>;
  surfaces: Array<{
    id: string;
    label: string;
    dialectState: Record<Dialect, State>;
    current: string;
    remaining: string;
    witnesses: Array<{ path: string; anchor: string; dialects: Dialect[] }>;
  }>;
};

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const auditPath = "tests/_fixtures/pug-jade-surface-audit-8908/audit.json";
const decisionPath = "docs/davinci/decisions/2026-10-22-8908-pug-jade-surface-audit.md";

function read(relative: string): string {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

function displayState(state: State): string {
  return state === "partial" ? "Partial" : "Preservation";
}

function validateAudit(audit: Audit, decision: string): void {
  assert.equal(audit.version, 1);
  assert.equal(audit.issue, 8908);
  assert.match(audit.auditedRevision, /^[0-9a-f]{40}$/u);
  assert.deepEqual(audit.dialects, ["pug", "jade"]);
  assert.deepEqual(
    audit.surfaces.map(({ id }) => id),
    ["compiler", "vite", "typecheck", "lint", "format", "lsp"],
  );
  const decisionRows = new Map(
    decision
      .split("\n")
      .filter((line) => line.startsWith("|"))
      .map((line) =>
        line
          .split("|")
          .slice(1, -1)
          .map((cell) => cell.trim()),
      )
      .filter(([label]) => label !== "Surface" && !/^-+$/u.test(label))
      .map((cells) => [cells[0], cells]),
  );
  assert.equal(decisionRows.size, 6, "decision must contain exactly six surface rows");

  const fixtureRoles = new Set<string>();
  for (const fixture of audit.fixtures) {
    assert.match(fixture.path, /^tests\/_fixtures\/pug-jade-surface-audit-8908\//u);
    assert.equal(sha256(read(fixture.path)), fixture.sha256, `${fixture.path} bytes drifted`);
    fixtureRoles.add(`${fixture.dialect}:${fixture.role}`);
  }
  assert.deepEqual([...fixtureRoles].sort(), [
    "jade:accepted-inline-declarative",
    "pug:accepted-inline-declarative",
    "pug:known-executable-gap",
  ]);

  for (const surface of audit.surfaces) {
    const expectedState: State = surface.id === "format" ? "preservation" : "partial";
    assert.deepEqual(surface.dialectState, { pug: expectedState, jade: expectedState });
    assert.ok(surface.current.length > 40, `${surface.id}: current behavior is not explicit`);
    assert.ok(surface.remaining.length > 40, `${surface.id}: remaining boundary is not explicit`);

    const covered = new Set<Dialect>();
    for (const witness of surface.witnesses) {
      assert.ok(witness.dialects.length > 0, `${surface.id}: empty witness dialects`);
      const source = read(witness.path);
      const first = source.indexOf(witness.anchor);
      assert.notEqual(first, -1, `${surface.id}: missing ${witness.path} anchor`);
      assert.equal(
        source.indexOf(witness.anchor, first + witness.anchor.length),
        -1,
        `${surface.id}: ambiguous ${witness.path} anchor`,
      );
      const isTestFile = /(?:\.test\.[cm]?[jt]s|\/tests\/.*\.rs)$/u.test(witness.path);
      const isInlineRustTest = source.slice(Math.max(0, first - 200), first).includes("#[test]");
      assert.ok(isTestFile || isInlineRustTest, `${witness.path} is not executable evidence`);
      for (const dialect of witness.dialects) covered.add(dialect);
    }
    assert.deepEqual([...covered].sort(), ["jade", "pug"], `${surface.id}: missing dialect proof`);

    assert.deepEqual(
      decisionRows.get(surface.label),
      [
        surface.label,
        displayState(surface.dialectState.pug),
        displayState(surface.dialectState.jade),
        surface.current,
        surface.remaining,
      ],
      `${surface.id}: decision table drifted from the audit`,
    );
  }
}

void test("#8908 audit pins every current Pug and Jade surface to executable evidence", () => {
  const audit = JSON.parse(read(auditPath)) as Audit;
  validateAudit(audit, read(decisionPath));

  const pug = read("tests/_fixtures/pug-jade-surface-audit-8908/accepted.pug.vue.txt");
  const jade = read("tests/_fixtures/pug-jade-surface-audit-8908/accepted.jade.vue.txt");
  assert.equal(jade.replace('lang="jade"', 'lang="pug"'), pug, "Jade must be the exact alias twin");
  for (const token of [
    "v-if=",
    "AuditCard(",
    "@select=",
    "template(#default=",
    "{{ label }}",
    "v-for=",
  ]) {
    assert.ok(pug.includes(token), `accepted fixture misses ${token}`);
  }

  const refused = read(
    "tests/_fixtures/pug-jade-surface-audit-8908/refused-executable.pug.vue.txt",
  );
  assert.match(refused, /(?:^|\n)mixin card\(title\)\n/u);
  assert.match(refused, /(?:^|\n)\+card\(heading\)\n/u);
  assert.match(
    read("tests/_fixtures/davinci-pug/refused/mixin-and-call.diagnostics"),
    /not supported/u,
  );
});

void test("#8908 audit fails closed when dialect proof, fixture custody, or the table drifts", () => {
  const original = JSON.parse(read(auditPath)) as Audit;
  const decision = read(decisionPath);

  const missingJade = structuredClone(original);
  missingJade.surfaces[0].witnesses = missingJade.surfaces[0].witnesses.filter(
    (witness) => !witness.dialects.includes("jade"),
  );
  assert.throws(() => validateAudit(missingJade, decision), /missing dialect proof/u);

  const falseClosure = structuredClone(original) as unknown as {
    surfaces: Array<{ dialectState: Record<Dialect, string> }>;
  };
  falseClosure.surfaces[0].dialectState.pug = "supported";
  assert.throws(() => validateAudit(falseClosure as unknown as Audit, decision));

  const forgedFixture = structuredClone(original);
  forgedFixture.fixtures[0].sha256 = "0".repeat(64);
  assert.throws(() => validateAudit(forgedFixture, decision), /bytes drifted/u);

  assert.throws(
    () => validateAudit(original, decision.replace("Real Vite client and SSR", "Vite")),
    /decision table drifted/u,
  );
});
