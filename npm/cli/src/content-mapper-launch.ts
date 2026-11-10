// The TypeScript host waits a fixed 5 s for the content mapper's `initialize`
// reply, and the first `require()` of a freshly installed native addon can
// take most of that (#12530). The reply only declares capabilities, so the
// launcher answers it here and loads the addon afterwards. The native server
// then reads the rest of the same stream, already initialized.
//
// Frames are read from fd 0 with exact lengths and no read-ahead, so every
// later byte is left for the native reader. Until `initialize` succeeds this
// replies as the native server does to an uninitialized stream.
import { readSync, writeSync } from "node:fs";

const MAX_HEADER_BYTES = 8 * 1024;
const MAX_MESSAGE_BYTES = 64 * 1024 * 1024;
const idle = new Int32Array(new SharedArrayBuffer(4));

export class ContentMapperFramingError extends Error {}

function readSome(buffer: Buffer, offset: number, length: number): number {
  for (;;) {
    try {
      return readSync(0, buffer, offset, length, null);
    } catch (error) {
      const code = (error as NodeJS.ErrnoException).code;
      if (code === "EOF") return 0;
      if (code !== "EAGAIN") throw error;
      Atomics.wait(idle, 0, 0, 5);
    }
  }
}

/** One frame body, or `null` at EOF before any header byte. */
function readFrame() {
  const byte = Buffer.alloc(1);
  let header = "";
  let line = "";
  let contentLength: number | undefined;
  for (;;) {
    if (header.length >= MAX_HEADER_BYTES) {
      throw new ContentMapperFramingError("content-mapper headers exceed 8 KiB");
    }
    if (readSome(byte, 0, 1) === 0) {
      if (header.length === 0) return null;
      throw new ContentMapperFramingError("EOF inside content-mapper headers");
    }
    const char = byte.toString("latin1");
    header += char;
    line += char;
    if (char !== "\n") continue;
    if (line === "\r\n" || line === "\n") break;
    const separator = line.indexOf(":");
    if (separator >= 0 && line.slice(0, separator).toLowerCase() === "content-length") {
      if (contentLength !== undefined) {
        throw new ContentMapperFramingError("duplicate Content-Length header");
      }
      const value = line.slice(separator + 1).trim();
      if (!/^\d+$/.test(value)) throw new ContentMapperFramingError("invalid Content-Length");
      contentLength = Number(value);
    }
    line = "";
  }
  if (contentLength === undefined) {
    throw new ContentMapperFramingError("missing Content-Length header");
  }
  if (contentLength > MAX_MESSAGE_BYTES) {
    throw new ContentMapperFramingError("content-mapper message exceeds 64 MiB");
  }
  const body = Buffer.alloc(contentLength);
  for (let offset = 0; offset < contentLength;) {
    const read = readSome(body, offset, contentLength - offset);
    if (read === 0) throw new ContentMapperFramingError("EOF inside a content-mapper message");
    offset += read;
  }
  return body;
}

function writeFrame(value: unknown): void {
  const body = Buffer.from(JSON.stringify(value));
  const frame = Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
  for (let offset = 0; offset < frame.length;) {
    offset += writeSync(1, frame, offset);
  }
}

// Key order follows the native server's sorted JSON objects.
const fail = (id: unknown, code: number, message: string) =>
  writeFrame({ error: { code, message }, id, jsonrpc: "2.0" });

function initializeProblem(params: unknown): string | null {
  if (typeof params !== "object" || params === null || Array.isArray(params)) {
    return "Invalid initialize params: expected an object";
  }
  const { locale, positionEncodings } = params as Record<string, unknown>;
  if (locale !== undefined && locale !== null && typeof locale !== "string") {
    return "Invalid initialize params: `locale` must be a string";
  }
  if (positionEncodings === undefined) {
    return "Invalid initialize params: missing field `positionEncodings`";
  }
  if (!Array.isArray(positionEncodings) || positionEncodings.some((e) => typeof e !== "string")) {
    return "Invalid initialize params: `positionEncodings` must be a string array";
  }
  if (!positionEncodings.includes("utf-8")) return "Vize requires the utf-8 position encoding";
  return null;
}

/**
 * Answer the stream until `initialize` succeeds. `true` means the native
 * server takes over initialized; `false` means the host closed the stream
 * first and there is nothing to serve.
 */
export function answerContentMapperInitialize(): boolean {
  for (;;) {
    const body = readFrame();
    if (body === null) return false;
    let request: Record<string, unknown>;
    try {
      const parsed: unknown = JSON.parse(body.toString("utf8"));
      if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
        throw new Error("expected a request object");
      }
      request = parsed as Record<string, unknown>;
      if (typeof request.method !== "string") throw new Error("missing field `method`");
    } catch (error) {
      fail(null, -32700, `Parse error: ${(error as Error).message}`);
      continue;
    }
    const id = request.id ?? null;
    if (request.method === "initialize") {
      const problem = initializeProblem(request.params ?? null);
      if (problem === null) {
        writeFrame({
          id,
          jsonrpc: "2.0",
          result: { diagnosticSource: "vize", positionEncoding: "utf-8" },
        });
        return true;
      }
      fail(id, -32602, problem);
    } else if (["openProject", "closeProject", "transform"].includes(request.method as string)) {
      fail(id, -32002, "Content mapper is not initialized");
    } else {
      fail(id, -32601, `Method not found: ${request.method as string}`);
    }
  }
}
