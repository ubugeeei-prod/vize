import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { record } from "./braces-lock.ts";

/** Public installations have neither the workspace overrides nor its patches. */
export function assertPublicKitAudit(value: unknown, status: number | null): void {
  assert.equal(status, 0, "the external runtime Kit audit must succeed");
  const report = record(value);
  assert.deepEqual(Object.keys(report).sort(), ["advisories", "metadata"]);
  const counts = record(record(report.metadata).vulnerabilities);
  for (const severity of ["info", "low", "moderate", "high", "critical"]) {
    assert.equal(counts[severity], 0, `public Kit ${severity} findings remain fatal`);
  }
  assert.deepEqual(report.advisories, {}, "every public Kit finding remains fatal");
}

function verify(archive: string, artifacts: string): void {
  fs.mkdirSync(artifacts, { recursive: true });
  const bytes = fs.readFileSync(archive);
  fs.writeFileSync(path.join(artifacts, "module.tgz"), bytes);
  const manifestBytes = run("manifest", "tar", ["-xOf", archive, "package/package.json"]);
  assert.equal(manifestBytes.status, 0, "the packed manifest must be readable");
  const manifest = record(JSON.parse(manifestBytes.stdout));
  assert.equal(manifest.name, "@vizejs/nuxt");
  const kit = record(manifest.dependencies)["@nuxt/kit"];
  assert.ok(typeof kit === "string" && /^3\.\d+\.\d+$/.test(kit), "publish an exact v3 Kit");

  // This is deliberately a fresh external projection of the packed Kit edge.
  // Full Nuxt 2/Bridge/3 hosts are checked separately; upstream Nuxt findings
  // cannot be hidden by workspace overrides or mistaken for this module's edge.
  const consumer = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-nuxt-kit-"));
  for (let parent = consumer; path.dirname(parent) !== parent; parent = path.dirname(parent)) {
    assert.equal(
      fs.existsSync(path.join(parent, "pnpm-workspace.yaml")),
      false,
      "consumer must be outside every workspace",
    );
  }
  fs.writeFileSync(
    path.join(consumer, "package.json"),
    JSON.stringify({ private: true, dependencies: { "@nuxt/kit": kit } }, null, 2) + "\n",
  );
  const install = run(
    "install",
    "pnpm",
    ["install", "--lockfile-only", "--ignore-scripts", "--no-frozen-lockfile"],
    consumer,
  );
  assert.equal(install.status, 0, "the external dependency graph must resolve");
  fs.copyFileSync(path.join(consumer, "pnpm-lock.yaml"), path.join(artifacts, "pnpm-lock.yaml"));
  const audit = run("audit", "pnpm", ["audit", "--prod", "--json"], consumer);
  const report: unknown = JSON.parse(audit.stdout);
  const result = {
    scope: "packed @vizejs/nuxt runtime Kit dependency closure; full hosts checked separately",
    archiveSha256: createHash("sha256").update(bytes).digest("hex"),
    packedManifest: manifest,
    consumer,
    consumerManifest: JSON.parse(fs.readFileSync(path.join(consumer, "package.json"), "utf8")),
    auditStatus: audit.status,
    audit: report,
  };
  fs.writeFileSync(path.join(artifacts, "result.json"), JSON.stringify(result, null, 2) + "\n");
  assertPublicKitAudit(report, audit.status);
  console.log(`Public @nuxt/kit@${kit} dependency closure is clean`);

  function run(name: string, command: string, args: string[], cwd?: string) {
    const result = spawnSync(command, args, {
      cwd,
      timeout: 120_000,
      maxBuffer: 16 * 1024 * 1024,
    });
    // Preserve raw producer output and command identity before interpreting it.
    fs.writeFileSync(path.join(artifacts, `${name}.stdout`), result.stdout ?? "");
    fs.writeFileSync(path.join(artifacts, `${name}.stderr`), result.stderr ?? "");
    fs.writeFileSync(
      path.join(artifacts, `${name}.process.json`),
      JSON.stringify(
        {
          command,
          args,
          cwd,
          status: result.status,
          signal: result.signal,
          error: result.error?.message,
        },
        null,
        2,
      ) + "\n",
    );
    assert.ifError(result.error);
    assert.equal(result.signal, null);
    return { ...result, stdout: result.stdout.toString("utf8") };
  }
}

if (import.meta.main) {
  assert.equal(
    process.argv.length,
    4,
    "usage: nuxt-kit-consumer.ts <packed-module.tgz> <artifacts>",
  );
  verify(path.resolve(process.argv[2]), path.resolve(process.argv[3]));
}
