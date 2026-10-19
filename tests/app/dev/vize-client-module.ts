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

/** Vite 8 logs the accepted module, followed by an optional importer boundary. */
export function isVizeClientCompletion(
  message: string,
  initialModuleUrl: string,
  viteBaseUrl: string,
): boolean {
  const prefix = "[vite] hot updated: ";
  if (!message.startsWith(prefix)) return false;
  const loggedPath = message.slice(prefix.length);
  const via = loggedPath.indexOf(" via ");
  const acceptedPath = via < 0 ? loggedPath : loggedPath.slice(0, via);
  if (!acceptedPath.startsWith("/") || acceptedPath.startsWith("//")) return false;
  try {
    const initial = new URL(initialModuleUrl);
    const base = new URL(viteBaseUrl);
    if (base.origin !== initial.origin || !base.pathname.endsWith("/") || base.search || base.hash)
      return false;
    // Match Vite's importUpdatedModule: base + acceptedPathWithoutQuery.slice(1).
    const accepted = new URL(acceptedPath.slice(1), base);
    if (!isVizeClientModuleUrl(initial.href, initial.pathname, initial.origin)) return false;
    initial.searchParams.sort();
    accepted.searchParams.sort();
    return accepted.href === initial.href;
  } catch {
    return false;
  }
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
