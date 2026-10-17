import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";
import {
  VrtSecrets,
  vrtDiagnosticRecord,
  verifiedVrtLog,
  type VrtPhase,
  type VrtDisposition,
} from "../../tools/support/release/public_acceptance/vrt_secrets.ts";

const fixture = fileURLToPath(
  new URL("fixtures/release/vrt-log-barrier/observer.fixture.ts", import.meta.url),
);
const control = Buffer.alloc(32, 2).toString("base64url");

test("diagnostics accept only fixed enum values without echoing rejected input", () => {
  assert.deepEqual(JSON.parse(vrtDiagnosticRecord("seal-private-scan", "failed")), {
    schema: "vize-public-vrt-diagnostic-v1",
    phase: "seal-private-scan",
    disposition: "failed",
  });
  for (const value of [
    control,
    "/private/path",
    "https://private.invalid/token",
    "__proto__",
    "seal-private-scan\n",
    null,
    {},
    ["seal-private-scan"],
  ]) {
    for (const invoke of [
      () => vrtDiagnosticRecord(value as VrtPhase, "failed"),
      () => vrtDiagnosticRecord("seal-private-scan", value as VrtDisposition),
    ])
      assert.throws(invoke, (error: unknown) => {
        assert.equal(String(error), "Error: Invalid VRT diagnostic vocabulary");
        return true;
      });
  }
});

async function run(
  t: TestContext,
  mode: string,
  diagnostic = "enabled",
  pipe = false,
  retainedSymlink = false,
) {
  const root = await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(), "vize-vrt-diagnostic-")));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  const log = path.join(root, "vrt.log");
  let retained = "";
  if (retainedSymlink) {
    retained = path.join(root, "retained");
    await fs.mkdir(retained);
    const borrowed = path.join(root, "borrowed.bin");
    await fs.writeFile(borrowed, "unscannable borrowed bytes");
    await fs.symlink(borrowed, path.join(retained, "borrowed.bin"));
  }
  const fd = await fs.open(log, "wx", 0o600);
  const rows: Buffer[] = [];
  const child = spawn(process.execPath, [fixture, root, mode, diagnostic], {
    stdio: pipe ? ["ignore", "pipe", "pipe", "pipe"] : ["ignore", fd.fd, fd.fd, "pipe"],
    env: { ...process.env, VRT_RETAINED_OUTPUT: retained },
  });
  child.stdout?.resume();
  child.stderr?.resume();
  child.stdio[3]!.on("data", (bytes: Buffer) => rows.push(bytes));
  let code: number | null;
  try {
    code = await new Promise<number | null>((resolve, reject) => {
      const timer = setTimeout(() => {
        child.kill("SIGKILL");
        reject(new Error("Diagnostic control did not close"));
      }, 5_000);
      child.once("error", (error) => {
        clearTimeout(timer);
        reject(error);
      });
      child.once("close", (status, signal) => {
        clearTimeout(timer);
        assert.equal(signal, null);
        resolve(status);
      });
    });
  } finally {
    await fd.close();
  }
  const diagnostics = Buffer.concat(rows).toString();
  assert.equal(diagnostics.includes(control), false);
  assert.equal(diagnostics.includes(root), false);
  return {
    root,
    log,
    code,
    diagnostics,
    rows: diagnostics
      .trim()
      .split("\n")
      .filter(Boolean)
      .map((row) => JSON.parse(row)),
  };
}

test("safe fd3 diagnostics survive private scan refusal without publishing raw private bytes", async (t) => {
  const result = await run(t, "scan-refusal");
  assert.equal(result.code, 1);
  assert.equal((await fs.readFile(result.log, "utf8")).includes(control), true);
  assert.ok(
    result.rows.some((row) => row.phase === "seal-private-scan" && row.disposition === "failed"),
  );
  assert.equal(
    result.rows.some((row) => row.phase === "seal-complete"),
    false,
  );
  await assert.rejects(verifiedVrtLog(result.log));
});

test("descriptor refusal identifies the exact drain boundary and preserves refusal", async (t) => {
  const result = await run(t, "plain", "enabled", true);
  assert.equal(result.code, 1);
  assert.ok(
    result.rows.some((row) => row.phase === "seal-stdio-drain" && row.disposition === "failed"),
  );
  await assert.rejects(verifiedVrtLog(result.log));
});

test("an unscannable retained symlink names only its fixed scan phase and still refuses", async (t) => {
  const result = await run(t, "plain", "enabled", false, true);
  assert.equal(result.code, 1);
  assert.ok(
    result.rows.some((row) => row.phase === "seal-retained-scan" && row.disposition === "failed"),
  );
  assert.equal(result.diagnostics.includes("borrowed"), false);
  await assert.rejects(verifiedVrtLog(result.log));
});

test("both real closure callbacks run despite one failure and expose no private exception", async () => {
  const rows: Array<{ phase: string; disposition: string }> = [];
  const scope = new VrtSecrets((phase, disposition) =>
    rows.push(JSON.parse(vrtDiagnosticRecord(phase, disposition))),
  );
  const visited: string[] = [];
  await assert.rejects(
    scope.closeBarrier(
      () => {
        visited.push("surfaces");
        throw new Error(`${control} /private/path`);
      },
      () => {
        visited.push("children");
      },
    ),
    /Owned VRT closure barrier refused/u,
  );
  assert.deepEqual(visited, ["surfaces", "children"]);
  assert.ok(
    rows.some((row) => row.phase === "closure-trusted-surfaces" && row.disposition === "failed"),
  );
  assert.ok(
    rows.some((row) => row.phase === "closure-published-children" && row.disposition === "passed"),
  );
  assert.equal(JSON.stringify(rows).includes(control), false);
});

test("a broken diagnostic sink still completes pending writers, cleanup and the full seal", async (t) => {
  const result = await run(t, "pending", "broken");
  assert.equal(result.code, 1);
  assert.equal(result.diagnostics, "");
  assert.equal(
    (await fs.readFile(path.join(result.root, "evidence/pending.bin"))).length,
    1024 * 1024,
  );
  assert.equal(
    await fs.readFile(path.join(result.root, "evidence/cleanup-end.bin"), "utf8"),
    "cleanup completed",
  );
  await verifiedVrtLog(result.log);
});

test("successful diagnostics do not alter verified original bytes or authorize a modified suffix", async (t) => {
  const result = await run(t, "plain");
  assert.equal(result.code, 0);
  assert.ok(
    result.rows.some((row) => row.phase === "seal-complete" && row.disposition === "passed"),
  );
  assert.deepEqual(await verifiedVrtLog(result.log), await fs.readFile(result.log));
  await fs.appendFile(result.log, "changed after seal");
  await assert.rejects(verifiedVrtLog(result.log), /changed after observer exit/u);
});

test("the reviewed shell keeps fd3 public before redirecting both private log streams", () => {
  const file = new URL("../../.github/workflows/release-public-acceptance.yml", import.meta.url);
  return fs.readFile(file, "utf8").then((source) => {
    assert.match(source, /node vrt_observer\.ts[^\n]* 3>&2 > "\$PUBLIC_OUTPUT\/vrt\.log" 2>&1/u);
    assert.doesNotMatch(source, /node vrt_observer\.ts[^\n]*2>&1 3>&2/u);
  });
});
