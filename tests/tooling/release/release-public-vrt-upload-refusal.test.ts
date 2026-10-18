import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdir, mkdtemp, open, readFile, realpath, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";
import { inventory } from "../../../tools/support/release/public_acceptance/vrt_artifacts.ts";

const fixture = fileURLToPath(
  new URL("../fixtures/release/vrt-log-barrier/observer.fixture.ts", import.meta.url),
);
const publisher = fileURLToPath(
  new URL("../../../tools/support/release/public_acceptance/vrt_retained.ts", import.meta.url),
);
const control = Buffer.alloc(32, 2).toString("base64url");

async function staged(t: TestContext) {
  const root = await realpath(await mkdtemp(path.join(os.tmpdir(), "vize-vrt-upload-control-")));
  t.after(() => rm(root, { recursive: true, force: true }));
  const privateRoot = await mkdtemp(path.join(root, "vize-public-vrt."));
  const retained = path.join(root, "public-release");
  await mkdir(retained);
  for (const file of ["native.json", "musea.json"])
    await writeFile(path.join(retained, file), `${file}: whole original safe bytes\n`);
  const output = path.join(root, "github-output");
  await writeFile(output, "", { flag: "wx" });
  return { privateRoot, retained, output };
}

async function closed(child: ReturnType<typeof spawn>): Promise<number | null> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      child.kill("SIGKILL");
      reject(new Error("Owned upload control did not close"));
    }, 5_000);
    child.once("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.once("close", (code) => {
      clearTimeout(timer);
      resolve(code);
    });
  });
}

async function observe(privateRoot: string, retained: string, mode = "plain") {
  const log = path.join(privateRoot, "vrt.log");
  const fd = await open(log, "wx", 0o600);
  try {
    const child = spawn(process.execPath, [fixture, privateRoot, mode], {
      stdio: ["ignore", fd.fd, fd.fd],
      env: { ...process.env, VRT_RETAINED_OUTPUT: retained },
    });
    return { log, code: await closed(child) };
  } finally {
    await fd.close();
  }
}

async function publish(log: string, retained: string, output: string) {
  const child = spawn(process.execPath, [publisher, log, retained], {
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, GITHUB_OUTPUT: output },
  });
  const stdout: Buffer[] = [],
    stderr: Buffer[] = [];
  child.stdout?.on("data", (chunk) => stdout.push(chunk));
  child.stderr?.on("data", (chunk) => stderr.push(chunk));
  const code = await closed(child);
  return {
    code,
    stdout: Buffer.concat(stdout),
    stderr: Buffer.concat(stderr),
    output: await readFile(output, "utf8"),
  };
}

async function originals(retained: string) {
  for (const file of ["native.json", "musea.json"])
    assert.equal(
      await readFile(path.join(retained, file), "utf8"),
      `${file}: whole original safe bytes\n`,
    );
}
async function safeTree(retained: string) {
  for (const item of await inventory(retained)) {
    assert.equal(item.path.includes(control), false);
    assert.equal(
      (await readFile(path.join(retained, item.path))).includes(Buffer.from(control)),
      false,
    );
  }
}

// These are real owned file/process controls; no upload, credential session, browser or tunnel runs.
test("retained body and filename control bytes are quarantined before refusing acceptance and authorizing safe originals", async (t) => {
  for (const kind of ["body", "name"]) {
    const f = await staged(t);
    const name = kind === "body" ? "added-campaign.bin" : `${control}.bin`;
    await writeFile(
      path.join(f.retained, name),
      kind === "body" ? control : "safe bytes under unsafe name",
    );
    const { log, code } = await observe(f.privateRoot, f.retained);
    assert.equal(code, 1);
    const quarantine = await inventory(path.join(f.privateRoot, "vrt-quarantined-retained"));
    assert.equal(quarantine.length, 1);
    assert.equal(quarantine[0].path, "0");
    assert.equal(
      await readFile(path.join(f.privateRoot, "vrt-quarantined-retained/0"), "utf8"),
      kind === "body" ? control : "safe bytes under unsafe name",
    );
    await assert.rejects(readFile(path.join(f.privateRoot, "vrt-log-barrier.json")), {
      code: "ENOENT",
    });
    const result = await publish(log, f.retained, f.output);
    assert.equal(result.code, 1);
    assert.equal(result.stdout.length, 0);
    assert.equal(result.output, "upload-safe=true\n");
    await originals(f.retained);
    await safeTree(f.retained);
    assert.deepEqual(
      (await inventory(f.retained)).map((x) => x.path),
      ["musea.json", "native.json", "vrt-refused.json"],
    );
    await assert.rejects(readFile(log), { code: "ENOENT" });
  }
});

