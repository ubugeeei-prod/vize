/** Only render preview URLs on the gallery's origin and documentation links over HTTPS. */
export function safeUrl(
  rawUrl: string,
  destination: "preview" | "help" = "preview",
): string | undefined {
  if (!rawUrl) return undefined;

  try {
    const url = new URL(rawUrl, window.location.href);
    if (
      destination === "preview" &&
      url.origin === window.location.origin &&
      (url.protocol === "http:" || url.protocol === "https:")
    ) {
      return url.href;
    }
    if (destination === "help" && url.protocol === "https:") {
      return url.href;
    }
  } catch {
    // Malformed URLs are not rendered.
  }

  return undefined;
}
