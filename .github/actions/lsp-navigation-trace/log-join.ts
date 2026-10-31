import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";

export const RAW_LIMIT = 8 * 1024 * 1024;
export const LINE_LIMIT = 64 * 1024;
const U64 = 18_446_744_073_709_551_615n;
const METHODS = new Set(["references", "prepare_rename", "rename"]);
type Fields = Record<string, string>;
type Scope = { name: string; fields: Fields };
export type RawOwner = { instance: string; pid: number; birthTicks: string; sourceRevision: string;
  imageSha256: string; rawSha256: string; rawBytes: number; eof: boolean; closed: boolean;
  truncated: boolean; errors: readonly string[] };
export type Row = { offset: number; bytes: number; sha256: string; disposition: "request" | "phase" | "unbound" | "other";
  reason: string; requestId?: string; method?: string; phaseId?: string; phase?: string;
  event?: string; contextSpan?: string; elapsedMs?: number; completed?: boolean };
export type Pair = { id: string; phase: string; requestId: string; method: string;
  beginOffset: number; endOffset: number; contextSpan: string; elapsedMs: number; completed: boolean };
export type Request = { id: string; method: string; announcementOffset: number | null;
  complete: boolean; reasons: string[]; phases: Pair[] };
export type JoinResult = { schema: "vize.navigation-trace-join"; version: 1; qualificationCredit: 0;
  owner: RawOwner; captureComplete: boolean; attributionComplete: boolean; errors: string[];
  rows: Row[]; requests: Request[]; unboundRows: number; otherRows: number;
  semantics: string };
