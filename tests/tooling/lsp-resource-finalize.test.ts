import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { finalizeLspResources } from "../../tools/support/compat/davinci/lib/lsp-resource-finalize.ts";

function fixture() {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-finalize-"));
  const profile = path.join(directory, "profile.json");
  const events: string[] = [];
  const result: {
    profile_json?: { path: string; bytes: number; sha256: string };
    finalization_error?: string;
  } = {};
  const options = {
    session: {
      async shutdown() {
        events.push("shutdown");
      },
      async kill() {
        events.push("kill");
      },
      async waitForClosed() {
        events.push("closed");
      },
      exitedGracefully: true,
    },
    profilePath: profile as string | null,
    workloadFailed: false,
    result,
    cleanup() {
      events.push("cleanup");
      fs.rmSync(directory, { recursive: true, force: true });
    },
    persist() {
      events.push("persist");
    },
  };
  return { directory, profile, events, result, options };
}

await test("graceful profile exports preserve actual bytes and always clean before persistence", async () => {
  const { profile, directory, events, result, options } = fixture();
  const bytes = Buffer.from('{"command":"lsp","spans":[]}\n');
  fs.writeFileSync(profile, bytes);
  await finalizeLspResources(options);
  assert.deepEqual(events, ["shutdown", "closed", "cleanup", "persist"]);
  assert.equal(fs.existsSync(directory), false);
  assert.deepEqual(result, {
    profile_json: {
      path: profile,
      bytes: bytes.byteLength,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    },
  });
});

await test("a missing profile preserves the original workload exception and cleans the workspace", async () => {
  const { directory, events, result, options } = fixture();
  const original = new Error("controlled edit returned an incorrect diagnostic");
  options.workloadFailed = true;
  await assert.rejects(
    async () => {
      try {
        throw original;
      } finally {
        await finalizeLspResources(options);
      }
    },
    (error) => error === original,
  );
  assert.deepEqual(events, ["shutdown", "closed", "cleanup", "persist"]);
  assert.equal(fs.existsSync(directory), false);
  assert.match(result.finalization_error ?? "", /ENOENT/);
  assert.equal(result.profile_json, undefined);
});

await test("a successful workload still fails when its required profile is missing", async () => {
  const { directory, events, result, options } = fixture();
  await assert.rejects(finalizeLspResources(options), /ENOENT/);
  assert.equal(fs.existsSync(directory), false);
  assert.deepEqual(events, ["shutdown", "closed", "cleanup", "persist"]);
  assert.match(result.finalization_error ?? "", /ENOENT/);
});

await test("a forced shutdown cannot admit a profile even when a report file exists", async () => {
  const { profile, events, result, options } = fixture();
  fs.writeFileSync(profile, '{"command":"lsp"}');
  options.session.shutdown = async () => {
    events.push("shutdown");
    throw new Error("shutdown request failed");
  };
  await assert.rejects(finalizeLspResources(options), /shutdown request failed/);
  assert.deepEqual(events, ["shutdown", "kill", "cleanup", "persist"]);
  assert.equal(result.profile_json, undefined);
});

await test("a killed or nonzero server exit fails profile admission after EOF", async () => {
  const { profile, events, result, options } = fixture();
  fs.writeFileSync(profile, '{"command":"lsp"}');
  options.session.exitedGracefully = false;
  await assert.rejects(finalizeLspResources(options), /must exit successfully/);
  assert.deepEqual(events, ["shutdown", "closed", "cleanup", "persist"]);
  assert.equal(result.profile_json, undefined);
});

await test("nonprofiled runs retain shutdown fallback and still persist after cleanup", async () => {
  const { events, result, options } = fixture();
  options.profilePath = null;
  options.session.shutdown = async () => {
    events.push("shutdown");
    throw new Error("shutdown request failed");
  };
  await finalizeLspResources(options);
  assert.deepEqual(events, ["shutdown", "kill", "cleanup", "persist"]);
  assert.deepEqual(result, {});
});
