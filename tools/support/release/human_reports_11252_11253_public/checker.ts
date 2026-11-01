import assert from "node:assert/strict";
import { readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { fileImage } from "../computed_event_custody.ts";
import { capture, cleanEnvironment } from "./capture.ts";
import type { CheckerEvidence, InstallReceipt } from "./model.ts";
import { bytes, object, text, wholeJson } from "./value.ts";
import { discard, type Sink } from "./evidence.ts";
export function identifyLauncher(
  id: string,
  install: InstallReceipt,
  output: string,
  sink: Sink = discard,
): CheckerEvidence {
  const provider = id.includes("tsrs")
    ? ["tsrs", "tsrs"]
    : id === "original-vue-tsc"
      ? ["vue-tsc", "vue-tsc"]
      : id.includes("ts7") || id === "original-pnpm-check"
        ? ["typescript", "tsc"]
        : null;
  if (!provider)
    return {
      state: "UNKNOWN",
      reason:
        "CLI checker/backend selection is not established by installed packages or an independent native image probe",
    };
  const [name, bin] = provider;
  assert(name && bin);
  const root = realpathSync(join(install.root, "node_modules", name));
  const manifest = object(wholeJson(readFileSync(join(root, "package.json"))));
  const entry = typeof manifest.bin === "string" ? manifest.bin : text(object(manifest.bin)[bin]);
  const executable = fileImage(join(root, entry));
  assert(
    install.packages.some(
      (p) =>
        p.name === name &&
        p.files.some((f) => f.path === executable.path && f.sha256 === executable.sha256),
    ),
  );
  const observed = capture(
    install.pnpm.path,
    ["exec", bin, "--version"],
    install.root,
    cleanEnvironment(sink),
    output,
    `${id}-launcher-version`,
  );
  assert.equal(observed.error, null);
  assert.equal(observed.signal, null);
  assert.equal(observed.status, 0);
  return {
    state: "LAUNCHER_OBSERVED",
    packageName: name,
    version: text(manifest.version),
    executable,
    versionObservation: JSON.stringify({
      ...observed,
      stdout: bytes(observed.stdoutBase64).toString(),
      stderr: bytes(observed.stderrBase64).toString(),
    }),
    backend: "UNKNOWN",
  };
}
