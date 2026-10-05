import assert from "node:assert/strict";
import fs from "node:fs";
import { performance } from "node:perf_hooks";

import { createSampler } from "../../../tools/support/compat/davinci/lib/lsp-resource-sampler.mjs";
import type { JsonRpcMessage } from "../../tooling/support/lsp/protocol.ts";
import { LspRequestError, LspSession } from "../../tooling/support/lsp/session.ts";
import { SessionRoots, type Packet, type RequestSpec } from "./warm-type-backed-packets.ts";
import { observeWork } from "./warm-type-backed-work.ts";

export class QueryRecorder {
  readonly rows: Array<Record<string, unknown>> = [];
  readonly failures: string[] = [];
  readonly sampler: ReturnType<typeof createSampler>;
  readonly session: LspSession;
  readonly nativeExecutable: string;
  readonly parentExecutable: string;
  readonly roots = new SessionRoots();
  readonly notifications: Array<{ method: string; params: unknown }> = [];
  readonly responses: JsonRpcMessage[] = [];

  constructor(session: LspSession, nativeExecutable: string, parentExecutable: string) {
    this.session = session;
    this.nativeExecutable = nativeExecutable;
    this.parentExecutable = fs.realpathSync(parentExecutable);
    this.sampler = createSampler(session.processId);
    session.notificationObservers.push((method, params) =>
      this.notifications.push({ method, params }),
    );
    session.responseObservers.push((message) => this.responses.push(message));
  }

  async query(
    stage: string,
    spec: RequestSpec,
    following?: (id: number) => JsonRpcMessage[],
  ): Promise<Packet> {
    const parent = { pid: this.session.processId, executable: this.parentExecutable };
    const workBefore = observeWork(this.sampler.sample(), parent);
    const before = this.sampler.sample();
    const started = performance.now();
    let result: unknown = null;
    let requestId: number | undefined;
    let error: Record<string, unknown> | null = null;
    try {
      result = await this.session.request(spec.method, spec.params, 60_000, (id) => {
        requestId = id;
        return following?.(id) ?? [];
      });
    } catch (failure) {
      error =
        failure instanceof LspRequestError
          ? {
              id: failure.id,
              method: failure.method,
              code: failure.code,
              data: failure.data ?? null,
              message: failure.message,
            }
          : { message: failure instanceof Error ? failure.message : String(failure) };
    }
    const wallMs = performance.now() - started;
    const after = this.sampler.sample();
    const workAfter = observeWork(after, parent);
    const packet = { name: spec.name, method: spec.method, result };
    let comparable: unknown;
    let comparableParams: unknown;
    const envelopes = this.responses.filter((message) => message.id === requestId);
    const response = envelopes.length === 1 ? envelopes[0] : undefined;
    if (envelopes.length !== 1) {
      this.failures.push(
        `${stage}/${spec.name}: exactly one complete response envelope is required`,
      );
    }
    let comparableResponse: unknown;
    try {
      comparable = this.roots.visit([packet]);
    } catch (failure) {
      this.failures.push(`${stage}/${spec.name} ownership: ${String(failure)}`);
    }
    try {
      comparableParams = this.roots.visit(spec.params);
    } catch (failure) {
      this.failures.push(`${stage}/${spec.name} request ownership: ${String(failure)}`);
    }
    const processes = after.processes.map((entry: { pid: number }) => {
      try {
        return { ...entry, executable: fs.realpathSync(`/proc/${entry.pid}/exe`) };
      } catch (failure) {
        return {
          ...entry,
          executable: null,
          observationError: failure instanceof Error ? failure.message : String(failure),
        };
      }
    });
    try {
      comparableResponse = this.roots.visit(response);
    } catch (failure) {
      this.failures.push(`${stage}/${spec.name} response ownership: ${String(failure)}`);
    }
    this.rows.push({
      stage,
      ...spec,
      requestId,
      response,
      comparableResponse,
      result,
      error,
      comparable,
      comparableParams,
      wallMs,
      cpuSeconds: after.cpu_seconds - before.cpu_seconds,
      before,
      after,
      kernelWork: {
        scope: "raw per-thread CPU ticks and process IO outside the request wall/CPU window",
        before: workBefore,
        after: workAfter,
      },
      processes,
    });
    if (error && stage !== "cancel")
      this.failures.push(`${stage}/${spec.name}: ${JSON.stringify(error)}`);
    if (stage === "cancel") {
      try {
        assert.deepEqual(error && { method: error.method, code: error.code, data: error.data }, {
          method: spec.method,
          code: -32800,
          data: null,
        });
      } catch (failure) {
        this.failures.push(`${stage}: ${String(failure)}`);
      }
    } else {
      if (
        !processes.some(
          (entry: { executable: string | null }) => entry.executable === this.nativeExecutable,
        )
      ) {
        this.failures.push(`${stage}/${spec.name}: locked native backend is not a live descendant`);
      }
    }
    return packet;
  }

  async sweep(stage: string, specs: RequestSpec[]): Promise<Packet[]> {
    const packets: Packet[] = [];
    for (const spec of specs) packets.push(await this.query(stage, spec));
    return packets;
  }
}
