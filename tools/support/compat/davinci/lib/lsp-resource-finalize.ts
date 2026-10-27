import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { isRecord } from "../../../json-value.ts";

interface ProfileResult {
  profile_json?: { path: string; bytes: number; sha256: string };
  finalization_error?: string;
}

interface Session {
  shutdown(): Promise<void>;
  kill(): Promise<void>;
  waitForClosed(): Promise<void>;
  readonly exitedGracefully: boolean;
}

/** Preserve the original workload failure while always restoring the harness. */
export async function finalizeLspResources(options: {
  session: Session;
  profilePath: string | null;
  workloadFailed: boolean;
  result: ProfileResult;
  cleanup(): void;
  persist(): void;
}): Promise<void> {
  let failed = false;
  let failure: unknown;
  const remember = (error: unknown): void => {
    if (!failed) failure = error;
    failed = true;
  };
  try {
    try {
      await options.session.shutdown();
    } catch (error) {
      await options.session.kill();
      // A forced termination cannot establish that the LSP exported a profile.
      if (options.profilePath) throw error;
    }
    if (options.profilePath) {
      await options.session.waitForClosed();
      assert.ok(options.session.exitedGracefully, "the profiled LSP must exit successfully");
      const bytes = fs.readFileSync(options.profilePath);
      const report: unknown = JSON.parse(bytes.toString("utf8"));
      assert.ok(isRecord(report) && report.command === "lsp", "the actual LSP exports its session");
      options.result.profile_json = {
        path: options.profilePath,
        bytes: bytes.byteLength,
        sha256: createHash("sha256").update(bytes).digest("hex"),
      };
    }
  } catch (error) {
    remember(error);
  } finally {
    try {
      options.cleanup();
    } catch (error) {
      remember(error);
    }
    if (failed) options.result.finalization_error = String(failure);
    try {
      options.persist();
    } catch (error) {
      remember(error);
    }
  }
  if (failed && !options.workloadFailed) throw failure;
}
