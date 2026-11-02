import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  hash,
  object,
  hoverPacket,
  queryParams,
  markdown,
  type Obj,
  type Target,
} from "./fixtures.ts";
export type Session = {
  processId: number;
  exitedGracefully: boolean;
  stderrText: string;
  stderrObservers: ((text: string) => void)[];
  notificationObservers: ((method: string, params: unknown) => void)[];
  responseObservers: ((message: unknown) => void)[];
  initialize: (root: string | readonly string[], options: Obj) => Promise<unknown>;
  notify: (method: string, params: unknown) => void;
  request: (
    method: string,
    params: unknown,
    timeout?: number,
    following?: (id: number) => unknown[],
  ) => Promise<unknown>;
  waitForNotification: (
    method: string,
    predicate: (params: unknown) => boolean,
    timeout?: number,
  ) => Promise<unknown>;
  waitForClosed: () => Promise<void>;
  shutdown: () => Promise<void>;
};
export const ns = (): string => String(process.hrtime.bigint());
export const errorText = (error: unknown): string =>
  error instanceof Error ? (error.stack ?? error.message) : String(error);
export function traceField(line: string, key: string): string | undefined {
  const match = line.match(new RegExp(`\\b${key}=("(?:[^"\\\\]|\\\\.)*"|[^\\s]+)`));
  if (!match) return undefined;
  return match[1]!.startsWith('"') ? String(JSON.parse(match[1]!)) : match[1];
}
export function phaseEvents(text: string, uri: string, previous: readonly Obj[] = []): Obj[] {
  const rows: Obj[] = [];
  for (const line of text.replace(/\u001b\[[0-9;]*m/g, "").split("\n")) {
    if (line.includes(`preparing initial native project for ${uri} version 1`))
      rows.push({ event: "preparing", line });
    if (line.includes(`prepared initial native project for ${uri} version 1`))
      rows.push({ event: "prepared", line });
    if (line.includes(`starting initial type diagnostics for ${uri} version 1`))
      rows.push({ event: "collectorSelected", line, scope: "before actual prepared/read call" });
    if (line.includes(`finished initial type diagnostics for ${uri} version 1`))
      rows.push({ event: "collectorFinished", line });
    if (
      !line.includes("vize_editor_preparation:") ||
      traceField(line, "phase") !== "initial_api_read"
    )
      continue;
    const event = traceField(line, "event"),
      requestUri = traceField(line, "request_uri");
    if (event === "host_request_binding" && traceField(line, "host_uri") === uri) {
      assert.ok(requestUri?.startsWith("file://"));
      assert.match(traceField(line, "revision") ?? "", /^[1-9]\d*$/);
      rows.push({
        event: "hostRequestBinding",
        line,
        requestUri,
        revision: traceField(line, "revision"),
      });
    }
    const binding = [...previous, ...rows].findLast((row) => row.event === "hostRequestBinding");
    if (
      !binding ||
      binding.requestUri !== requestUri ||
      traceField(line, "route") !== "compiler_kinds"
    )
      continue;
    const identity = {
      line,
      requestUri,
      config: traceField(line, "config"),
      project: traceField(line, "project"),
      snapshot: traceField(line, "snapshot"),
    };
    if (
      ![identity.config, identity.project, identity.snapshot].every(
        (value) => typeof value === "string" && value.length > 0,
      )
    )
      continue;
    if (event === "entered") rows.push({ event: "apiEntered", ...identity });
    if (["complete", "unsupported", "failed"].includes(event ?? ""))
      rows.push({ event: "apiEnded", ...identity });
  }
  return rows;
}
export function classifyInterval(events: Obj[], sentNs: string, responseNs?: string): Obj {
  const entries = events.filter(
    (row) => row.event === "apiEntered" && BigInt(String(row.ns)) <= BigInt(sentNs),
  );
  const entry = entries.at(-1);
  const ended =
    entry &&
    events.find(
      (row) =>
        row.event === "apiEnded" &&
        Number(row.sequence) > Number(entry.sequence) &&
        ["requestUri", "project", "snapshot", "config"].every((key) => row[key] === entry[key]),
    );
  const inside =
    !!entry &&
    !!ended &&
    BigInt(String(entry.ns)) < BigInt(sentNs) &&
    BigInt(sentNs) < BigInt(String(ended.ns));
  return {
    state: inside ? "matchedCallerApiWait" : "UNMATCHED",
    entry: entry ?? null,
    end: ended ?? null,
    requestSentNs: sentNs,
    responseRelation:
      responseNs && ended
        ? BigInt(responseNs) < BigInt(String(ended.ns))
          ? "before"
          : "after"
        : "unknown",
    nativeCpuEntered: "UNKNOWN",
  };
}
export function completionPacket(packet: unknown, uri: string): Obj {
  assert.ok(Array.isArray(packet));
  assert.equal(packet.length, 1);
  const item = object(packet[0]);
  assert.equal(item.label, "invoiceTotal");
  assert.equal(item.kind, 5);
  assert.equal(item.sortText, "11");
  const wrapper = object(object(item.data).vizeCompletion);
  assert.deepEqual(Object.keys(object(item.data)), ["vizeCompletion"]);
  assert.deepEqual(Object.keys(wrapper).sort(), ["item", "requestUri", "revision", "uri"]);
  assert.equal(wrapper.uri, uri);
  assert.ok(Number.isSafeInteger(wrapper.revision) && Number(wrapper.revision) > 0);
  assert.equal(typeof wrapper.requestUri, "string");
  const native = object(wrapper.item);
  assert.equal(native.label, "invoiceTotal");
  assert.equal(native.kind, 5);
  assert.equal(native.sortText, "11");
  object(native.data);
  return {
    packet,
    issued: item,
    revision: wrapper.revision,
    initialProjectionAuthority: "UNAVAILABLE",
    scope:
      "source-qualified native issued item; opaque keys preserved, never disk-derived or wildcard-normalized",
  };
}
export async function firstPair(
  session: Session,
  target: Target,
  repaired: boolean,
  emit: (row: Obj) => void,
): Promise<Obj> {
  const source = repaired ? target.repair : target.invalid;
  const params = queryParams(target.uri, source);
  const sentNs = ns();
  const request = (method: string): Promise<unknown> => {
    const started = process.hrtime.bigint();
    const actualParams =
      method === "textDocument/hover"
        ? { textDocument: params.textDocument, position: params.position }
        : params;
    return session
      .request(method, actualParams, 1000, (id) => {
        emit({ kind: "requestAttempt", id, method, params: actualParams, ns: ns() });
        return [];
      })
      .then((value) => {
        const elapsedMs = Number(process.hrtime.bigint() - started) / 1e6;
        emit({ kind: "individualQueryResult", method, elapsedMs, value, ns: ns() });
        assert.ok(elapsedMs <= 1000, "unchanged 1000ms native query cap");
        return value;
      });
  };
  const values = await Promise.allSettled([
    request("textDocument/hover"),
    request("textDocument/completion"),
  ]);
  const receivedNs = ns();
  emit({
    kind: "queryPairResult",
    sentNs,
    receivedNs,
    repaired,
    values: values.map((value) =>
      value.status === "fulfilled"
        ? value
        : { status: "rejected", reason: errorText(value.reason) },
    ),
  });
  for (const value of values)
    assert.equal(
      value.status,
      "fulfilled",
      value.status === "rejected" ? errorText(value.reason) : "",
    );
  const hover = (values[0] as PromiseFulfilledResult<unknown>).value;
  const completion = (values[1] as PromiseFulfilledResult<unknown>).value;
  assert.deepEqual(hover, hoverPacket(source, repaired));
  const bound = completionPacket(completion, target.uri);
  return {
    sentNs,
    receivedNs,
    elapsedMs: Number(BigInt(receivedNs) - BigInt(sentNs)) / 1e6,
    hover,
    completion: bound,
  };
}
export async function resolveIssued(
  session: Session,
  pair: Obj,
  emit: (row: Obj) => void,
): Promise<Obj> {
  const issued = structuredClone(object(object(pair.completion).issued));
  const started = ns();
  const actual = await session.request("completionItem/resolve", issued, 5000, (id) => {
    emit({ kind: "untimedResolveAttempt", id, issued, ns: ns() });
    return [];
  });
  const expected = {
    ...issued,
    detail: "(property) invoiceTotal: number",
    documentation: { kind: "markdown", value: markdown("number") },
  };
  emit({ kind: "untimedResolveResult", startedNs: started, endedNs: ns(), actual, expected });
  assert.deepEqual(actual, expected);
  return {
    actual,
    expected,
    scope:
      "one declared untimed full issued-item conservation; initial opaque projection remains UNAVAILABLE",
  };
}

export function open(session: Session, target: Target, text: string, version: number): void {
  session.notify("textDocument/didOpen", {
    textDocument: { uri: target.uri, languageId: "vue", version, text },
  });
}
export function change(session: Session, target: Target, text: string, version: number): void {
  session.notify("textDocument/didChange", {
    textDocument: { uri: target.uri, version },
    contentChanges: [{ text }],
  });
}

export function collectCaptures(
  args: { repo: string; output: string; source: string },
  image: string,
): Obj[] {
  const root = path.join(args.repo, "target/differential/lsp-sessions");
  assert.ok(fs.existsSync(root), "no source raw capture directory");
  const destination = path.join(args.output, "raw");
  fs.renameSync(root, destination);
  const rows: Obj[] = [];
  for (const name of fs.readdirSync(destination)) {
    assert.match(name, /^[a-zA-Z0-9-]+$/);
    const dir = path.join(destination, name);
    assert.ok(fs.lstatSync(dir).isDirectory() && !fs.lstatSync(dir).isSymbolicLink());
    const bytes = fs.readFileSync(path.join(dir, "observation.json"));
    const row = object(JSON.parse(bytes.toString()));
    assert.equal(row.state, "process-closed");
    assert.equal(row.sourceRevision, args.source);
    assert.equal(object(row.binary).binarySha256, image);
    assert.equal(object(row.buildReceipt).binarySha256, image);
    assert.equal(object(row.buildReceipt).recipe, "cargo build --release -p vize");
    assert.deepEqual(row.process, { exitStatus: 0, signal: null, error: null });
    const streams: Obj = {};
    for (const stream of ["client", "server", "stderr"]) {
      const value = object(object(row.streams)[stream]);
      assert.equal(value.path, `${stream}.bin`);
      const data = fs.readFileSync(path.join(dir, `${stream}.bin`));
      assert.equal(data.length, value.capturedBytes);
      assert.equal(data.length, value.observedBytes);
      assert.equal(hash(data), value.sha256);
      assert.equal(value.truncated, false);
      if (stream !== "stderr") assert.equal(object(value.framing).state, "complete");
      streams[stream] = {
        bytes: data.length,
        sha256: hash(data),
        ...(stream === "stderr" ? {} : { framing: "complete" }),
      };
    }
    rows.push({
      directory: `raw/${name}`,
      observationSha256: hash(bytes),
      complete: true,
      sourceRevision: args.source,
      binarySha256: image,
      streams,
      process: row.process,
    });
  }
  return rows;
}

export function writeAll(
  fd: number,
  value: string | Buffer,
  write: (fd: number, bytes: Buffer, offset: number, length: number) => number = (
    file,
    bytes,
    offset,
    length,
  ) => fs.writeSync(file, bytes, offset, length),
): void {
  const bytes = typeof value === "string" ? Buffer.from(value, "utf8") : value;
  let offset = 0;
  while (offset < bytes.length) {
    const count = write(fd, bytes, offset, bytes.length - offset);
    assert.ok(
      Number.isSafeInteger(count) && count > 0 && count <= bytes.length - offset,
      "evidence writer must make bounded progress",
    );
    offset += count;
  }
}
