import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import { QueryRecorder } from "./warm-type-backed-measure.ts";
import { assertTypedPackets, requests } from "./warm-type-backed-packets.ts";

/** Observe initial preparation before the existing readiness/idle/warm protocol. */
export async function startup(
  recorder: QueryRecorder,
  workspace: string,
  uri: string,
  source: string,
  coldStart: boolean,
) {
  const start = performance.now();
  const initialization = await recorder.session.initialize(workspace, {
    editor: true,
    typecheck: true,
  });
  const initializeMs = performance.now() - start;
  let began!: () => void;
  const actualDiagnostics = new Promise<void>((resolve) => {
    began = resolve;
  });
  const observe = (text: string) => {
    if (text.includes(`collect_corsa_diagnostics: ${uri}`)) began();
  };
  if (coldStart) recorder.session.stderrObservers.push(observe);
  const opened = performance.now();
  recorder.session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version: 1, text: source },
  });
  if (coldStart) {
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      const specs = requests(uri, source);
      const first = await recorder.query("cold", specs[0]);
      // "starting initial" is before scope acquisition. This producer marker
      // is inside actual native collection; do not substitute queued readiness.
      await Promise.race([
        actualDiagnostics,
        new Promise<void>((_, reject) => {
          timeout = setTimeout(
            () => reject(new Error("actual initial native collection missing")),
            300_000,
          );
        }),
      ]);
      assert.ok(!recorder.session.stderrText.includes("finished initial type diagnostics for "));
      const during = await recorder.query("background", specs[0]);
      assert.notEqual(during.result, null);
      const rest = await recorder.sweep("cold", specs.slice(1));
      assertTypedPackets([first, ...rest], workspace);
      assert.deepEqual(
        during.result,
        first.result,
        "background hover retains complete original information",
      );
    } finally {
      clearTimeout(timeout);
      const index = recorder.session.stderrObservers.indexOf(observe);
      if (index !== -1) recorder.session.stderrObservers.splice(index, 1);
    }
  }
  return {
    initialization,
    startupTimings: {
      initializeMs,
      openedAtMs: opened - start,
      coldCompletedMs: performance.now() - opened,
    },
  };
}
