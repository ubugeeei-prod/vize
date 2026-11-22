import fs from "node:fs";
import { type classifyVitePluginRequest, resolveViteVuePath } from "@vizejs/native";

export function resolvePageMacroRequest(
  root: string,
  request: ReturnType<typeof classifyVitePluginRequest>,
  importer: string | undefined,
  environment?: { plugins?: readonly { name: string }[] },
): string | null {
  if (!request.isVueSfcPath || !(request.hasMacroQuery || request.hasDefinePageQuery)) return null;
  const file = resolveViteVuePath(root, request.path, importer);
  if (!file || !fs.existsSync(file)) return null;
  const id = `${file}${request.querySuffix}`;
  // Vue Router extracts definePage from the authored SFC. Running its transform
  // on Vize's already extracted module would replace the record with an empty object.
  if (
    request.hasDefinePageQuery &&
    environment?.plugins?.some(
      ({ name }) => name === "vue-router" || name === "unplugin-vue-router",
    )
  ) {
    return id;
  }
  // Standalone definePage and Nuxt's ?macro=true use Vize's native artifacts.
  return `\0${id}`;
}
