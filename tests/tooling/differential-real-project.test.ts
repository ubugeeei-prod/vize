import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { loadProductManifest } from "../differential/harness.mjs";
import {
  ledgerPath,
  readDomCorpus,
  readShards,
  realProjectProducts,
  registryPath,
  writeRealProjectResults,
} from "../differential/real-project.ts";

const sourceRevision = "a".repeat(40);
const projects = [
  { id: "alpha", fixturePath: "tests/_fixtures/_git/alpha", revision: "1".repeat(40) },
  { id: "beta", fixturePath: "tests/_fixtures/_git/beta", revision: "2".repeat(40) },
  { id: "gamma", fixturePath: "tests/_fixtures/_git/gamma", revision: "3".repeat(40) },
];
const coverage = ["compiler", "linter", "typechecker", "formatter", "syntax-highlighter", "lsp"];
const tools = ["compiler", "typechecker", "linter", "formatter"];

function writeJson(file: string, value: unknown) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(value)}\n`);
}

function shard(
  dir: string,
  index: number,
  ids: string[],
  status = (_id: string, _tool: string) => "ok",
) {
  const shardDir = path.join(dir, `real-project-matrix-${index}`);
  const reported = ids.map((id) => {
    const project = projects.find((candidate) => candidate.id === id)!;
    return {
      id,
      fixturePath: project.fixturePath,
      revision: project.revision,
      runs: tools.map((tool) => {
        const outputPath = `real-project-results/shard-${index}/${id}-${tool}.json`;
        writeJson(path.join(shardDir, `${id}-${tool}.json`), { schema: "vize.fixtureToolRun" });
        return { tool, status: status(id, tool), exitCode: 0, fileCount: 3, outputPath };
      }),
    };
  });
  writeJson(path.join(shardDir, "summary.json"), {
    schema: "vize.fixtureToolMatrixReport",
    command: { shardIndex: index, shardCount: 2 },
    projects: reported,
  });
  writeJson(path.join(shardDir, "lsp-lifecycle-summary.json"), {
    schema: "vize.realProjectLspLifecycle",
    projects: ids.map((id) => ({
      id,
      status: "ok",
      vueFileCount: 3,
      actualFileDiagnostics: { count: 0, sha256: "d".repeat(64) },
    })),
  });
  return shardDir;
}

function withRoot(run: (root: string, artifacts: string, out: string) => void) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-real-project-"));
  try {
    writeJson(path.join(root, registryPath), {
      projects: projects.map((project) => ({ ...project, coverage })),
    });
    writeJson(path.join(root, ledgerPath), {
      fixtures: [
        {
          fixturePath: "tests/_fixtures/_git/alpha",
          dialectCoverage: {
            state: "partial",
            evidence: [
              { kind: "pinned-package", dialects: ["vue3-sfc"], file: "package.json" },
              { kind: "pinned-package", dialects: ["jsx-babel"], file: "docs/package.json" },
            ],
          },
        },
        {
          fixturePath: "tests/_fixtures/_git/beta",
          dialectCoverage: { state: "unknown", evidence: [] },
        },
      ],
    });
    run(root, path.join(root, "artifacts"), path.join(root, "out"));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

test("every pinned project gets one shared-format row per product target", () => {
  withRoot((root, artifacts, out) => {
    shard(artifacts, 0, ["alpha", "gamma"]);
    shard(artifacts, 1, ["beta"], (id, tool) => (tool === "typechecker" ? "configuration" : "ok"));
    const { outputs, markdown } = writeRealProjectResults({
      root,
      sourceRevision,
      shards: readShards(artifacts),
      outDir: out,
    });
    assert.deepEqual(
      outputs.map(({ product, target }) => `${product}/${target}`),
      realProjectProducts.map(({ product, target }) => `${product}/${target}`),
    );
    for (const { product, report, acceptance } of outputs) {
      const loaded = loadProductManifest(path.join(out, "T2", product, "manifest.json"), product);
      assert.equal(report.manifestSha256, loaded.manifestSha256);
      assert.deepEqual(
        loaded.cases.map((fixture: { projectId: string }) => fixture.projectId),
        ["alpha", "beta", "gamma"],
      );
      const native = product === "compiler" ? "missing-evidence" : "unsupported";
      assert(
        report.rows.every((row: any) => row.native.state === native),
        product,
      );
      assert.equal(acceptance.scope, "real-projects");
      assert.equal(acceptance.total.nativeHandled, 0);
      assert.equal(acceptance.total[product === "compiler" ? "unverified" : "unsupported"], 3);
      const expected = product === "typechecker" ? 2 : 3;
      assert.equal(report.summary.legacyCompleted, expected, product);
    }
    const typechecker = outputs.find(({ product }) => product === "typechecker")!.report;
    assert.equal(typechecker.rows[1].legacy.state, "configuration");
    const [alpha] = loadProductManifest(
      path.join(out, "T2/compiler/manifest.json"),
      "compiler",
    ).cases;
    assert.deepEqual(alpha.dialectCoverage, {
      state: "partial",
      registryRef: `${ledgerPath}#tests/_fixtures/_git/alpha`,
      evidence: [{ kind: "pinned-package", dialects: ["jsx-babel", "vue3-sfc"] }],
    });
    assert.match(markdown, /linter \| lint \| 3 \| 3 \| 0 \| 0/);
    assert.match(markdown, /Native-only acceptance \(real projects\)/);
  });
});

