import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { sha256 } from "../differential/harness.mjs";
import { errorText } from "../differential/lsp-types.ts";
import {
  captureNativeAttributeSession,
  hoverProviders,
} from "./support/lsp/native-attribute-hover-capture.ts";
import {
  assertWholeNativeHovers,
  loadHoverCorpus,
  type HoverCapture,
} from "./support/lsp/native-attribute-hover-contract.ts";
import { root } from "./support/lsp/paths.ts";

await test("original native attribute hovers return all twelve complete physical-coordinate replies", async (t) => {
  const destination = path.join(root, "target/differential/native-attribute-hover-range.json");
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.rmSync(destination, { force: true });
  const sessions: HoverCapture[] = [];
  let setupFailure: string | null = null;
  const report: Record<string, unknown> = {
    schema: "vize.native-attribute-hover-range.observation",
    version: 1,
    scope:
      "four current-source typed sessions; original full historical contents unprovided; native/history credit zero",
    sessions,
  };
  try {
    const fixtureRoot = path.join(
      root,
      "tests/_fixtures/differential/lsp/native-attribute-hover-range-original",
    );
    const { corpus, manifestSha256 } = loadHoverCorpus(fixtureRoot);
    report.manifestSha256 = manifestSha256;
    report.source = corpus.source;
    report.initialization = corpus.initialization;
    const identity = expectedBuildIdentity(root);
    const binary = path.join(root, identity.binaryPath);
    const receiptBytes = fs.readFileSync(`${binary}.differential-build.json`);
    const receipt = JSON.parse(receiptBytes.toString("utf8")) as Record<string, unknown>;
    validateBuildReceipt(receipt, identity);
    report.build = { identity, receipt, receiptSha256: sha256(receiptBytes) as string };
    const tree = spawnSync("git", ["rev-parse", "HEAD^{tree}"], { cwd: root, encoding: "utf8" });
    assert.equal(tree.status, 0);
    report.sourceTree = tree.stdout.trim();
    const providers = hoverProviders(root);
    report.providers = providers;
    assert.equal(providers.runtimeProbe.status, 0, "native TypeScript version probe must succeed");
    assert.equal(providers.runtimeProbe.error, null);
    for (const session of corpus.sessions) {
      sessions.push(
        await captureNativeAttributeSession(binary, fixtureRoot, corpus, session, providers),
      );
    }
    report.buildAfter = expectedBuildIdentity(root);
    assert.deepEqual(report.buildAfter, identity, "source-built executable identity stays fixed");
    assert.equal(sha256(fs.readFileSync(providers.runtime.resolvedPath)), providers.runtime.sha256);
    assert.equal(
      sha256(fs.readFileSync(path.join(providers.vue.path, "package.json"))),
      providers.vue.manifestSha256,
      "actual Vue provider identity stays fixed",
    );
    // Persist every complete result and every raw frame before comparing any answer.
    fs.writeFileSync(destination, `${JSON.stringify({ ...report, setupFailure }, null, 2)}\n`);
    assertWholeNativeHovers(corpus, sessions);
    report.acceptedWholeResponses = 12;
  } catch (error) {
    setupFailure = errorText(error);
  } finally {
    fs.writeFileSync(destination, `${JSON.stringify({ ...report, setupFailure }, null, 2)}\n`);
  }
  t.diagnostic(`complete native hover and raw framing observation: ${destination}`);
  assert.equal(setupFailure, null, "every session and whole native response is required");
});
