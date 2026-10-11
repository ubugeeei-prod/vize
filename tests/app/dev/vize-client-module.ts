export const NUXT_UI_HMR_MODULE_PATH = "/components/Matrix.vue";

export function logVizeClientModules(
  name: string,
  responses: ReadonlyArray<{ url(): string; status(): number }>,
): void {
  console.log(
    `${name} authored client modules:`,
    JSON.stringify(
      responses.map((response) => ({
        url: response.url(),
        status: response.status(),
      })),
    ),
  );
}

/** Match the public client identity emitted by toPluginVisibleVirtualId. */
export function isVizeClientModuleUrl(
  urlValue: string,
  moduleSuffix: string,
  origin: string,
  hot = false,
): boolean {
  let url: URL;
  try {
    url = new URL(urlValue);
  } catch {
    return false;
  }
  return (
    url.origin === origin &&
    url.pathname.endsWith(moduleSuffix) &&
    url.searchParams.has("vue") &&
    url.searchParams.has("vize") &&
    !url.searchParams.has("vize-ssr") &&
    (!hot || url.searchParams.has("t"))
  );
}
