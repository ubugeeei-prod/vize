import assert from "node:assert/strict";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync, readFileSync, mkdirSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  prepareColdScale,
  verifyColdDiagnostics,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale.ts";

// Independently equal public vize0.596 / native TS7.1 / vue-tsc3.3.13 packets.
const diagnostics = [
  "error:4:7 [TS2322] Type 'string' is not assignable to type 'number'.",
  "error:6:34 [TS2339] Property 'definitelyMissingMethod' does not exist on type 'HTMLButtonElement'.",
  "error:8:61 [TS2322] Type 'number' is not assignable to type 'string'.",
];
const packet = () => ({
  fileCount: 2500,
  errorCount: 3,
  warningCount: 0,
  files: [{ file: "src/Control.vue", diagnostics: [...diagnostics] }],
});

test("scale evidence preserves complete authored coordinates, codes and messages", () => {
  const cli = verifyColdDiagnostics(JSON.stringify(packet()), "cli", 2500, true);
  const native = diagnostics
    .map(
      (x) =>
        "src/Control.vue(" + x.replace(/^error:(\d+):(\d+) \[TS(\d+)\] /, "$1,$2): error TS$3: "),
    )
    .join("\n");
  assert.deepEqual(verifyColdDiagnostics(native, "mapper", 2500, true), cli);
  assert.deepEqual(verifyColdDiagnostics(native, "vue-tsc", 2500, true), cli);
});

test("skipped files, missing findings, wrong source locations and lost ref types fail", () => {
  for (const mutate of [
    (x: ReturnType<typeof packet>) => {
      x.fileCount--;
    },
    (x: ReturnType<typeof packet>) => {
      x.files[0].diagnostics.pop();
      x.errorCount--;
    },
    (x: ReturnType<typeof packet>) => {
      x.files[0].diagnostics[1] = x.files[0].diagnostics[1].replace("6:34", "6:33");
    },
    (x: ReturnType<typeof packet>) => {
      x.files[0].diagnostics[1] = x.files[0].diagnostics[1].replace("HTMLButtonElement", "unknown");
    },
    (x: ReturnType<typeof packet>) => {
      x.files[0].file = "src/Other.vue";
    },
  ]) {
    const x = packet();
    mutate(x);
    assert.throws(() => verifyColdDiagnostics(JSON.stringify(x), "cli", 2500, true), {
      code: "ERR_ASSERTION",
    });
  }
});

test("fixture preparation refuses to replace an existing evidence directory", () => {
  const root = mkdtempSync(join(tmpdir(), "cold-scale-protected-"));
  try {
    const witness = join(root, "evidence.txt");
    writeFileSync(witness, "retained");
    assert.throws(() => prepareColdScale(root, root, 2500, false), { code: "ERR_ASSERTION" });
    assert.equal(readFileSync(witness, "utf8"), "retained");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("Actions evidence packaging retains packets without following dependency-link loops", () => {
  const root = mkdtempSync(join(tmpdir(), "cold-scale-archive-"));
  try {
    const evidence = join(root, "cold-scale", "head-1");
    const project = join(evidence, "project");
    mkdirSync(join(project, "node_modules", "@ws"), { recursive: true });
    symlinkSync(project, join(project, "node_modules", "@ws", "loop"), "dir");
    const packet = '{"correctness":{"passed":true},"files":4323,"sfcFiles":2500}\n';
    writeFileSync(join(evidence, "measurement.json"), packet);
    writeFileSync(join(evidence, "stdout.txt"), "complete diagnostic bytes\n");
    const workflow = readFileSync(
      new URL("../../.github/workflows/cold-typechecker-scale.yml", import.meta.url),
      "utf8",
    );
    const match =
      /name: Package evidence without traversing workspace links\n[\s\S]*?run: \|\n([\s\S]*?)(?=      - uses:)/.exec(
        workflow,
      );
    assert.ok(match, "the real packaging step must be exercised");
    const script = match[1]
      .split("\n")
      .map((line) => line.replace(/^          /, ""))
      .join("\n");
    const packed = spawnSync("bash", ["-euc", script], {
      env: { ...process.env, RUNNER_TEMP: root },
      encoding: "utf8",
      timeout: 10000,
    });
    assert.equal(packed.status, 0, packed.stderr);
    const archive = join(root, "cold-scale-evidence.tgz");
    const listing = spawnSync("tar", ["-tzf", archive], { encoding: "utf8" });
    assert.equal(listing.status, 0, listing.stderr);
    assert.deepEqual(listing.stdout.trim().split("\n").sort(), [
      "cold-scale/",
      "cold-scale/head-1/",
      "cold-scale/head-1/measurement.json",
      "cold-scale/head-1/stdout.txt",
    ]);
    const restored = spawnSync("tar", ["-xOf", archive, "cold-scale/head-1/measurement.json"], {
      encoding: "utf8",
    });
    assert.equal(restored.status, 0, restored.stderr);
    assert.equal(restored.stdout, packet);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
