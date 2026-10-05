import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { BUILD_RECIPE, validateBuildReceipt } from "../../../differential/build-receipt.mjs";
import { sha256 } from "../../../differential/harness.mjs";
import { decodeFrames } from "../../../differential/lsp-wire.ts";
import type { VerifiedLspLaunch } from "./launch.ts";
import { linkSessionSourceOrigins, type SourceWitness } from "./session-source-links.ts";

const MAX_CAPTURE_BYTES = 16 * 1024 * 1024;
const STREAMS = ["client", "server", "stderr"] as const;
type Stream = (typeof STREAMS)[number];
let nextSession = 0;

function sourceWitnesses(repoRoot: string, callerStack: string): SourceWitness[] {
  const witnesses: SourceWitness[] = [];
  for (const line of callerStack.split("\n")) {
    const frame = line.match(/(file:\/\/[^\s)]+):(\d+):(\d+)\)?$/);
    if (!frame) continue;
    const absolute = fileURLToPath(frame[1]);
    const relative = path.relative(repoRoot, absolute);
    if (
      !relative.startsWith(`tests${path.sep}tooling${path.sep}`) &&
      !relative.startsWith(`tests${path.sep}snapshots${path.sep}`)
    )
      continue;
    if (relative.endsWith("session-process.ts") || relative.endsWith("session.ts")) continue;
    witnesses.push({
      path: relative.split(path.sep).join("/"),
      line: Number(frame[2]),
      column: Number(frame[3]),
      sha256: sha256(fs.readFileSync(absolute)),
    });
  }
  return witnesses;
}

/** Passive whole-wire observations are pending evidence, never accepted corpus rows. */
export class LspSessionCapture {
  readonly directory: string;
  private readonly launch: VerifiedLspLaunch;
  private readonly witnesses: SourceWitness[];
  private readonly sourceOrigins: ReturnType<typeof linkSessionSourceOrigins>;
  private readonly chunks: Record<Stream, Buffer[]> = { client: [], server: [], stderr: [] };
  private readonly observedBytes: Record<Stream, number> = { client: 0, server: 0, stderr: 0 };
  private capturedBytes = 0;
  private error: string | null = null;
  private closed = false;
  private exitStatus: number | null = null;
  private signal: string | null = null;

  constructor({
    repoRoot,
    outputRoot,
    launch,
    callerStack,
  }: {
    repoRoot: string;
    outputRoot: string;
    launch: VerifiedLspLaunch;
    callerStack: string;
  }) {
    assert.ok(launch, "verified source launch is required for raw observations");
    validateBuildReceipt(launch.receipt, launch.expected, launch.buildRecipe ?? BUILD_RECIPE);
    const probe = launch.versionProbe;
    assert.ok(probe, "actual source version probe is required for raw observations");
    assert.equal(probe.exitStatus, 0);
    assert.equal(probe.signal, null);
    assert.equal(probe.processError, null);
    assert.equal(
      Buffer.from(probe.stdoutBase64, "base64").toString().trim(),
      launch.expected.cliVersion,
    );
    assert.equal(path.resolve(launch.binary), path.resolve(repoRoot, launch.expected.binaryPath));
    this.launch = launch;
    this.witnesses = sourceWitnesses(repoRoot, callerStack);
    this.sourceOrigins = linkSessionSourceOrigins(repoRoot, this.witnesses);
    fs.mkdirSync(outputRoot, { recursive: true });
    this.directory = fs.mkdtempSync(path.join(outputRoot, `${process.pid}-${++nextSession}-`));
    this.writeMetadata(null, null, "awaiting-process-close");
  }

  append(stream: Stream, chunk: Buffer): void {
    this.observedBytes[stream] += chunk.length;
    const remaining = MAX_CAPTURE_BYTES - this.capturedBytes;
    const retained = Math.min(chunk.length, remaining);
    if (retained > 0) {
      this.chunks[stream].push(Buffer.from(chunk.subarray(0, retained)));
      this.capturedBytes += retained;
    }
    if (this.closed) this.writeMetadata(this.exitStatus, this.signal, "process-closed");
  }

  processError(error: Error): void {
    this.error = error.message;
  }

  finish(exitStatus: number | null, signal: string | null): void {
    assert.ok(!this.closed, "raw observation cannot close twice");
    this.closed = true;
    this.exitStatus = exitStatus;
    this.signal = signal;
    this.writeMetadata(exitStatus, signal, "process-closed");
  }

  private writeMetadata(
    exitStatus: number | null,
    signal: string | null,
    state: "awaiting-process-close" | "process-closed",
  ): void {
    const streams = Object.fromEntries(
      STREAMS.map((stream) => {
        const bytes = Buffer.concat(this.chunks[stream]);
        const relative = `${stream}.bin`;
        fs.writeFileSync(path.join(this.directory, relative), bytes);
        let framing:
          | { state: "complete"; messageCount: number }
          | { state: "invalid-or-truncated"; error: string }
          | undefined;
        if (stream !== "stderr") {
          if (bytes.length !== this.observedBytes[stream]) {
            framing = {
              state: "invalid-or-truncated",
              error: "raw stream was truncated by the capture byte limit",
            };
          } else {
            try {
              framing = { state: "complete", messageCount: decodeFrames(bytes).messages.length };
            } catch (error) {
              framing = {
                state: "invalid-or-truncated",
                error: error instanceof Error ? error.message : String(error),
              };
            }
          }
        }
        return [
          stream,
          {
            path: relative,
            capturedBytes: bytes.length,
            observedBytes: this.observedBytes[stream],
            sha256: sha256(bytes),
            truncated: bytes.length !== this.observedBytes[stream],
            ...(framing ? { framing } : {}),
          },
        ];
      }),
    );
    const observation = {
      schema: "vize.lsp.session.observation",
      version: 1,
      state,
      sourceRevision: this.launch.expected.sourceRevision,
      buildReceipt: this.launch.receipt,
      binary: this.launch.expected,
      versionProbe: this.launch.versionProbe,
      argv: ["lsp"],
      clientWireMeaning: "attempted-existing-stdin-write",
      captureByteLimit: MAX_CAPTURE_BYTES,
      sourceWitnesses: this.witnesses,
      sourceOrigins: this.sourceOrigins,
      streams,
      process: { exitStatus, signal, error: this.error },
      acceptance: {
        state: "pending-original-input-and-session-reconciliation",
        wholeFixesClosed: 0,
        nativeHandled: 0,
        nativeEquivalent: 0,
        workspaceDependenciesCaptured: false,
      },
    };
    const destination = path.join(this.directory, "observation.json");
    fs.writeFileSync(`${destination}.tmp`, `${JSON.stringify(observation, null, 2)}\n`);
    fs.renameSync(`${destination}.tmp`, destination);
  }
}
