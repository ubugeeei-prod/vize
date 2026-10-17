import assert from "node:assert/strict";
import { appendFile, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import {
  cleanup,
  evidence,
} from "../../../../../tools/support/release/public_acceptance/vrt_artifacts.ts";
import {
  VrtSecrets,
  writeVrtDiagnostic,
  type VrtPhase,
} from "../../../../../tools/support/release/public_acceptance/vrt_secrets.ts";

const [root, mode] = process.argv.slice(2);
assert.ok(root && mode);
const diagnostic = process.argv[4];
const scope = new VrtSecrets(
  diagnostic === "enabled" || diagnostic === "invalid"
    ? writeVrtDiagnostic
    : diagnostic === "broken"
      ? () => {
          throw new Error("private diagnostic sink failure");
        }
      : undefined,
);
const e = scope.evidence(await evidence(path.join(root, "evidence")));
// Deterministic control bytes are not credentials of any product session.
const a = Buffer.alloc(32, 2).toString("base64url");
const b = Buffer.alloc(32, 3).toString("base64url");
scope.remember(a);
scope.remember(b);
if (diagnostic === "invalid") scope.stage(a as VrtPhase);
if (diagnostic === "enabled" && mode === "scan-refusal") console.error(a);
try {
  let failure: unknown;
  try {
    await e.save("body.bin", Buffer.from([0, 255, 128, 10]));
    if (mode === "failure" || mode === "cleanup-failure")
      await readFile(path.join(root, "missing-acquisition"));
    if (mode === "writer-failure") await e.save("body.bin", "overwritten");
    if (mode === "scan-refusal")
      await writeFile(path.join(root, "injected-control.bin"), a, { flag: "wx" });
    const redacted = scope.sanitizeError(new AggregateError([new Error(a)], `Control ${b}`));
    await e.json("redacted.json", {
      message: String(redacted),
      stack: redacted instanceof Error ? redacted.stack : undefined,
    });
  } catch (error) {
    failure = scope.sanitizeError(error);
    await e.json("failure.json", {
      error: String(failure),
      stack: failure instanceof Error ? failure.stack : undefined,
    });
  } finally {
    try {
      await cleanup(e, "real-cleanup", [
        {
          name: "natural-cleanup-refusal",
          run: async () => {
            if (mode === "cleanup-failure") await rm(path.join(root, "already-removed"));
          },
        },
        {
          name: "later-retained-evidence",
          run: () => e.save("cleanup-end.bin", "cleanup completed"),
        },
      ]);
    } catch (error) {
      e.records.push({ phase: "cleanup-failure", error: String(scope.sanitizeError(error)) });
      failure ??= scope.sanitizeError(error);
    }
    await e.json("observations.json", { success: !failure, records: e.records });
  }
  if (failure) {
    console.error(failure);
    process.exitCode = 7;
  } else console.log(JSON.stringify({ control: mode, success: true }));
  if (mode === "terminal-write-failure")
    await writeFile(path.join(root, "evidence"), "terminal output", { flag: "wx" });
} catch (error) {
  console.error(scope.sanitizeError(error));
  process.exitCode ||= 9;
} finally {
  let pending: Promise<void> | undefined;
  if (mode === "pending") pending = e.save("pending.bin", Buffer.alloc(1024 * 1024, 11));
  try {
    await scope.seal(path.join(root, "vrt.log"), process.env.VRT_RETAINED_OUTPUT);
    await pending;
    assert.throws(() => e.save("late.bin", "late"), /writer already closed/);
  } catch {
    scope.failed();
    console.error("VRT retained evidence refused");
    process.exitCode ||= 1;
  } finally {
    scope.erase();
  }
}
assert.throws(() => scope.remember(a), /already closed/);
if (mode === "changed-after-barrier")
  await appendFile(path.join(root, "vrt.log"), "late safe suffix\n");
