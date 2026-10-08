import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "../../../performance/support/warm-type-backed-source.ts";
import { publicAuthority } from "./responsive-lint-public-authority.ts";
import { publicWire } from "./responsive-lint-public-wire.ts";
import { corpus, sourceHash, run } from "./responsive-lint-observations.ts";
import { root } from "./paths.ts";

const [input, callerPath, callerSha256, ...extra] = process.argv.slice(2);
assert.equal(extra.length, 0);
assert.ok(process.env.RUNNER_TEMP);
assert.equal(input, path.join(fs.realpathSync(process.env.RUNNER_TEMP), "warm-pair"));
const output = path.join(input, "responsive-lint-8002");
assert.equal(fs.existsSync(output), false, "this one replay owns a fresh output directory");
fs.mkdirSync(output);
const observed: Record<string, unknown> = {
  schema: "vize.lsp.responsive-lint-diagnostics.public.observation",
  version: 1,
  sourceHash,
  originalAuthority: corpus,
  scope:
    "same six original edit cycles and authored whole native positive; official installed Linux CLI",
  limits:
    "tiny original-case timings only; no ratio to source builds, universal speedup or native-lifetime claim",
};
const save = () =>
  fs.writeFileSync(path.join(output, "observation.json"), `${JSON.stringify(observed, null, 2)}\n`);
const priorCorsa = process.env.CORSA_PATH;
try {
  const callerBytes = fs.readFileSync(callerPath);
  fs.writeFileSync(path.join(output, "caller.json"), callerBytes);
  assert.match(callerSha256, /^[a-f0-9]{64}$/u);
  assert.equal(sha256(callerBytes), callerSha256);
  observed.callerSha256 = callerSha256;
  save();
  const authority = publicAuthority(input, callerBytes, output);
  observed.authority = authority;
  process.env.CORSA_PATH = authority.runtime.executable;
  async function side(typecheck: boolean) {
    const captureRoot = path.join(output, typecheck ? "native-sessions" : "control-sessions");
    const result = await run(typecheck, authority.runtime.executable, {
      binding: {
        repoRoot: root,
        binary: authority.installation.binary,
        published: {
          authority: "published-release",
          receiptPath: authority.launch.receiptPath,
          receiptSha256: authority.launch.receiptSha256,
          captureRoot,
        },
      },
      failureRoot: output,
    });
    observed[typecheck ? "native" : "control"] = result;
    save();
    assert.deepEqual(result.runtime.native, {
      executable: authority.runtime.executable,
      sha256: authority.runtime.binarySha256,
      manifest: authority.runtime.native.content,
    });
    assert.deepEqual(result.runtime.vue, authority.runtime.vue.content);
    assert.equal(result.gates.length, typecheck ? 4 : 0);
    const wire = await publicWire(captureRoot, result, authority.installation, typecheck);
    observed[typecheck ? "nativeWire" : "controlWire"] = wire;
    save();
    return result;
  }
  const control = await side(false);
  const native = await side(true);
  assert.equal(control.rows.length, 6);
  assert.equal(native.rows.length, 7);
  for (let index = 0; index < 6; index++) {
    assert.deepEqual(
      native.rows[index].early,
      {
        uri: native.uri,
        version: native.rows[index].version,
        diagnostics: control.rows[index].early.diagnostics,
      },
      "entire early public packet matches the independent typecheck-disabled control",
    );
  }
  observed.outcome = "whole original and authored control packets matched";
} catch (failure) {
  observed.failure = failure instanceof Error ? failure.stack : String(failure);
  throw failure;
} finally {
  if (priorCorsa === undefined) delete process.env.CORSA_PATH;
  else process.env.CORSA_PATH = priorCorsa;
  save();
}
