import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { assertNativeCase, type RunHost } from "./content-mapper-instance-globals-hosts-10617.ts";
import { sha, type GlobalCase } from "./content-mapper-instance-globals-10617.ts";

type ProfileControl = {
  id: string;
  entry?: GlobalCase;
  caseId?: string;
  mode: "enabled" | "template-checking-disabled";
  uniqueScopeExpressions: number;
  minimumHits: number;
  minimumOutputBuffers: number;
  diagnosticPrefixes: number;
};
type SourceProfile = {
  schema_version: number;
  tool: string;
  command: string;
  truncation: { dropped_spans: number; dropped_counters: number };
  spans: Array<{ key: string; count: number }>;
  counters: Array<{ key: string; samples: number; total: number; min: number; max: number }>;
};
const namespace = "canon.instance_global_rewrite";
const counterNames = [
  "parse_attempts",
  "source_buffers",
  "cache_hits",
  "cache_misses",
  "cache_entries",
  "output_buffers",
  "diagnostic_prefixes",
] as const;

export function assertRewriteProfile(
  profile: SourceProfile,
  control: Pick<
    ProfileControl,
    "uniqueScopeExpressions" | "minimumHits" | "minimumOutputBuffers" | "diagnosticPrefixes"
  >,
) {
  assert.equal(profile.schema_version, 1);
  assert.equal(profile.tool, "vize");
  assert.equal(profile.command, "check");
  assert.deepEqual(
    profile.truncation,
    { dropped_spans: 0, dropped_counters: 0 },
    "missing counters cannot mean zero after profile truncation",
  );
  const records = new Map(profile.counters.map((row) => [row.key, row]));
  assert.equal(records.size, profile.counters.length);
  const totals = Object.fromEntries(
    counterNames.map((name) => {
      const row = records.get(`${namespace}.${name}`);
      if (row) {
        assert.ok(Number.isSafeInteger(row.total) && row.total > 0);
        assert.deepEqual(
          { samples: row.samples, min: row.min, max: row.max },
          { samples: row.total, min: 1, max: 1 },
        );
      }
      return [name, row?.total ?? 0];
    }),
  ) as Record<(typeof counterNames)[number], number>;
  for (const name of ["parse_attempts", "source_buffers", "cache_misses", "cache_entries"] as const)
    assert.equal(totals[name], control.uniqueScopeExpressions, name);
  assert.equal(
    totals.diagnostic_prefixes,
    control.diagnosticPrefixes,
    "every physical prefix emission keeps its own diagnostic ownership",
  );
  const parse = profile.spans.filter((row) => row.key === `${namespace}.parse`);
  assert.ok(parse.length <= 1);
  assert.equal(
    parse[0]?.count ?? 0,
    control.uniqueScopeExpressions,
    "one valid grammar attempt per admitted scope/text cache miss",
  );
  if (control.uniqueScopeExpressions === 0) {
    for (const name of counterNames) assert.equal(totals[name], 0, name);
  } else {
    assert.ok(
      totals.cache_hits >= control.minimumHits,
      "repeated requests must reuse the scoped relative result",
    );
    assert.ok(
      totals.output_buffers >= control.minimumOutputBuffers,
      "each emitted copy still needs its own output buffer and diagnostic ownership",
    );
  }
  return totals;
}

