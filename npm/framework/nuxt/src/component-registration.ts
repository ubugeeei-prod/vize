import type { NuxtComponentDescriptor } from "./components.ts";

export function registeredComponentPath(
  component: NuxtComponentDescriptor,
  components: NuxtComponentDescriptor[],
): string {
  const name = component.pascalName ?? component.name;
  const hasClientVariant = components.some(
    (candidate) => candidate.pascalName === component.pascalName && candidate.mode === "client",
  );
  if (component._raw || component.mode !== "server" || !name || hasClientVariant) {
    return component.filePath;
  }

  const separator = component.filePath.includes("?") ? "&" : "?";
  return `${component.filePath}${separator}nuxt_component=server&nuxt_component_name=${name}&nuxt_component_export=${component.export ?? "default"}`;
}
