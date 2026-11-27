import type { IncomingMessage } from "node:http";

type RequestHeaders = Pick<IncomingMessage, "headers">;

/**
 * Whether a page of another site sent the request. A browser says so in
 * `Sec-Fetch-Site`, or names an origin that is not the host it asked.
 */
export function isCrossSiteRequest(req: RequestHeaders): boolean {
  if (header(req, "sec-fetch-site") === "cross-site") return true;
  const origin = header(req, "origin");
  return origin !== undefined && !namesRequestHost(req, origin);
}

/**
 * Whether the server's own page, or the user directly, sent the request. A
 * request that does not say where it comes from is not taken as either.
 */
export function isSameOriginRequest(req: RequestHeaders): boolean {
  const site = header(req, "sec-fetch-site");
  if (site !== undefined) return site === "same-origin" || site === "none";
  const origin = header(req, "origin");
  return origin !== undefined && namesRequestHost(req, origin);
}

function namesRequestHost(req: RequestHeaders, origin: string): boolean {
  // HTTP/2 requests carry the authority instead of a Host header.
  const host = header(req, "host") ?? header(req, ":authority");
  try {
    return host !== undefined && new URL(origin).host === host;
  } catch {
    return false;
  }
}

function header(req: RequestHeaders, name: string): string | undefined {
  const value = req.headers[name];
  return Array.isArray(value) ? value[0] : value;
}
