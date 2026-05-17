import { randomBytes, timingSafeEqual } from "node:crypto";
import fs from "node:fs";
import type { IncomingMessage } from "node:http";
import path from "node:path";

export const DEFAULT_API_BODY_LIMIT_BYTES = 1024 * 1024;

export class HttpError extends Error {
  readonly status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "HttpError";
    this.status = status;
  }
}

export function createDevSessionToken(): string {
  return randomBytes(32).toString("base64url");
}

function realpathNearest(targetPath: string): string {
  let current = path.resolve(targetPath);
  const missingParts: string[] = [];

  while (true) {
    try {
      const real = fs.realpathSync.native(current);
      return missingParts.length > 0 ? path.join(real, ...missingParts.reverse()) : real;
    } catch {
      const parent = path.dirname(current);
      if (parent === current) {
        return path.resolve(targetPath);
      }
      missingParts.push(path.basename(current));
      current = parent;
    }
  }
}

function isResolvedPathInside(parentDir: string, candidatePath: string): boolean {
  const parent = path.resolve(parentDir);
  const candidate = path.resolve(candidatePath);
  const relative = path.relative(parent, candidate);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

export function isPathInside(parentDir: string, candidatePath: string): boolean {
  return isResolvedPathInside(realpathNearest(parentDir), realpathNearest(candidatePath));
}

export function isPathInsideAny(parentDirs: string[], candidatePath: string): boolean {
  const candidate = realpathNearest(candidatePath);
  return parentDirs.some((parentDir) =>
    isResolvedPathInside(realpathNearest(parentDir), candidate),
  );
}

export function resolveInside(parentDir: string, candidatePath: string, label = "path"): string {
  return resolveInsideAny([parentDir], candidatePath, label);
}

export function resolveInsideAny(
  parentDirs: string[],
  candidatePath: string,
  label = "path",
): string {
  if (candidatePath.includes("\0")) {
    throw new HttpError(`${label} contains an invalid character`, 400);
  }

  if (parentDirs.length === 0) {
    throw new HttpError(`No allowed directories configured for ${label}`, 500);
  }

  const parent = path.resolve(parentDirs[0] ?? ".");
  const resolved = path.isAbsolute(candidatePath)
    ? path.resolve(candidatePath)
    : path.resolve(parent, candidatePath);

  if (!isPathInsideAny(parentDirs, resolved)) {
    throw new HttpError(`${label} escapes the allowed directory`, 400);
  }

  return resolved;
}

export function resolveUrlPathInside(
  parentDir: string,
  requestUrl: string,
  label = "path",
): string {
  const rawPath = requestUrl.split(/[?#]/, 1)[0] || "/";
  let pathname = decodeUrlComponent(rawPath, label);

  pathname = pathname.replaceAll("\\", "/");
  if (pathname.split("/").includes("..")) {
    throw new HttpError(`${label} must not contain parent directory segments`, 400);
  }

  const relativePath = `.${pathname}`;
  return resolveInside(parentDir, relativePath, label);
}

export function decodeUrlComponent(value: string, label = "path"): string {
  try {
    return decodeURIComponent(value);
  } catch {
    throw new HttpError(`${label} is not valid URL encoding`, 400);
  }
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

export function validateDevApiRequest(
  req: IncomingMessage,
  sessionToken: string,
): HttpError | null {
  const originError = validateOrigin(req);
  if (originError) return originError;

  if (!isUnsafeMethod(req.method)) {
    return null;
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