export function runRewriteProfileControls(
  files: Map<string, string>,
  cases: GlobalCase[],
  source: string,
  output: string,
  binary: string,
  beta: string,
  ts5: string,
  run: RunHost,
) {
  const data = JSON.parse(files.get("profile-controls.json")!) as {
    schema: string;
    version: number;
    controls: ProfileControl[];
  };
  assert.equal(data.schema, "vize.instance-global-profile-controls");
  assert.equal(data.version, 1);
  assert.deepEqual(
    data.controls.map((row) => row.id),
    ["no-dollar", "disabled", "same-scope-repeat", "actual-scope-isolation"],
  );
  const root = path.join(output, "profile-controls");
  fs.mkdirSync(root, { recursive: true });
  const summaries: unknown[] = [];
  const checks: Array<{ id: string; run: () => void }> = [];
  const retain = () =>
    fs.writeFileSync(path.join(root, "summary.json"), `${JSON.stringify(summaries, null, 2)}\n`);
  for (const control of data.controls) {
    assert.notEqual(Boolean(control.entry), Boolean(control.caseId));
    const entry = control.entry ?? cases.find((row) => row.id === control.caseId);
    assert.ok(entry, control.id);
    fs.writeFileSync(path.join(source, "src/Globals.vue"), entry.source);
    fs.writeFileSync(path.join(source, "src/globals.d.ts"), entry.globals);
    fs.writeFileSync(path.join(source, "oracle/Context.ts"), entry.oracle);
    for (const [name, content] of [
      ["Globals.vue", entry.source],
      ["globals.d.ts", entry.globals],
      ["Context.ts", entry.oracle],
    ])
      fs.writeFileSync(path.join(root, `${control.id}-${name}.txt`), content);
    const context = run(
      `profile/${control.id}/independent-beta-context`,
      beta,
      ["--noEmit", "--pretty", "false", "-p", "oracle.tsconfig.json"],
      source,
    );
    const independent = run(
      `profile/${control.id}/independent-5.9-context`,
      process.execPath,
      [ts5, "--noEmit", "--pretty", "false", "-p", "oracle.tsconfig.json"],
      source,
    );
    const profileFile = path.join(root, `${control.id}.profile.json`);
    const disabled = control.mode === "template-checking-disabled";
    const args = [
      "check",
      "src",
      "--tsconfig",
      "tsconfig.json",
      "--format",
      "json",
      "--quiet",
      "--corsa-path",
      beta,
      "--profile-json",
      profileFile,
    ];
    if (disabled) args.push("--no-check-template-bindings");
    assert.equal(args.includes("--no-check-template-bindings"), disabled);
    const actual = run(`profile/${control.id}/source-check`, binary, args, source);
    let profile: SourceProfile | undefined;
    let profileError: string | undefined;
    try {
      profile = JSON.parse(fs.readFileSync(profileFile, "utf8")) as SourceProfile;
    } catch (error) {
      profileError = String(error);
    }
    const summary = {
      id: control.id,
      mode: control.mode,
      sourceSha256: sha(entry.source),
      globalsSha256: sha(entry.globals),
      oracleSha256: sha(entry.oracle),
      source: actual,
      independent: [context, independent],
      profile,
      profileError,
      totals: undefined as ReturnType<typeof assertRewriteProfile> | undefined,
      credit: disabled
        ? "Explicitly disabled template checking; no clean-program acceptance credit"
        : "Source operation-count control and complete native diagnostic report",
    };
    summaries.push(summary);
    retain();
    checks.push({
      id: `profile/${control.id}`,
      run() {
        try {
          assert.deepEqual(
            {
              status: independent.status,
              signal: independent.signal,
              error: independent.error,
              stdout: independent.stdout,
              stderr: independent.stderr,
            },
            {
              status: context.status,
              signal: context.signal,
              error: context.error,
              stdout: context.stdout,
              stderr: context.stderr,
            },
          );
          assertNativeCase(
            entry.source,
            entry,
            context,
            actual,
            JSON.parse(files.get("project/tsconfig.json.txt")!).compilerOptions,
            control.id,
            disabled,
          );
          assert.ok(profile, profileError);
          summary.totals = assertRewriteProfile(profile, control);
        } finally {
          retain();
        }
      },
    });
  }
  return checks;
}

// Synthetic comparator laws only; these objects never count as source measurements.
export function assertProfileComparatorNegatives() {
  const control = {
    uniqueScopeExpressions: 0,
    minimumHits: 0,
    minimumOutputBuffers: 0,
    diagnosticPrefixes: 0,
  };
  const empty = {
    schema_version: 1,
    tool: "vize",
    command: "check",
    truncation: { dropped_spans: 0, dropped_counters: 0 },
    spans: [],
    counters: [] as Array<{
      key: string;
      samples: number;
      total: number;
      min: number;
      max: number;
    }>,
  };
  assertRewriteProfile(empty, control);
  for (const name of [
    "parse_attempts",
    "source_buffers",
    "cache_hits",
    "cache_misses",
    "cache_entries",
    "output_buffers",
    "diagnostic_prefixes",
  ])
    assert.throws(() =>
      assertRewriteProfile(
        {
          ...empty,
          counters: [
            { key: `canon.instance_global_rewrite.${name}`, total: 1, samples: 1, min: 1, max: 1 },
          ],
        },
        control,
      ),
    );
  assert.throws(() =>
    assertRewriteProfile(
      { ...empty, truncation: { dropped_spans: 0, dropped_counters: 1 } },
      control,
    ),
  );
  assert.throws(() => assertRewriteProfile(empty, { ...control, uniqueScopeExpressions: 2 }));
  const positive = {
    ...empty,
    spans: [{ key: `${namespace}.parse`, count: 2 }],
    counters: counterNames.map((name) => ({
      key: `${namespace}.${name}`,
      total: 2,
      samples: 2,
      min: 1,
      max: 1,
    })),
  };
  const scoped = {
    uniqueScopeExpressions: 2,
    minimumHits: 2,
    minimumOutputBuffers: 2,
    diagnosticPrefixes: 2,
  };
  assertRewriteProfile(positive, scoped);
  for (const name of counterNames) {
    const collapsed = structuredClone(positive);
    const row = collapsed.counters.find((item) => item.key === `${namespace}.${name}`)!;
    row.total = row.samples = 1;
    assert.throws(() => assertRewriteProfile(collapsed, scoped));
  }
  assert.throws(() =>
    assertRewriteProfile({ ...positive, spans: [{ key: `${namespace}.parse`, count: 1 }] }, scoped),
  );
}
