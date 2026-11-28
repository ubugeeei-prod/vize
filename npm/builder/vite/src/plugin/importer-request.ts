import { classifyVitePluginRequest } from "@vizejs/native";
import type { VizePluginState } from "./state.ts";

/** Classification depends only on the exact immutable id; each state keeps one. */
export function classifyImporterRequest(
  state: VizePluginState,
  importer: string | undefined,
): ReturnType<typeof classifyVitePluginRequest> | null {
  if (!importer) return null;
  const cached = state.viteImporterRequest;
  if (cached?.id === importer) return cached.request;
  const request = classifyVitePluginRequest(importer);
  state.viteImporterRequest = { id: importer, request };
  return request;
}
