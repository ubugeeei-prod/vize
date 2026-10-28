import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  cliPacket,
  equalFindings,
  mapperPacket,
  record,
  string,
  write,
  type Expected,
  type Runtime,
} from "./model.ts";
export function environment(runtime: Runtime, prefix: string): NodeJS.ProcessEnv {
  return {
    ...process.env,
    NODE_OPTIONS: "--import=" + new URL("./observer.ts", import.meta.url).href,
    VIZE_TYPE_ADMISSION_OBSERVER: JSON.stringify({
      workspace: runtime.workspace,
      native: runtime.native,
      prefix,
    }),
  };
}
export function verifyCustody(runtime: Runtime, prefix: string): unknown[] {
  const directory = path.dirname(prefix),
    base = path.basename(prefix);
  const paths = fs
    .readdirSync(directory)
    .filter((name) => name.startsWith(base + "-") && name.endsWith(".txt"));
  const events = paths.flatMap((name) =>
    fs
      .readFileSync(path.join(directory, name), "utf8")
      .trim()
      .split("\n")
      .map((text) => record(JSON.parse(text))),
  );
  const loaded = events.filter((event) => event.event === "returned" && event.expected === true);
  assert.ok(loaded.length > 0, "the actual process never loaded the qualified native image");
  for (const event of loaded) {
    assert.equal(string(event.path), runtime.native.path);
    assert.equal(string(event.sha256), runtime.native.sha256);
  }
  assert.ok(!events.some((event) => event.event === "rejected" || event.event === "failed"));
  return events;
}
export function checkProject(
  runtime: Runtime,
  directory: string,
  stage: string,
  expected: Expected[],
  capture: string,
) {
  fs.mkdirSync(capture, { recursive: true });
  const results: { tool: string; stage: string; count: number; custody: unknown[] }[] = [];
  for (const tool of ["cli", "mapper"] as const) {
    const prefix = path.join(capture, tool + "-native");
    const command =
      tool === "cli"
        ? [runtime.node, runtime.cli, "check", "--corsa-path", runtime.backend, "--format", "json"]
        : [
            runtime.backend,
            "--runExternalCode",
            "--project",
            "tsconfig.mapper.json",
            "--pretty",
            "false",
            "--noErrorTruncation",
          ];
    const [executable, ...args] = command;
    assert.ok(executable);
    const result = spawnSync(executable, args, {
      cwd: directory,
      env: environment(runtime, prefix),
      timeout: 180_000,
      maxBuffer: 64 * 1024 * 1024,
    });
    fs.writeFileSync(path.join(capture, tool + ".stdout.txt"), result.stdout ?? Buffer.alloc(0));
    fs.writeFileSync(path.join(capture, tool + ".stderr.txt"), result.stderr ?? Buffer.alloc(0));
    write(path.join(capture, tool + ".process.txt"), {
      command,
      cwd: directory,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stage,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    const packet =
      tool === "cli"
        ? cliPacket(result.stdout, result.status, directory)
        : { diagnostics: mapperPacket(result.stdout, result.stderr, result.status, directory) };
    const diagnostics = packet.diagnostics;
    write(path.join(capture, tool + ".packet.txt"), packet);
    equalFindings(diagnostics, expected, tool + " " + stage);
    const custody = verifyCustody(runtime, prefix);
    results.push({ tool, stage, count: diagnostics.length, custody });
  }
  return results;
}
