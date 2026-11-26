import { randomBytes, timingSafeEqual } from "node:crypto";
import type { IncomingMessage } from "node:http";

import { HttpError } from "./http-error.js";

export { HttpError } from "./http-error.js";
export {
  assertArtVueSourcePath,
  assertComponentSourcePath,
  assertVueSourcePath,
  decodeUrlComponent,
  isArtVueSourcePath,
  isPathInside,
  isPathInsideAny,
  isTrustedSourcePath,
  isVueSourcePath,
  resolveInside,
  resolveInsideAny,
  resolveTrustedSourcePath,
  resolveUrlPathInside,
} from "./trusted-path.js";

export const DEFAULT_API_BODY_LIMIT_BYTES = 1024 * 1024;

export function createDevSessionToken(): string {
  return randomBytes(32).toString("base64url");
}

export function collectRequestBody(
  req: IncomingMessage,
  limit = DEFAULT_API_BODY_LIMIT_BYTES,
): Promise<string> {
  return new Promise((resolve, reject) => {
    let body = "";
    let size = 0;
    let completed = false;

    req.on("data", (chunk: Buffer | string) => {
      if (completed) return;

      const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
      size += buffer.byteLength;
      if (size > limit) {
        completed = true;
        reject(new HttpError(`Request body exceeds ${limit} bytes`, 413));
        return;
      }

      body += buffer.toString("utf-8");
    });

    req.on("end", () => {
      if (!completed) {
        completed = true;
        resolve(body);
      }
    });

    req.on("error", (error) => {
      if (!completed) {
        completed = true;
        reject(error);
      }
    });
  });
}

export function parseJsonBody<T = unknown>(body: string): T {
  try {
    return JSON.parse(body) as T;
  } catch (error) {
    if (error instanceof SyntaxError) {
      throw new HttpError("Malformed JSON body", 400);
    }
    throw error;
  }
}

export function isLoopbackAddress(address: string): boolean {
  const value = address
    .trim()
    .toLowerCase()
    .replace(/^\[|\]$/g, "");
  if (value === "localhost" || value === "::1") {
    return true;
  }
  if (value.startsWith("::ffff:") || value.startsWith(":ffff:")) {
    return isLoopbackAddress(value.slice(value.indexOf("ffff:") + 5));
  }
  const parts = value.split(".");
  if (parts.length !== 4 || parts[0] !== "127") {
    return false;
  }
  return parts.every((part, index) => {
    if (index === 0) return true;
    const octet = Number(part);
    return Number.isInteger(octet) && octet >= 0 && octet <= 255;
  });
}

export function isLoopbackRequest(req: IncomingMessage): boolean {
  const remote = req.socket?.remoteAddress;
  // Fail closed when the peer address is missing. Falling back to Host would
  // let a client on `vite --host` spoof `Host: localhost` and pass the write
  // API loopback gate.
  if (!remote) {
    return false;
  }
  return isLoopbackAddress(remote);
}

const FORWARDING_HEADERS = ["forwarded", "x-forwarded-for", "x-forwarded-host", "x-real-ip"];
// A host name or bracketed IPv6 literal, with an optional port and nothing else.
const HOST_NAME = /^(\[[^\]\s]+\]|[^\s/:@[\]]+)(?::\d+)?$/;

/**
 * A tunnel or reverse proxy on this machine also connects from loopback, so a
 * local request must be addressed to a loopback name and carry no forwarding
 * headers as well.
 */
export function isLocalRequest(req: IncomingMessage): boolean {
  if (!isLoopbackRequest(req)) return false;
  if (FORWARDING_HEADERS.some((name) => req.headers[name] !== undefined)) return false;
  const name = HOST_NAME.exec(getHost(req) ?? "")?.[1].toLowerCase();
  return (
    name === "localhost" ||
    name === "127.0.0.1" ||
    name === "[::1]" ||
    name?.endsWith(".localhost") === true
  );
}

export function validateDevApiRequest(
  req: IncomingMessage,
  sessionToken: string,
): HttpError | null {
  const originError = validateOrigin(req);
  if (originError) return originError;

  if (!isUnsafeMethod(req.method)) {
    return null;
  }

  // Browsers name the origin of every write; a request without one cannot be matched to the Host.
  if (!getHeader(req, "origin")) {
    return new HttpError("Musea write APIs require an Origin header", 403);
  }

  if (!isLocalRequest(req)) {
    return new HttpError("Musea write APIs are limited to local requests", 403);
  }

  if (!hasValidSessionToken(req, sessionToken)) {
    return new HttpError("Invalid Musea dev session token", 403);
  }

  if (!isJsonRequest(req)) {
    return new HttpError("Content-Type must be application/json", 415);
  }

  return null;
}

export function serializeScriptValue(value: unknown): string {
  return (JSON.stringify(value) ?? "undefined").replace(/[<>&\u2028\u2029]/g, (char) => {
    switch (char) {
      case "<":
        return "\\u003C";
      case ">":
        return "\\u003E";
      case "&":
        return "\\u0026";
      case "\u2028":
        return "\\u2028";
      case "\u2029":
        return "\\u2029";
      default:
        return char;
    }
  });
}

function isUnsafeMethod(method: string | undefined): boolean {
  return method === "POST" || method === "PUT" || method === "PATCH" || method === "DELETE";
}

function isJsonRequest(req: IncomingMessage): boolean {
  const contentType = getHeader(req, "content-type");
  return contentType?.split(";")[0]?.trim().toLowerCase() === "application/json";
}

function validateOrigin(req: IncomingMessage): HttpError | null {
  const secFetchSite = getHeader(req, "sec-fetch-site");
  if (secFetchSite === "cross-site") {
    return new HttpError("Cross-origin Musea API requests are not allowed", 403);
  }

  const origin = getHeader(req, "origin");
  if (!origin) return null;

  const host = getHost(req);
  if (!host) {
    return new HttpError("Missing Host header", 400);
  }

  try {
    const originUrl = new URL(origin);
    if (originUrl.host !== host) {
      return new HttpError("Cross-origin Musea API requests are not allowed", 403);
    }
  } catch {
    return new HttpError("Invalid Origin header", 400);
  }

  return null;
}

function hasValidSessionToken(req: IncomingMessage, expectedToken: string): boolean {
  const actualToken = getHeader(req, "x-musea-session");
  if (!actualToken) return false;

  const actual = Buffer.from(actualToken);
  const expected = Buffer.from(expectedToken);
  return actual.length === expected.length && timingSafeEqual(actual, expected);
}

/** HTTP/2 requests carry the authority instead of a Host header. */
function getHost(req: IncomingMessage): string | undefined {
  return getHeader(req, "host") ?? getHeader(req, ":authority");
}

function getHeader(req: IncomingMessage, name: string): string | undefined {
  const value = req.headers[name];
  if (Array.isArray(value)) return value[0];
  return value;
}
