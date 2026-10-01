import assert from "node:assert/strict";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import path from "node:path";
import { resolveVizeLaunchCommand, type VerifiedLspLaunch } from "./launch.ts";
import { root } from "./paths.ts";
import { LspSessionCapture } from "./session-capture.ts";

const captures = new WeakMap<ChildProcessWithoutNullStreams, LspSessionCapture>();

/** Reuse the existing launch and version probe; capture never starts another server. */
export function spawnLspSessionProcess(
  repoRoot = root,
  sourceRequired = process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD === "1",
  envBinary = process.env.VIZE_LSP_BIN,
): ChildProcessWithoutNullStreams {
  let verified: VerifiedLspLaunch | undefined;
  const [command, ...args] = resolveVizeLaunchCommand(undefined, envBinary, {
    required: sourceRequired,
    repoRoot,
    onVerifiedLaunch: (launch) => {
      verified = launch;
    },
  });
  const capture = sourceRequired
    ? new LspSessionCapture({
        repoRoot,
        outputRoot: path.join(repoRoot, "target/differential/lsp-sessions"),
        launch: verified!,
        callerStack: new Error().stack ?? "",
      })
    : undefined;
  assert.ok(!sourceRequired || capture, "source-built sessions require raw observations");
  const child = spawn(command, args, { cwd: repoRoot, stdio: ["pipe", "pipe", "pipe"] });
  if (capture) {
    captures.set(child, capture);
    child.stdout.on("data", (chunk: Buffer) => capture.append("server", chunk));
    child.stderr.on("data", (chunk: Buffer) => capture.append("stderr", chunk));
    child.on("error", (error) => capture.processError(error));
    child.on("close", (code, signal) => capture.finish(code, signal));
  }
  return child;
}

/** The recorded bytes are exactly the existing client's single write, including batches. */
export function recordLspClientWire(child: ChildProcessWithoutNullStreams, frame: string): void {
  captures.get(child)?.append("client", Buffer.from(frame, "utf8"));
}
