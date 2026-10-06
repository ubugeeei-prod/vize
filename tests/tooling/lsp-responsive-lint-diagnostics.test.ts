import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { root } from "./support/lsp/paths.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { corpus, sourceHash, run } from "./support/lsp/responsive-lint-observations.ts";

await test("original edit lint and repairs publish before a gated genuine native type pass", async (t) => {
  if (process.platform !== "linux") {
    t.skip("real Linux native PID/birth/executable gate; other session laws remain portable");
    return;
  }
  const corsa = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "actual native diagnostics runtime",
    "native runtime unavailable",
  );
  if (!corsa) return;
  const binary = path.join(root, "target/ci/vize");
  resolveVizeLaunchCommand(undefined, binary, { required: true });
  const prior = {
    binary: process.env.VIZE_LSP_BIN,
    required: process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD,
    corsa: process.env.CORSA_PATH,
  };
  process.env.VIZE_LSP_BIN = binary;
  process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = "1";
  process.env.CORSA_PATH = corsa;
  const output = path.join(root, "target/differential/responsive-lint-diagnostics-8002.json");
  const observed: {
    control?: Awaited<ReturnType<typeof run>>;
    native?: Awaited<ReturnType<typeof run>>;
    failure?: string;
  } = {};
  try {
    observed.control = await run(false, corsa);
    observed.native = await run(true, corsa);
    assert.equal(observed.control.rows.length, 6);
    assert.equal(observed.native.rows.length, 7);
    for (let index = 0; index < 6; index++) {
      assert.deepEqual(
        observed.native.rows[index].early,
        {
          uri: observed.native.uri,
          version: observed.native.rows[index].version,
          diagnostics: observed.control.rows[index].early.diagnostics,
        },
        "entire early public packet matches the independent typecheck-disabled control",
      );
    }
  } catch (failure) {
    observed.failure = failure instanceof Error ? failure.stack : String(failure);
    throw failure;
  } finally {
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(
      output,
      JSON.stringify(
        {
          schema: "vize.lsp.responsive-lint-diagnostics.observation",
          version: 1,
          sourceHash,
          originalAuthority: corpus,
          scope:
            "original three remove/restore cycles, whole public packets, real stopped native process; no universal speedup claim",
          ...observed,
        },
        null,
        2,
      ),
    );
    if (prior.binary === undefined) delete process.env.VIZE_LSP_BIN;
    else process.env.VIZE_LSP_BIN = prior.binary;
    if (prior.required === undefined) delete process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
    else process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = prior.required;
    if (prior.corsa === undefined) delete process.env.CORSA_PATH;
    else process.env.CORSA_PATH = prior.corsa;
  }
});
