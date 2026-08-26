import { randomBytes, timingSafeEqual } from "node:crypto";
import type { IncomingMessage } from "node:http";

import { HttpError } from "./http-error.js";

export { HttpError } from "./http-error.js";
export {
  decodeUrlComponent,
  isPathInside,
  isPathInsideAny,
  isTrustedSourcePath,
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

export function validateDevApiRequest(
  req: IncomingMessage,
  sessionToken: string,
): HttpError | null {
  const originError = validateOrigin(req);
  if (originError) return originError;

  if (!isUnsafeMethod(req.method)) {
    return null;
  }

  if (!isLoopbackRequest(req)) {
    return new HttpError("Musea write APIs are limited to loopback clients", 403);
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

  const host = getHeader(req, "host");
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

function getHeader(req: IncomingMessage, name: string): string | undefined {
  const value = req.headers[name];
  if (Array.isArray(value)) return value[0];
  return value;
}
