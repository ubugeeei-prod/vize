import type { Page } from "playwright";

/** Narrow isolated-certificate trust seam. Ordinary launches have no added arguments. */
export function hostedCertificateArguments(spki: string | undefined): string[] {
  if (spki === undefined) return [];
  const bytes = Buffer.from(spki, "base64");
  if (
    !/^[A-Za-z0-9+/]{43}=$/.test(spki) ||
    bytes.length !== 32 ||
    bytes.toString("base64") !== spki
  )
    throw new Error("Invalid hosted certificate SPKI");
  return [`--ignore-certificate-errors-spki-list=${spki}`];
}

/** Restrict Documents only. Asset requests retain the gallery's normal behavior. */
export async function guardHostedNavigation(page: Page, galleryUrl: string): Promise<() => void> {
  const gallery = new URL(galleryUrl);
  const prefix = `${gallery.pathname.replace(/\/+$/, "")}/`;
  let refused = false;
  await page.route("**/*", async (route) => {
    const request = route.request();
    if (request.isNavigationRequest() && request.frame() === page.mainFrame()) {
      const url = new URL(request.url());
      if (url.origin !== gallery.origin || !url.pathname.startsWith(prefix)) {
        refused = true;
        await route.abort("blockedbyclient");
        return;
      }
    }
    await route.continue();
  });
  page.on("framenavigated", (frame) => {
    if (frame !== page.mainFrame() || frame.url() === "about:blank") return;
    const url = new URL(frame.url());
    if (url.origin !== gallery.origin || !url.pathname.startsWith(prefix)) refused = true;
  });
  return () => {
    if (refused) throw new Error("Hosted preview navigation left its gallery");
  };
}
