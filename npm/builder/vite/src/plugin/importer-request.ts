import { classifyVitePluginRequest } from "@vizejs/native";
import type { VizePluginState } from "./state.ts";

/** Dev imports reuse one exact immutable id; builds keep their allocation path. */
export function classifyImporterRequest(
  state: VizePluginState,
  importer: string | undefined,
): ReturnType<typeof classifyVitePluginRequest> | null {
  if (!importer) return null;
  if (state.server === null) return classifyVitePluginRequest(importer);
  const cached = state.viteImporterRequest;
  if (cached?.id === importer) return cached.request;
  const request = classifyVitePluginRequest(importer);
  state.viteImporterRequest = { id: importer, request };
  return request;
}
