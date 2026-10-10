import type { IncomingMessage, ServerResponse } from "node:http";
import { timingSafeEqual } from "node:crypto";

export function authorizeSession(
  request: IncomingMessage,
  response: ServerResponse,
  origin: string,
  host: string,
  token: string,
): boolean {
  response.setHeader("Cache-Control", "no-store");
  response.setHeader("Vary", "Origin");
  if (request.headers.origin !== origin || request.headers.host !== host) {
    response.writeHead(403).end();
    return false;
  }
  response.setHeader("Access-Control-Allow-Origin", origin);
  if (request.method === "OPTIONS") {
    const method = request.headers["access-control-request-method"];
    const headers = String(request.headers["access-control-request-headers"] ?? "")
      .toLowerCase()
      .split(",")
      .map((item) => item.trim());
    if (
      !["GET", "POST"].includes(String(method)) ||
      !headers.includes("authorization") ||
      headers.some((item) => !["authorization", "content-type"].includes(item))
    ) {
      response.writeHead(403).end();
      return false;
    }
    response.setHeader("Access-Control-Allow-Methods", "GET, POST");
    response.setHeader("Access-Control-Allow-Headers", "Authorization, Content-Type");
    response.writeHead(204).end();
    return false;
  }
  const actual = Buffer.from(request.headers.authorization ?? "");
  const expected = Buffer.from(`Bearer ${token}`);
  if (actual.length !== expected.length || !timingSafeEqual(actual, expected)) {
    response.writeHead(401).end();
    return false;
  }
  return true;
}

export async function captureInput(
  request: IncomingMessage,
): Promise<{ artPath: string; update: boolean }> {
  if (request.headers["content-type"] !== "application/json") throw new Error("Expected JSON");
  let body = "";
  for await (const chunk of request) {
    body += String(chunk);
    if (Buffer.byteLength(body) > 4096) throw new Error("Capture input is too large");
  }
  const input: unknown = JSON.parse(body);
  if (
    !input ||
    typeof input !== "object" ||
    Array.isArray(input) ||
    Object.keys(input).sort().join(",") !== "artPath,update" ||
    typeof (input as { artPath: unknown }).artPath !== "string" ||
    typeof (input as { update: unknown }).update !== "boolean"
  )
    throw new Error("Capture requires a known artPath and boolean update");
  return input as { artPath: string; update: boolean };
}