test("natural quarantine destination EEXIST leaves no safety receipt or upload authorization", async (t) => {
  const f = await staged(t);
  await writeFile(path.join(f.retained, "added-campaign.bin"), control);
  await writeFile(
    path.join(f.privateRoot, "vrt-quarantined-retained"),
    "occupied private destination",
  );
  const { log, code } = await observe(f.privateRoot, f.retained);
  assert.equal(code, 1);
  await assert.rejects(readFile(path.join(f.privateRoot, "vrt-retained-safe.json")), {
    code: "ENOENT",
  });
  const result = await publish(log, f.retained, f.output);
  assert.equal(result.code, 1);
  assert.equal(result.stdout.length, 0);
  assert.equal(result.output, "");
  assert.equal(result.stderr.includes(Buffer.from(control)), false);
  await originals(f.retained);
  assert.equal(await readFile(path.join(f.retained, "added-campaign.bin"), "utf8"), control);
  // The bytes remain only in a refused upload tree: the actual workflow guard below denies it.
});

test("missing, forged boolean-only, and post-scan changed retained inventories never authorize upload", async (t) => {
  for (const kind of ["missing", "forged", "changed"]) {
    const f = await staged(t);
    const { log, code } = await observe(f.privateRoot, f.retained, "scan-refusal");
    assert.equal(code, 1);
    const receipt = path.join(f.privateRoot, "vrt-retained-safe.json");
    if (kind === "missing") await rm(receipt);
    if (kind === "forged")
      await writeFile(
        receipt,
        JSON.stringify({
          schema: "vize-public-vrt-retained-safe-v1",
          privateRoot: f.privateRoot,
          retained: f.retained,
          ownedWritersClosed: true,
          allRetainedPathsAndBodiesScanned: true,
          quarantinedEntries: 0,
        }),
      );
    if (kind === "changed") await writeFile(path.join(f.retained, "added-campaign.bin"), control);
    const result = await publish(log, f.retained, f.output);
    assert.equal(result.code, 1);
    assert.equal(result.stdout.length, 0);
    assert.equal(result.output, "");
    await originals(f.retained);
  }
});

test("verified success and private refusal emit only the fixed safe-output flag", async (t) => {
  for (const mode of ["plain", "scan-refusal", "terminal-write-failure"]) {
    const f = await staged(t);
    const { log, code } = await observe(f.privateRoot, f.retained, mode);
    assert.equal(code, mode === "plain" ? 0 : mode === "scan-refusal" ? 1 : 9);
    const result = await publish(log, f.retained, f.output);
    assert.equal(result.code, mode === "scan-refusal" ? 1 : 0);
    assert.equal(result.output, "upload-safe=true\n");
    await originals(f.retained);
    await safeTree(f.retained);
  }
});

const uploadCondition =
  "if: ${{ always() && (steps.vrt.outcome == 'skipped' || ((steps.vrt.outcome == 'success' || steps.vrt.outcome == 'failure') && steps.vrt.outputs.upload-safe == 'true')) }}";
const uploader = "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a";
const diagnosticTitle = "Retain diagnostic public metadata first";
const completeTitle = "Retain actual public observations";
const diagnosticName = "public-diagnostics-${{ github.run_id }}-${{ github.run_attempt }}";
const completeName = "public-release-${{ github.run_id }}-${{ github.run_attempt }}";

function workflowStep(workflow: string, name: string) {
  const marker = `      - name: ${name}\n`;
  assert.equal(workflow.split(marker).length - 1, 1, `exactly one ${name}`);
  const start = workflow.indexOf(marker);
  const next = workflow.indexOf("\n      - ", start + marker.length);
  return workflow.slice(start, next < 0 ? undefined : next + 1);
}

