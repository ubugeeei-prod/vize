const PLACEHOLDER_ORIGIN = "https://vize.local";

/**
 * Parse a location for matching, or return undefined when it cannot name a route.
 *
 * A string is an in-app reference: once resolved it has to stay on the placeholder
 * origin, so `//host/path`, `\\host/path` and absolute URLs do not match. A pathname
 * that starts with `//` is refused as well, because written back on its own it
 * would be read as a host.
 */
export function parseLocation(input: string | URL): URL | undefined {
  let url: URL;
  if (input instanceof URL) {
    url = input;
  } else {
    try {
      url = new URL(input, PLACEHOLDER_ORIGIN);
    } catch {
      return undefined;
    }
    if (url.origin !== PLACEHOLDER_ORIGIN) return undefined;
  }
  return url.pathname.startsWith("//") ? undefined : url;
}

/** Path, query and hash of a parsed location: the form a history adapter receives. */
export function locationPath(url: URL): string {
  return `${url.pathname}${url.search}${url.hash}`;
}
