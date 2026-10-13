import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "../typed_defaults/custody.mjs";
import { templateDirectory } from "./fixtures.mjs";
import { installPackageGraph } from "./package_graph.mjs";

export const dependencies = {
  vue: "3.5.42",
  "vue-router": "5.3.1",
  "@typescript/typescript-linux-x64": "7.0.2",
};
export async function installDependencies(consumer, output, source) {
  const { records, assertDependencies } = await installPackageGraph(consumer, output, dependencies);
  const references = JSON.parse(source.get(`${templateDirectory}/reference/custody.json`));
  for (const [member, pin] of Object.entries(references.declarationsAndLicense)) {
    const bytes = fs.readFileSync(
      path.join(consumer, "node_modules/vue-router", member.slice("package/".length)),
    );
    assert.deepEqual(bytes, source.get(`${templateDirectory}/reference/${pin.file}`));
  }
  assertDependencies();
  const provider = path.join(consumer, "node_modules/@typescript/typescript-linux-x64/lib/tsc");
  const probe = spawnSync(provider, ["--version"], { cwd: consumer, timeout: 30_000 });
  const providerObservation = {
    path: provider,
    sha256: sha256(fs.readFileSync(provider)),
    status: probe.status,
    signal: probe.signal,
    error: probe.error?.message ?? null,
    stdoutBase64: probe.stdout?.toString("base64"),
    stderrBase64: probe.stderr?.toString("base64"),
  };
  fs.writeFileSync(
    path.join(output, "provider.json"),
    JSON.stringify(providerObservation, null, 2) + "\n",
  );
  assert.equal(probe.error, undefined);
  assert.equal(probe.signal, null);
  assert.equal(probe.status, 0);
  assert.equal(probe.stdout.toString("utf8"), "Version 7.0.2\n");
  assert.equal(probe.stderr.length, 0);
  return { records, providerObservation, assertDependencies };
}
