import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { sha256 } from "./providers.mjs";
import {
  BUILD_RECIPE,
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../differential/build-receipt.ts";

export function sourceIdentity(repoRoot, observerFile) {
  assert.equal(process.env.VIZE_REQUIRE_NATIVE_RUNTIME_SELECTION, "1");
  assert.equal(
    process.platform,
    "linux",
    "actual native syscall capture requires its Linux Actions host",
  );
  assert.equal(
    process.arch,
    "x64",
    "the qualified 7.0.2 image is the explicitly owned Linux x64 provider",
  );
  const requested = process.env.VIZE_SOURCE_CLI ?? process.env.SOURCE_CLI;
  assert.ok(requested && path.isAbsolute(requested));
  if (process.env.VIZE_SOURCE_CLI && process.env.SOURCE_CLI)
    assert.equal(
      fs.realpathSync(process.env.VIZE_SOURCE_CLI),
      fs.realpathSync(process.env.SOURCE_CLI),
    );
  const executable = fs.realpathSync(requested);
  assert.equal(
    executable,
    fs.realpathSync(path.join(repoRoot, binaryRelativePath())),
    "only the original installed canonical source artifact is admitted",
  );
  const receiptFile = `${executable}.differential-build.json`;
  const receipt = JSON.parse(fs.readFileSync(receiptFile, "utf8"));
  const expected = expectedBuildIdentity(repoRoot);
  validateBuildReceipt(receipt, expected, BUILD_RECIPE);
  const probe = spawnSync(executable, ["--version"], { encoding: null });
  assert.equal(probe.error, undefined);
  assert.equal(probe.status, 0);
  assert.equal(probe.signal, null);
  assert.equal(probe.stdout.toString().trim(), receipt.cliVersion);
  assert.equal(probe.stderr.length, 0);
  const observer = {
    file: path.relative(repoRoot, observerFile),
    sha256: sha256(fs.readFileSync(observerFile)),
    revision: receipt.sourceRevision,
  };
  assert.equal(
    sha256(
      execFileSync("git", ["show", `${observer.revision}:${observer.file}`], { cwd: repoRoot }),
    ),
    observer.sha256,
  );
  return {
    executable,
    receiptFile,
    receipt,
    observer,
    versionProbe: {
      stdoutBase64: probe.stdout.toString("base64"),
      stderrBase64: probe.stderr.toString("base64"),
      status: probe.status,
      signal: probe.signal,
    },
  };
}

export function executionAuthority(repoRoot, source, consumer, environment) {
  const loader = path.join(repoRoot, "crates/vize_carton/src/config/loader");
  const rust = fs.readFileSync(path.join(loader, "js.rs"), "utf8");
  const embedded =
    /let script = \[\s*include_str!\("vite-runtime\.mjs"\),\s*include_str!\("module-runtime\.mjs"\),\s*r#"([\s\S]*?)"#,\s*\]\s*\.concat\(\);/u.exec(
      rust,
    );
  assert.ok(embedded, "exact current source config script recipe");
  assert.match(
    fs.readFileSync(path.join(loader, "lsp_snapshot.rs"), "utf8"),
    /builder\.prefix\("vize-config-compile-"\)/u,
  );
  const escape = (value) => value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  return {
    source: { path: source.executable, sha256: source.receipt.binarySha256 },
    node: {
      path: path.join(environment.PATH, "node"),
      physicalPath: fs.realpathSync(process.execPath),
      sha256: sha256(fs.readFileSync(process.execPath)),
    },
    configScript:
      fs.readFileSync(path.join(loader, "vite-runtime.mjs"), "utf8") +
      fs.readFileSync(path.join(loader, "module-runtime.mjs"), "utf8") +
      embedded[1],
    configFile: fs.realpathSync(path.join(consumer.configRoot, "vite.config.mjs")),
    configCachePattern: new RegExp(
      `^${escape(environment.TMPDIR ?? os.tmpdir())}/vize-config-compile-[A-Za-z0-9]+$`,
      "u",
    ),
    sdk: consumer.expectedExecutable,
  };
}
