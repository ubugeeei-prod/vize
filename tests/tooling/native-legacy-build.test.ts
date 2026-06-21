import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(filePath: string): string {
  return fs.readFileSync(path.join(root, filePath), "utf-8");
}

function expectedNapiArgs(crateName: string, features: string): string {
  return [
    "exec",
    "napi",
    "build",
    "--platform",
    "--release",
    "--manifest-path",
    `../../crates/${crateName}/Cargo.toml`,
    "-p",
    crateName,
    "--features",
    features,
    "--output-dir",
    ".",
    "--target",
    "x86_64-unknown-linux-gnu",
  ].join("\n");
}

test("vize native package builds with legacy Vue support", () => {
  const packageJson = JSON.parse(readRepoFile("npm/native/package.json")) as {
    scripts?: Record<string, string>;
  };

  assert.match(packageJson.scripts?.["build:ci"] ?? "", /--features napi,legacy(?:\s|$)/);
  assert.match(readRepoFile("npm/native/scripts/build-local.mjs"), /"--features",\s+"napi,legacy"/);
  assert.match(
    readRepoFile("crates/vize_vitrine/Cargo.toml"),
    /^legacy = \["vize\/legacy", "vize_canon\/legacy"\]$/m,
  );
});

test("github/build_napi_package keeps legacy features scoped to vize_vitrine", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "native-legacy-build-"));
  const binDir = path.join(tempDir, "bin");
  const packageDir = path.join(tempDir, "npm", "native");
  const argsPath = path.join(tempDir, "vp-args.txt");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    fs.mkdirSync(packageDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "vp",
      `require("node:fs").writeFileSync(${JSON.stringify(argsPath)}, process.argv.slice(2).join("\\n"));`,
    );

    const runForCrate = (crateName: string): string => {
      const result = runMoonScript(
        "github/build_napi_package",
        [packageDir, `../../crates/${crateName}/Cargo.toml`, crateName, "x86_64-unknown-linux-gnu"],
        {
          cwd: tempDir,
          env: {
            PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
          },
        },
      );

      assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
      return fs.readFileSync(argsPath, "utf-8");
    };

    assert.equal(runForCrate("vize_vitrine"), expectedNapiArgs("vize_vitrine", "napi,legacy"));
    assert.equal(runForCrate("vize_fresco"), expectedNapiArgs("vize_fresco", "napi"));
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