function assertDiagnosticUploads(workflow: string) {
  assert.equal(workflow.split(uploadCondition).length - 1, 2);
  assert.equal(workflow.split("uses: actions/upload-artifact@").length - 1, 2);
  assert.equal(
    workflowStep(workflow, diagnosticTitle),
    `      - name: ${diagnosticTitle}
        ${uploadCondition}
        uses: ${uploader} # v7.0.1
        with:
          name: ${diagnosticName}
          path: |
            artifacts/public-release/**/*.json
            artifacts/public-release/**/*.jsonl
          if-no-files-found: error
          retention-days: 30
`,
    "diagnostics retain complete JSON/JSONL only, under the original safety gate",
  );
  assert.equal(
    workflowStep(workflow, completeTitle),
    `      - name: ${completeTitle}
        ${uploadCondition}
        uses: ${uploader} # v7.0.1
        with:
          name: ${completeName}
          path: artifacts/public-release
          if-no-files-found: error
          retention-days: 30
`,
    "the original complete artifact is mandatory and unchanged",
  );
  const seal = workflowStep(workflow, "Seal installed public acceptance");
  assert.ok(workflow.indexOf(seal) < workflow.indexOf(workflowStep(workflow, diagnosticTitle)));
  assert.ok(
    workflow.indexOf(workflowStep(workflow, diagnosticTitle)) <
      workflow.indexOf(workflowStep(workflow, completeTitle)),
    "diagnostic upload completes before the original large artifact upload",
  );
}

test("the actual upload condition fails closed for cancelled/incomplete VRT and permits only safe or precredential skipped output", async () => {
  const workflow = await readFile(
    new URL("../../../.github/workflows/release-public-acceptance.yml", import.meta.url),
    "utf8",
  );
  assertDiagnosticUploads(workflow);
  const cases = [
    ["cancelled", undefined, false],
    ["cancelled", "true", false],
    ["", undefined, false],
    ["", "true", false],
    ["failure", undefined, false],
    ["failure", "false", false],
    ["failure", "true", true],
    ["success", undefined, false],
    ["success", "false", false],
    ["success", "true", true],
    ["skipped", undefined, true],
    ["skipped", "false", true],
    ["skipped", "true", true],
  ] as const;
  for (const [outcome, flag, expected] of cases) {
    const allowed =
      outcome === "skipped" ||
      ((outcome === "success" || outcome === "failure") && flag === "true");
    assert.equal(allowed, expected, `${outcome}/${flag ?? "absent"}`);
  }
});

test("diagnostic transport rejects unsafe guards, partial evidence and replacement of the full artifact", async (t) => {
  const workflow = await readFile(
    new URL("../../../.github/workflows/release-public-acceptance.yml", import.meta.url),
    "utf8",
  );
  assertDiagnosticUploads(workflow);
  const diagnostic = workflowStep(workflow, diagnosticTitle);
  const complete = workflowStep(workflow, completeTitle);
  const mutations: Array<[string, (source: string) => string]> = [
    [
      "unguarded metadata cannot bypass credential quarantine",
      (source) => source.replace(diagnostic, diagnostic.replace(uploadCondition, "if: always()")),
    ],
    [
      "metadata cannot broaden to source archives, images or binaries",
      (source) =>
        source.replace(
          diagnostic,
          diagnostic.replace("artifacts/public-release/**/*.json\n", "artifacts/public-release\n"),
        ),
    ],
    [
      "complete protocol JSONL cannot be omitted",
      (source) =>
        source.replace(
          diagnostic,
          diagnostic.replace("            artifacts/public-release/**/*.jsonl\n", ""),
        ),
    ],
    [
      "small diagnostics cannot replace the full artifact",
      (source) => source.replace(complete, ""),
    ],
    [
      "the full artifact cannot become metadata-only",
      (source) =>
        source.replace(
          complete,
          complete.replace(
            "path: artifacts/public-release\n",
            "path: artifacts/public-release/**/*.json\n",
          ),
        ),
    ],
    [
      "diagnostics must be available before the full upload",
      (source) => source.replace(diagnostic + complete, complete + diagnostic),
    ],
  ];
  for (const [name, mutate] of mutations)
    await t.test(name, () => {
      const changed = mutate(workflow);
      assert.notEqual(changed, workflow, "the negative control must change the actual recipe");
      assert.throws(() => assertDiagnosticUploads(changed), assert.AssertionError);
    });
});
