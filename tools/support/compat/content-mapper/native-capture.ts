import { configuredNodeLauncher } from "../../process-launcher.ts";
/** Observe the physical native child before Vize maps or filters diagnostics. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";

export type ProcessPacket = {
  argv: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
  elapsedMilliseconds: number;
};

export function observe(
  argv: string[],
  cwd: string,
  directory: string,
  env: NodeJS.ProcessEnv,
): ProcessPacket {
  const binary = argv.at(0);
  assert.ok(binary);
  const started = performance.now();
  const result = spawnSync(binary, argv.slice(1), {
    cwd,
    env,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  const packet = {
    argv,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
    elapsedMilliseconds: performance.now() - started,
  };
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(path.join(directory, "process.json"), JSON.stringify(packet, null, 2) + "\n");
  fs.writeFileSync(path.join(directory, "stdout.raw"), packet.stdout);
  fs.writeFileSync(path.join(directory, "stderr.raw"), packet.stderr);
  return packet;
}

export function nativeCapture(
  cli: string[],
  native: string,
  cwd: string,
  directory: string,
  env: NodeJS.ProcessEnv,
) {
  fs.mkdirSync(directory, { recursive: false });
  const wrapper = path.join(directory, "native-observer.mjs");
  fs.writeFileSync(
    wrapper,
    configuredNodeLauncher(new URL("./observe-native-child.js", import.meta.url), {
      directory,
      host: native,
    }),
    { mode: 0o755 },
  );
  const result = observe([...cli, "--corsa-path", wrapper], cwd, path.join(directory, "vize"), env);
  const processes = fs
    .readdirSync(directory)
    .filter((name) => name.endsWith(".argv.json"))
    .sort()
    .map((name) => {
      const prefix = path.join(directory, name.slice(0, -".argv.json".length));
      const config = JSON.parse(fs.readFileSync(prefix + ".argv.json", "utf8")) as {
        argv: string[];
        cwd: string;
        parent: number;
      };
      return {
        prefix,
        ...config,
        ...JSON.parse(fs.readFileSync(prefix + ".exit.json", "utf8")),
        stdout: fs.readFileSync(prefix + ".stdout.raw", "utf8"),
        stderr: fs.readFileSync(prefix + ".stderr.raw", "utf8"),
        stdin: fs.readFileSync(prefix + ".stdin.raw"),
      };
    });
  assert.ok(processes.length, "the observer must capture a real child");
  fs.writeFileSync(
    path.join(directory, "physical-provider.json"),
    JSON.stringify(
      {
        path: native,
        sha256: createHash("sha256").update(fs.readFileSync(native)).digest("hex"),
        wrapperSha256: createHash("sha256").update(fs.readFileSync(wrapper)).digest("hex"),
        scope:
          "Transparent observation of the physical native CLI child; no claim of wrapper/provider authentication for an LSP lifecycle",
      },
      null,
      2,
    ) + "\n",
  );
  return { result, processes };
}
