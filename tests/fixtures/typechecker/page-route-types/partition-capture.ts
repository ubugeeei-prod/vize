import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type { TestContext } from "node:test";

import { repoRoot } from "../../../_helpers/realworld-patch.ts";
import type { Observation, Topology } from "./partition-assertions.ts";

const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const json = (value: unknown) => `${JSON.stringify(value, null, 2)}\n`;
function bytes(file: string) {
  const raw = fs.readFileSync(file);
  return { bytes: raw.length, sha256: digest(raw), base64: raw.toString("base64") };
}
function identity(file: string) {
  const raw = fs.readFileSync(file);
  return { path: file, physical: fs.realpathSync(file), bytes: raw.length, sha256: digest(raw) };
}
function processError(error: Error | undefined): Record<string, unknown> | null {
  if (!error) return null;
  return {
    name: error.name,
    ...Object.fromEntries(
      Object.getOwnPropertyNames(error).map((name) => [name, Reflect.get(error, name) as unknown]),
    ),
  };
}

/** Save failed and successful raw observations before decoding or asserting. */
export function secondaryCapture(cli: string, corsa: string, workspace: string) {
  const capture = path.join(
    repoRoot,
    "target/differential/router-page-types-secondary",
    randomUUID(),
  );
  fs.mkdirSync(capture, { recursive: true });
  fs.copyFileSync(
    path.join(repoRoot, "target/ci/vize.differential-build.json"),
    path.join(capture, "vize.differential-build.json"),
  );
  fs.writeFileSync(
    path.join(capture, "authority.json"),
    json({
      authority: "secondary-synthetic-no-provider-credit",
      cli: identity(cli),
      corsa: identity(corsa),
      workspace,
    }),
  );
  let sequence = 0;
  return {
    run(
      t: TestContext,
      label: { label: string; topology: Topology; requestedServers: number },
      patterns: string[],
      files: string[],
    ): Observation {
      const args = [
        "check",
        ...patterns,
        "--servers",
        String(label.requestedServers),
        "--show-virtual-ts",
        "--tsconfig",
        "tsconfig.json",
        "--format",
        "json",
        "--quiet",
        "--corsa-path",
        corsa,
      ];
      const inputs = Object.fromEntries(
        files.map((file) => [file, bytes(path.join(workspace, file))]),
      );
      const result = spawnSync(cli, args, {
        cwd: workspace,
        env: { ...process.env, LANG: "C", LC_ALL: "C" },
        maxBuffer: 64 * 1024 * 1024,
        timeout: 120_000,
      });
      const raw = {
        ...label,
        command: cli,
        args,
        cwd: workspace,
        inputs,
        status: result.status,
        signal: result.signal,
        error: processError(result.error),
        stdoutBase64: result.stdout?.toString("base64") ?? null,
        stderrBase64: result.stderr?.toString("base64") ?? null,
      };
      const receipt = path.join(capture, `${String(++sequence).padStart(3, "0")}.json`);
      fs.writeFileSync(receipt, json(raw));
      t.diagnostic(
        JSON.stringify({ authority: "secondary-synthetic-no-provider-credit", receipt, ...label }),
      );
      assert.deepEqual(
        Object.fromEntries(files.map((file) => [file, bytes(path.join(workspace, file))])),
        inputs,
        "authored input and configuration bytes remain unchanged during checking",
      );
      if (result.error) throw result.error;
      assert.ok(Buffer.isBuffer(result.stdout) && Buffer.isBuffer(result.stderr));
      const stdout = result.stdout.toString("utf8");
      const stderr = result.stderr.toString("utf8");
      const report = JSON.parse(stdout) as Observation["report"];
      return { status: result.status, stdout, stderr, report };
    },
  };
}
