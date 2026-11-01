/** Proposed diagnostic-only Node preload; no Cargo retry, argv/env/output rewrite or native query. */
import childProcess, { type ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import { errorMonitor } from "node:events";
import fs from "node:fs";
import path from "node:path";
import { syncBuiltinESMExports } from "node:module";
import { fileURLToPath } from "node:url";

export function observeMetadataSpawn(spawn: typeof childProcess.spawn, directory: string) {
  let index = 0;
  return new Proxy(spawn, {
    apply(target, receiver, args: unknown[]) {
      const argv =
        args[0] === "cargo" && Array.isArray(args[1]) && args[1][0] === "metadata"
          ? [...args[1]]
          : null;
      const child: ChildProcess = Reflect.apply(target, receiver, args);
      if (!argv) return child;
      const options = typeof args[2] === "object" && args[2] !== null ? args[2] : {};
      const requestedCwd = Reflect.get(options, "cwd");
      const safeOptions = {
        cwd: typeof requestedCwd === "string" ? requestedCwd : process.cwd(),
        stdio: Reflect.get(options, "stdio") === "pipe" ? "pipe" : "other-unclassified",
        environmentOverridden: Reflect.has(options, "env"),
      };
      const ordinal = ++index;
      const stdout: Buffer[] = [],
        stderr: Buffer[] = [];
      let exactBufferEvents = true;
      const errors: { name: string; message: string; code: string | null }[] = [];
      let event = 0;
      const retain = (chunks: Buffer[], value: unknown) => {
        if (Buffer.isBuffer(value)) chunks.push(Buffer.from(value));
        else {
          exactBufferEvents = false;
          chunks.push(Buffer.from(typeof value === "string" ? value : String(value)));
        }
      };
      child.stdout?.on("data", (value: unknown) => retain(stdout, value));
      child.stderr?.on("data", (value: unknown) => retain(stderr, value));
      const save = (
        phase: "error" | "close",
        code: number | null,
        signal: NodeJS.Signals | null,
      ) => {
        // The observer never coerces a signal/null status to a successful exit.
        // I/O failure is visible, and never replaces the actual producer result.
        try {
          const output = path.join(directory, `${process.pid}-${ordinal}`, `${++event}-${phase}`);
          fs.mkdirSync(output, { recursive: true });
          fs.writeFileSync(path.join(output, "stdout.bin"), Buffer.concat(stdout), { flag: "wx" });
          fs.writeFileSync(path.join(output, "stderr.bin"), Buffer.concat(stderr), { flag: "wx" });
          fs.writeFileSync(
            path.join(output, "process.json"),
            JSON.stringify(
              {
                schema: 1,
                scope: "actual metadata child output only; no build/native acceptance",
                phase,
                streamsClosed: phase === "close",
                command: args[0],
                argv,
                parentPid: process.pid,
                parentArgv: process.argv,
                parentExecPath: process.execPath,
                childPid: child.pid ?? null,
                exitStatus: code,
                signal,
                errors,
                exactBufferEvents,
                stdoutStreamPresent: child.stdout !== null,
                stderrStreamPresent: child.stderr !== null,
                callerOptions: safeOptions,
                cargoImageVerified: false,
                retryPerformed: false,
                observer: {
                  module: import.meta.url,
                  sha256: createHash("sha256")
                    .update(fs.readFileSync(fileURLToPath(import.meta.url)))
                    .digest("hex"),
                  node: process.version,
                  githubSha: process.env.GITHUB_SHA ?? null,
                  eventName: process.env.GITHUB_EVENT_NAME ?? null,
                  loadedViaNodeOptions:
                    process.env.NODE_OPTIONS?.includes(fileURLToPath(import.meta.url)) ?? false,
                },
              },
              null,
              2,
            ) + "\n",
            { flag: "wx" },
          );
        } catch (failure) {
          process.stderr.write(`[cargo-metadata-observer] capture failed: ${String(failure)}\n`);
        }
      };
      // Monitor, rather than handle, errors: an absent caller listener still throws.
      child.on(errorMonitor, (value: NodeJS.ErrnoException) => {
        errors.push({ name: value.name, message: value.message, code: value.code ?? null });
        save("error", null, null);
      });
      child.once("close", (code: number | null, signal: NodeJS.Signals | null) =>
        save("close", code, signal),
      );
      return child;
    },
  });
}

// Vite+ preserves NODE_OPTIONS but removes custom variables from task environments.
export function metadataCaptureDirectory(url: string, environment: NodeJS.ProcessEnv) {
  return new URL(url).searchParams.get("capture") || environment.VIZE_CARGO_METADATA_CAPTURE;
}

/** A partial write is retained for upload but must never qualify as a packet. */
export function validateMetadataCapture(directory: string): void {
  const invocations = fs.readdirSync(directory, { withFileTypes: true });
  if (!invocations.length) throw new Error("No Cargo metadata capture packets");
  for (const invocation of invocations) {
    if (!invocation.isDirectory() || !/^[1-9]\d*-\d+$/.test(invocation.name))
      throw new Error("Invalid Cargo metadata invocation directory");
    const root = path.join(directory, invocation.name);
    const packets = fs.readdirSync(root, { withFileTypes: true });
    if (!packets.length) throw new Error("No Cargo metadata event packets");
    for (const packet of packets) {
      if (!packet.isDirectory() || !/^\d+-(?:error|close)$/.test(packet.name))
        throw new Error("Invalid Cargo metadata event directory");
      const files = path.join(root, packet.name);
      for (const name of ["stdout.bin", "stderr.bin", "process.json"])
        if (!fs.lstatSync(path.join(files, name)).isFile())
          throw new Error(`Missing regular Cargo metadata packet file: ${name}`);
      JSON.parse(fs.readFileSync(path.join(files, "process.json"), "utf8"));
    }
  }
}

const output = metadataCaptureDirectory(import.meta.url, process.env);
if (output) {
  childProcess.spawn = observeMetadataSpawn(childProcess.spawn, path.resolve(output));
  syncBuiltinESMExports();
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, directory, ...extra] = process.argv.slice(2);
  if (mode !== "--validate-dir" || !directory || extra.length)
    throw new Error("--validate-dir capture-directory required");
  validateMetadataCapture(directory);
}
