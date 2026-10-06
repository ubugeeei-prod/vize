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
  const opened = performance.now();
  recorder.session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version: 1, text: source },
  });
  if (coldStart) {
    assertTypedPackets(await recorder.sweep("cold", requests(uri, source)), workspace);
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
