import { parseSfc } from "@vizejs/native";
import {
  listSfcSrcRequests,
  resolveSfcSrcRequests,
  type HostResolve,
} from "../../../shared/sfc-src.ts";
import type { VizePluginState } from "./state.ts";

/**
 * Resolve one SFC's `src` module requests before the synchronous compile reads
 * them from `state.srcResolutions`. A request left unresolved fails that compile.
 */
export async function prepareSfcSrcRequests(
  state: Pick<VizePluginState, "srcResolutions">,
  resolve: HostResolve,
  file: string,
  source: string,
): Promise<void> {
  const requests = listSfcSrcRequests(parseSfc, file, source);
  if (requests.length === 0) {
    state.srcResolutions?.delete(file);
    return;
  }
  (state.srcResolutions ??= new Map()).set(
    file,
    await resolveSfcSrcRequests(file, requests, resolve),
  );
}