function sha(bytes: Uint8Array): string { return createHash("sha256").update(bytes).digest("hex"); }
function u64(value: string | undefined): string {
  if (value === undefined || !/^[1-9][0-9]*$/.test(value) || BigInt(value) > U64) throw new Error("invalid positive u64");
  return value;
}
function fields(text: string): Fields {
  const result: Fields = {}; let cursor = 0;
  while (cursor < text.length) {
    while (/\s/.test(text[cursor] ?? "")) cursor += 1;
    if (cursor === text.length) break;
    const matched = /^([A-Za-z_][A-Za-z0-9_]*)=/.exec(text.slice(cursor));
    if (matched === null || matched[1] === undefined) throw new Error("unsupported field grammar");
    const key = matched[1]; cursor += matched[0].length;
    if (Object.hasOwn(result, key)) throw new Error("duplicate field");
    let value = "";
    if (text[cursor] === '"') {
      const start = cursor; cursor += 1; let escaped = false, closed = false;
      while (cursor < text.length) {
        const char = text[cursor++];
        if (!escaped && char === '"') { closed = true; break; }
        if (!escaped && char === "\\") escaped = true; else escaped = false;
      }
      if (!closed) throw new Error("unterminated quoted field");
      const parsed: unknown = JSON.parse(text.slice(start, cursor));
      if (typeof parsed !== "string") throw new Error("non-string quoted field"); value = parsed;
    } else {
      const matchedValue = /^[^\s]+/.exec(text.slice(cursor));
      if (matchedValue === null) throw new Error("missing field value"); value = matchedValue[0]; cursor += value.length;
    }
    if (cursor < text.length && !/\s/.test(text[cursor] ?? "")) throw new Error("field separator missing");
    result[key] = value;
  }
  return result;
}
function formatted(line: string): { scopes: Scope[]; event: Fields } {
  const prefix = /^\d{4}-\d{2}-\d{2}T[0-9:.]+Z\s+INFO\s+/.exec(line);
  if (prefix === null) throw new Error("unsupported Full formatter prefix");
  let cursor = prefix[0].length; const scopes: Scope[] = [];
  while (!line.startsWith("vize_editor_preparation:", cursor)) {
    const matched = /^([A-Za-z_][A-Za-z0-9_.-]*)/.exec(line.slice(cursor));
    if (matched === null || matched[1] === undefined) throw new Error("unsupported scope grammar");
    const name = matched[1]; cursor += matched[0].length; let values: Fields = {};
    if (line[cursor] === "{") {
      const start = ++cursor; let quoted = false, escaped = false;
      while (cursor < line.length) {
        const char = line[cursor];
        if (!quoted && char === "}") break;
        if (!escaped && char === '"') quoted = !quoted;
        if (!escaped && char === "\\") escaped = true; else escaped = false;
        cursor += 1;
      }
      if (line[cursor] !== "}") throw new Error("unterminated scope");
      values = fields(line.slice(start, cursor)); cursor += 1;
    }
    if (line[cursor] !== ":") throw new Error("scope separator missing"); cursor += 1;
    while (/\s/.test(line[cursor] ?? "")) cursor += 1;
    scopes.push({ name, fields: values });
    if (scopes.length > 32) throw new Error("scope depth cap");
  }
  cursor += "vize_editor_preparation:".length;
  return { scopes, event: fields(line.slice(cursor).trim()) };
}
function parseRow(bytes: Buffer, offset: number, terminal: boolean): Row {
  const row: Row = { offset, bytes: bytes.length, sha256: sha(bytes), disposition: "other", reason: "non-target raw retained" };
  if (!terminal) return { ...row, disposition: "unbound", reason: "unterminated final line" };
  if (bytes.length > LINE_LIMIT) return { ...row, disposition: "unbound", reason: "line byte cap" };
  let line: string;
  try { line = new TextDecoder("utf-8", { fatal: true }).decode(bytes); }
  catch { return { ...row, disposition: "unbound", reason: "invalid UTF8" }; }
  if (!line.includes("vize_editor_preparation")) return row;
  try {
    const parsed = formatted(line.trimEnd()), candidates = parsed.scopes.filter(scope => scope.name === "navigation_request");
    if (candidates.length !== 1) throw new Error(candidates.length === 0 ? "missing request ancestry" : "ambiguous request ancestry");
    const scope = candidates[0]; if (scope === undefined) throw new Error("request scope absent");
    const requestId = u64(scope.fields["request_id"]), method = scope.fields["method"];
    if (method === undefined || !METHODS.has(method)) throw new Error("unknown request method");
    const event = parsed.event, eventKind = event["event"], contextSpan = u64(event["context_span"]);
    if (eventKind === "request") {
      if (u64(event["request_id"]) !== requestId || event["method"] !== method) throw new Error("announcement owner conflict");
      return { ...row, disposition: "request", reason: "inherited owner", requestId, method, contextSpan, event: eventKind };
    }
    if (eventKind !== "begin" && eventKind !== "end") throw new Error("unknown target event");
    const phaseId = u64(event["id"]), phase = event["phase"];
    if (phase === undefined || !/^[A-Za-z0-9_.-]+$/.test(phase)) throw new Error("invalid phase name");
    const result: Row = { ...row, disposition: "phase", reason: "inherited owner", requestId, method, contextSpan, phaseId, phase, event: eventKind };
    if (eventKind === "begin") {
      const items = event["items"]; if (items === undefined || !/^[0-9]+$/.test(items) || BigInt(items) > U64) throw new Error("invalid items");
    } else {
      const elapsed = event["elapsed_ms"], completed = event["completed"];
      if (elapsed === undefined || !/^(?:[0-9]+(?:\.[0-9]+)?)(?:e[+-]?[0-9]+)?$/i.test(elapsed)) throw new Error("invalid elapsed");
      const value = Number(elapsed); if (!Number.isFinite(value) || value < 0) throw new Error("invalid elapsed");
      if (completed !== "true" && completed !== "false") throw new Error("invalid completed flag");
      result.elapsedMs = value; result.completed = completed === "true";
    }
    return result;
  } catch (error: unknown) { return { ...row, disposition: "unbound", reason: error instanceof Error ? error.message : "unsupported target row" }; }
}
/** No adjacency, numeric span-ID, thread, timestamp or FIFO inference. All raw remains in its owned file. */
export function joinRaw(bytes: Buffer, owner: RawOwner): JoinResult {
  const errors: string[] = [], rows: Row[] = [];
  if (bytes.length > RAW_LIMIT) errors.push("raw byte cap");
  if (!/^[a-f0-9]{40}$/.test(owner.sourceRevision) || !/^[a-f0-9]{64}$/.test(owner.imageSha256)
    || !Number.isSafeInteger(owner.pid) || owner.pid < 1 || !/^[1-9][0-9]*$/.test(owner.birthTicks)
    || owner.instance.length === 0 || owner.rawBytes !== bytes.length || owner.rawSha256 !== sha(bytes)) errors.push("raw owner authority incomplete");
  const captureComplete = errors.length === 0 && owner.eof && owner.closed && !owner.truncated && owner.errors.length === 0;
  let cursor = 0;
  while (cursor < Math.min(bytes.length, RAW_LIMIT)) {
    if (rows.length >= 32768) { errors.push("raw line count cap; remaining bytes retained"); break; }
    const end = bytes.indexOf(10, cursor), next = end === -1 ? bytes.length : end + 1;
    rows.push(parseRow(bytes.subarray(cursor, next), cursor, end !== -1)); cursor = next;
  }
  const requests = new Map<string, Request>(), phaseRows = new Map<string, Row[]>();
  function request(row: Row): Request | undefined {
    if (row.requestId === undefined || row.method === undefined) return undefined;
    let found = requests.get(row.requestId);
    if (found === undefined) { found = { id: row.requestId, method: row.method, announcementOffset: null, complete: false, reasons: [], phases: [] }; requests.set(row.requestId, found); }
    if (found.method !== row.method) found.reasons.push("request method conflict"); return found;
  }
  for (const row of rows) {
    const found = request(row); if (found === undefined) continue;
    if (row.disposition === "request") {
      if (found.announcementOffset !== null) found.reasons.push("duplicate request announcement"); else found.announcementOffset = row.offset;
    } else if (row.disposition === "phase" && row.phaseId !== undefined) {
      const group = phaseRows.get(row.phaseId) ?? []; group.push(row); phaseRows.set(row.phaseId, group);
    }
  }
  for (const [id, group] of phaseRows) {
    const begins = group.filter(row => row.event === "begin"), ends = group.filter(row => row.event === "end"), begin = begins[0], end = ends[0];
    const owners = [...new Set(group.flatMap(row => row.requestId === undefined ? [] : [row.requestId]))];
    const issue = begins.length !== 1 || ends.length !== 1 ? "missing/duplicate phase endpoint"
      : begin !== undefined && end !== undefined && begin.offset >= end.offset ? "reversed phase endpoint order"
      : begin?.requestId !== end?.requestId || begin?.method !== end?.method || begin?.phase !== end?.phase || begin?.contextSpan !== end?.contextSpan ? "phase owner/label/context conflict" : null;
    if (issue !== null || begin === undefined || end === undefined) {
      for (const ownerId of owners) requests.get(ownerId)?.reasons.push(issue ?? "phase absent"); continue;
    }
    if (begin.requestId === undefined || begin.method === undefined || begin.phase === undefined || begin.contextSpan === undefined || end.elapsedMs === undefined || end.completed === undefined) throw new Error("validated phase lost required field");
    requests.get(begin.requestId)?.phases.push({ id, phase: begin.phase, requestId: begin.requestId, method: begin.method,
      beginOffset: begin.offset, endOffset: end.offset, contextSpan: begin.contextSpan, elapsedMs: end.elapsedMs, completed: end.completed });
  }
  for (const found of requests.values()) {
    if (found.announcementOffset === null) found.reasons.push("missing request announcement");
    const outer = found.phases.filter(phase => phase.phase === "navigation.request");
    if (outer.length !== 1) found.reasons.push("missing/duplicate outer request phase");
    if (found.phases.some(phase => !phase.completed)) found.reasons.push("unfinished phase retained");
    if (!captureComplete) found.reasons.push("raw capture incomplete");
    found.complete = found.reasons.length === 0;
  }
  const unboundRows = rows.filter(row => row.disposition === "unbound").length;
  return { schema: "vize.navigation-trace-join", version: 1, qualificationCredit: 0, owner, captureComplete,
    attributionComplete: captureComplete && errors.length === 0 && unboundRows === 0 && requests.size > 0 && [...requests.values()].every(request => request.complete),
    errors, rows, requests: [...requests.values()], unboundRows, otherRows: rows.filter(row => row.disposition === "other").length,
    semantics: "Owned-process/request_id ancestry only; context_span is a hint. Inclusive wall times overlap and are not CPU or additive exclusive shares. Late worker rows remain with their original owner. completed=true includes returned errors. No wire-ID mapping or qualification credit." };
}

