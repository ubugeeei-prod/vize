import { readToolingProgram, toolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const helper = path.join(root, "npm/native/native-binding-identity.js");

function probe(script: string): void {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-identity-"));
  try {
    // Exercise real filesystem paths through both subprocess boundaries. A
    // literal backslash on Unix also exposes accidental JavaScript escaping.
    const helperDirectory = path.join(directory, "helper with spaces");
    fs.mkdirSync(helperDirectory);
    const helperPath = path.join(
      helperDirectory,
      process.platform === "win32" ? "binding.cjs" : "native\\binding.cjs",
    );
    fs.copyFileSync(helper, helperPath);
    const filename = path.join(directory, "probe.cjs");
    fs.writeFileSync(
      filename,
      readToolingProgram("commands/native-binding-identity/probe-loaded-binding.cjs.in", {
        PROBE: script,
      }),
    );
    const result = spawnSync(process.execPath, [filename, helperPath], { encoding: "utf8" });
    assert.ifError(result.error);
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

test("identity belongs to the loaded image and its getter runs only once", () => {
  probe(
    readToolingProgram(
      "commands/native-binding-identity/observe-independent-binding-loads.cjs.in",
      {
        CHILD_PROGRAM: JSON.stringify(
          toolingProgram("commands/native-binding-identity/inspect-fresh-binding.cjs"),
        ),
      },
    ),
  );
});

test("uninstrumented and non-native providers fail toward a process identity", () => {
  probe(readToolingProgram("commands/native-binding-identity/reuse-cached-binding.cjs"));
});

test("a file replaced during successful loading retains the actual image token", () => {
  probe(readToolingProgram("commands/native-binding-identity/replace-loaded-image.cjs"));
});

test("OS image reuse survives require-cache eviction and independent helper copies", () => {
  probe(readToolingProgram("commands/native-binding-identity/reuse-os-image.cjs"));
});

test("cached deleted-path bindings preserve successful require and resolver failures refuse persistence", () => {
  probe(readToolingProgram("commands/native-binding-identity/resolve-deleted-binding.cjs"));
});