function domCorpus(root: string, divergences: number) {
  const dir = path.join(root, "dom-corpus");
  writeJson(path.join(dir, "summary.json"), {
    evidence: { canonicalScope: true, divergences },
  });
  writeJson(path.join(dir, "files.json"), [
    ["alpha/src/App.vue", "e".repeat(64), 10, "f".repeat(40)],
    ["alpha/src/Two.vue", "e".repeat(64), 10, "f".repeat(40)],
    ["gamma/App.vue", "e".repeat(64), 10, "f".repeat(40)],
  ]);
  return readDomCorpus(dir);
}

test("the compiler DOM lane reads the canonical corpus as legacy-backed", () => {
  withRoot((root, artifacts, out) => {
    shard(artifacts, 0, ["alpha", "beta", "gamma"]);
    const run = (divergences: number) =>
      writeRealProjectResults({
        root,
        sourceRevision,
        shards: readShards(artifacts),
        outDir: out,
        domCorpus: domCorpus(root, divergences),
      }).outputs.find(({ product }) => product === "compiler")!;
    const clean = run(0);
    assert.deepEqual(
      clean.report.rows.map((row: any) => [row.native.state, row.comparison.state]),
      [
        ["legacy-backed", "equal"],
        ["unverified", "not-compared"],
        ["legacy-backed", "equal"],
      ],
    );
    assert.equal(clean.report.rows[0].native.observation.files, 2);
    assert.equal(clean.report.summary.comparedEqual, 2);
    assert.equal(clean.acceptance.total.legacyBacked, 2);
    assert.equal(clean.acceptance.total.nativeHandled, 0);
    const diverged = run(1);
    assert.equal(diverged.report.rows[0].comparison.state, "not-attributed");
    assert.equal(diverged.report.summary.comparedEqual, 0);
  });
});

test("a missing shard or raw output still yields every planned row", () => {
  withRoot((root, artifacts, out) => {
    const dir = shard(artifacts, 0, ["alpha", "gamma"]);
    fs.rmSync(path.join(dir, "gamma-formatter.json"));
    const { outputs } = writeRealProjectResults({
      root,
      sourceRevision,
      shards: readShards(artifacts),
      outDir: out,
    });
    for (const { product, report } of outputs) {
      assert.equal(report.rows.length, 3);
      assert.equal(report.rows[1].legacy.state, "missing-evidence", product);
    }
    const formatter = outputs.find(({ product }) => product === "formatter")!.report;
    assert.equal(formatter.rows[2].legacy.state, "missing-evidence");
    assert.equal(formatter.summary.legacyCompleted, 1);
  });
});

test("forged or stitched shard evidence fails", () => {
  const cases: Array<[string, (artifacts: string) => void, RegExp]> = [
    [
      "duplicate project",
      (artifacts) => {
        shard(artifacts, 0, ["alpha"]);
        shard(artifacts, 1, ["alpha"]);
      },
      /alpha: reported by two shards/,
    ],
    [
      "drifted revision",
      (artifacts) => {
        const dir = shard(artifacts, 0, ["alpha"]);
        const file = path.join(dir, "summary.json");
        const summary = JSON.parse(fs.readFileSync(file, "utf8"));
        summary.projects[0].revision = "9".repeat(40);
        writeJson(file, summary);
      },
      /alpha: revision/,
    ],
    [
      "LSP row outside its shard",
      (artifacts) => {
        shard(artifacts, 0, ["alpha"]);
        const dir = shard(artifacts, 1, ["beta"]);
        const file = path.join(dir, "lsp-lifecycle-summary.json");
        const lsp = JSON.parse(fs.readFileSync(file, "utf8"));
        lsp.projects[0].id = "alpha";
        writeJson(file, lsp);
      },
      /alpha: LSP outside its shard/,
    ],
    [
      "output outside the shard report",
      (artifacts) => {
        const dir = shard(artifacts, 0, ["alpha"]);
        const file = path.join(dir, "summary.json");
        const summary = JSON.parse(fs.readFileSync(file, "utf8"));
        summary.projects[0].runs[0].outputPath = "real-project-results/shard-1/alpha-compiler.json";
        writeJson(file, summary);
      },
      /shard 0: output outside its report/,
    ],
  ];
  for (const [name, arrange, error] of cases) {
    withRoot((root, artifacts, out) => {
      arrange(artifacts);
      assert.throws(
        () =>
          writeRealProjectResults({
            root,
            sourceRevision,
            shards: readShards(artifacts),
            outDir: out,
          }),
        error,
        name,
      );
    });
  }
});