/** Actual hosted Rust fixture grammar/ancestry proof only: no invented process/ELF owner. */
export function inspectFormatterFixture(bytes: Buffer): object {
  assert(bytes.length > 0 && bytes.length <= 1024 * 1024);
  const rows: Row[] = []; let cursor = 0;
  while (cursor < bytes.length) { const end = bytes.indexOf(10, cursor); assert(end !== -1, "actual fixture must end with newline"); const next = end + 1; rows.push(parseRow(bytes.subarray(cursor, next), cursor, true)); cursor = next; }
  const announcements = rows.filter(row => row.disposition === "request"), phases = rows.filter(row => row.disposition === "phase"), unbound = rows.filter(row => row.disposition === "unbound");
  assert.equal(announcements.length, 8); assert.equal(new Set(announcements.map(row => row.requestId)).size, 8);
  assert.equal(phases.length, 34); assert.equal(unbound.length, 2);
  assert(unbound.every(row => row.reason === "missing request ancestry")); assert.equal(rows.length, 44);
  const groups = new Map<string, Row[]>(); for (const row of phases) { assert(row.phaseId !== undefined); const group = groups.get(row.phaseId) ?? []; group.push(row); groups.set(row.phaseId, group); }
  for (const group of groups.values()) {
    assert.equal(group.length, 2); const begin = group.find(row => row.event === "begin"), end = group.find(row => row.event === "end"); assert(begin !== undefined && end !== undefined);
    assert(begin.offset < end.offset, "actual phase begin must precede its end");
    assert.equal(begin.requestId, end.requestId); assert.equal(begin.method, end.method); assert.equal(begin.phase, end.phase); assert.equal(begin.contextSpan, end.contextSpan);
    assert(announcements.some(row => row.requestId === begin.requestId && row.method === begin.method));
  }
  const endpoint = (phase: string, event: string): Row => { const row = phases.find(row => row.phase === phase && row.event === event); assert(row !== undefined, phase); return row; };
  const nested = phases.filter(row => row.phase === "fixture.nested"); assert.equal(nested.length, 4);
  for (const row of nested) assert.notEqual(row.contextSpan, announcements.find(owner => owner.requestId === row.requestId)?.contextSpan);
  const a = endpoint("fixture.concurrent_a", "begin"), b = endpoint("fixture.concurrent_b", "begin"); assert.notEqual(a.requestId, b.requestId); assert.equal(a.method, b.method);
  assert(a.offset < b.offset && endpoint("fixture.concurrent_b", "end").offset < endpoint("fixture.concurrent_a", "end").offset);
  assert.equal(endpoint("fixture.pending_drop", "end").completed, false); assert.equal(endpoint("fixture.returned_error", "end").completed, true);
  const late = endpoint("fixture.late_worker", "end"), lateOuter = phases.find(row => row.phase === "navigation.request" && row.event === "end" && row.requestId === late.requestId);
  assert(lateOuter !== undefined && lateOuter.completed === false); assert(late.offset > endpoint("fixture.after_cancel", "end").offset);
  assert.equal(endpoint("fixture.migrating", "begin").requestId, endpoint("fixture.migrating", "end").requestId);
  return { schema: "vize.navigation-actual-formatter-fixture", version: 1, bytes: bytes.length, sha256: sha(bytes), requests: 8, ownedPhasePairs: groups.size, expectedBackgroundUnboundRows: unbound.length,
    rawRows: rows, actualFormatterGrammarChecked: true, processOrCliProtocolQualification: false, qualificationCredit: 0 };
}
if (process.argv[1] !== undefined && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv[2], "--formatter-fixture"); const file = process.argv[3]; assert(file !== undefined); assert(fs.statSync(file).size <= 1024 * 1024);
  process.stdout.write(JSON.stringify(inspectFormatterFixture(fs.readFileSync(file)), null, 2) + "\n");
}
