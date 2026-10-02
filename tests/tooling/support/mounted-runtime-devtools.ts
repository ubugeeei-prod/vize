import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { EventEmitter } from "node:events";
import { appendFileSync } from "node:fs";

const hookKey = "__VUE_DEVTOOLS_GLOBAL_HOOK__";

/** Own setup failures as well as the completed trace without masking their error. */
export async function withMountedRuntimeDevtools<T>(
  production: boolean,
  close: () => Promise<unknown>,
  run: (observer: ReturnType<typeof mountedRuntimeDevtools>) => Promise<T>,
  options: Parameters<typeof mountedRuntimeDevtools>[1] = {},
) {
  const observer = mountedRuntimeDevtools(production, options);
  try {
    const result = await run(observer);
    observer.dispose();
    return result;
  } catch (error) {
    // A failed setup still owns its DOM/hook; cleanup cannot replace that failure.
    try {
      await close();
    } catch {}
    try {
      observer.dispose();
    } catch {}
    throw error;
  }
}

/** Observe each fresh development runtime through Vue's existing devtools hook. */
export function mountedRuntimeDevtools(
  production: boolean,
  {
    target = globalThis,
    enabled = process.env.VIZE_VUE_RUNTIME_DEVTOOLS_OBSERVER !== "off",
  }: {
    target?: object;
    enabled?: boolean;
  } = {},
) {
  const original = Object.getOwnPropertyDescriptor(target, hookKey);
  const active =
    !production &&
    enabled &&
    Reflect.get(target, hookKey) === undefined &&
    (original ? original.configurable === true : Object.isExtensible(target));
  const events: Record<string, number> = {};
  const lifecycle = new Map<
    unknown,
    { initialized: number; unmounted: number; version: unknown }
  >();
  const listeners = new EventEmitter();
  const hook = {
    enabled: false,
    appRecords: [] as { id: number; app: unknown; version: unknown; types: unknown }[],
    emit(event: string, ...payload: unknown[]) {
      events[event] = (events[event] ?? 0) + 1;
      if (event === "app:init") {
        const app = payload[0];
        const record = lifecycle.get(app) ?? { initialized: 0, unmounted: 0, version: payload[1] };
        record.initialized++;
        lifecycle.set(app, record);
        hook.appRecords.push({
          id: hook.appRecords.length,
          app,
          version: payload[1],
          types: payload[2],
        });
      } else if (event === "app:unmount") {
        const record = lifecycle.get(payload[0]);
        if (record) record.unmounted++;
        hook.appRecords = hook.appRecords.filter(({ app }) => app !== payload[0]);
      }
      listeners.emit(event, ...payload);
    },
    on: listeners.on.bind(listeners),
    once: listeners.once.bind(listeners),
    off: listeners.off.bind(listeners),
    cleanupBuffer: () => false,
  };
  if (active)
    Object.defineProperty(target, hookKey, { configurable: true, writable: true, value: hook });
  return {
    active,
    hook,
    attach(runtime: { setDevtoolsHook?: (observerHook: typeof hook, target: object) => void }) {
      if (active) {
        assert.equal(
          typeof runtime.setDevtoolsHook,
          "function",
          "pinned Vue must expose its devtools attachment API",
        );
        runtime.setDevtoolsHook!(hook, target);
      }
    },
    complete(app: unknown, snapshots: unknown, diagnostics: unknown[]) {
      const record = lifecycle.get(app);
      if (active) {
        assert.equal(hook.enabled, true, "Vue must attach the fresh devtools observer");
        assert.equal(
          record?.initialized,
          1,
          "devtools must observe this app's initialization once",
        );
        assert.equal(record?.unmounted, 1, "devtools must observe this app's unmount once");
        assert.equal(events["app:init"], 1, "the complete trace must initialize exactly one app");
        assert.equal(events["app:unmount"], 1, "the complete trace must unmount exactly one app");
        assert.equal(hook.appRecords.length, 0, "devtools app records must be released on unmount");
      }
      assert.deepEqual(diagnostics, [], "runtime diagnostics must remain empty");
      if (process.env.VIZE_VUE_RUNTIME_TRACE_EVIDENCE)
        appendFileSync(
          process.env.VIZE_VUE_RUNTIME_TRACE_EVIDENCE,
          `${JSON.stringify({
            pid: process.pid,
            production,
            observer: active,
            events,
            initialized: record?.initialized ?? 0,
            unmounted: record?.unmounted ?? 0,
            runtimeVersion: record?.version ?? null,
            appRecords: hook.appRecords.length,
            diagnostics,
            traceSha256: createHash("sha256").update(JSON.stringify(snapshots)).digest("hex"),
          })}\n`,
        );
    },
    dispose() {
      if (active && Reflect.get(target, hookKey) === hook) {
        if (original) Object.defineProperty(target, hookKey, original);
        else Reflect.deleteProperty(target, hookKey);
      }
      hook.appRecords = [];
      lifecycle.clear();
      listeners.removeAllListeners();
    },
  };
}
