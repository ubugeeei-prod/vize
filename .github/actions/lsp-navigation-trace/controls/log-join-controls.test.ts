import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { ChildProcess } from "node:child_process";
import { PassThrough } from "node:stream";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { adapterIdentity, shippingBenchmark, shippingBinary, shippingCargoArtifact, ORIGINAL_BENCHMARK, INVOKED_BENCHMARK, SHIPPING_RECIPE } from "../shipping-adapter.ts";
import { test } from "node:test";
import { joinRaw, inspectFormatterFixture, HOVER_PHASES, LINE_LIMIT, RAW_LIMIT, type RawOwner } from "../log-join.ts";
import { instrumentWirePrototype, RawCapture, originalArguments, expectedNavigationCounts, validatePreloadTerminals, writeAll } from "../controller.ts";

// These grammar/reuse/corruption packets are explicitly synthetic, not actual formatter output.
const digest = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
function get(value: unknown, key: string): unknown { assert(value !== null && typeof value === "object"); return Reflect.get(value, key); }
function hasErrors(value: unknown): boolean { const errors = get(get(value, "owner"), "errors"); assert(Array.isArray(errors)); return errors.length > 0; }
function line(owner: string, fields: string, child = "", method = "rename"): string {
  return `2026-10-31T08:00:00.000000Z  INFO navigation_request{request_id=${owner} method="${method}"}:${child} vize_editor_preparation: ${fields}\n`;
}
const announce = (owner: string, context = "1", method = "rename"): string => line(owner, `request_id=${owner} method="${method}" context_span=${context} event="request"`, "", method);
const begin = (owner: string, id: string, phase = "navigation.request", context = "1", child = "", method = "rename"): string => line(owner, `phase="${phase}" id=${id} items=1 context_span=${context} event="begin"`, child, method);
const end = (owner: string, id: string, phase = "navigation.request", context = "1", completed = true, child = "", method = "rename"): string => line(owner, `phase="${phase}" id=${id} context_span=${context} elapsed_ms=1.25 completed=${completed} event="end"`, child, method);
function owner(bytes: Buffer, changes: Partial<RawOwner> = {}): RawOwner {
  return { instance: "run:1:session:pid:birth", pid: 42, birthTicks: "99", sourceRevision: "a".repeat(40), imageSha256: "b".repeat(64),
    rawSha256: digest(bytes), rawBytes: bytes.length, eof: true, closed: true, truncated: false, errors: [], ...changes };
}
function join(text: string, changes: Partial<RawOwner> = {}): ReturnType<typeof joinRaw> { const bytes = Buffer.from(text); return joinRaw(bytes, owner(bytes, changes)); }
const complete = (id = "1", phase = "1", context = "1"): string => announce(id, context) + begin(id, phase, "navigation.request", context) + end(id, phase, "navigation.request", context);

test("complete request and worker pairs use owner ancestry, not span equality", () => {
  const child = 'child{request_id=999 method="other"}:';
  const result = join(announce("1") + begin("1", "1") + begin("1", "2", "native.rename", "55", child)
    + end("1", "2", "native.rename", "55", true, child) + end("1", "1"));
  assert.equal(result.attributionComplete, true); assert.equal(result.requests[0]?.phases.length, 2);
  assert.equal(result.requests[0]?.phases.find(p => p.id === "2")?.contextSpan, "55");
});
test("same-method concurrent interleaving and reverse completion do not use FIFO", () => {
  const result = join(announce("2") + begin("2", "20") + announce("1") + begin("1", "10") + end("1", "10") + end("2", "20"));
  assert.equal(result.attributionComplete, true); assert.deepEqual(result.requests.map(r => r.id), ["2", "1"]);
  assert.equal(result.requests.find(r => r.id === "1")?.phases[0]?.id, "10");
});
test("repeated phase names remain separate phase counter identities", () => {
  const result = join(announce("1") + begin("1", "1") + begin("1", "2", "native.rename") + end("1", "2", "native.rename")
    + begin("1", "3", "native.rename") + end("1", "3", "native.rename") + end("1", "1"));
  assert.equal(result.attributionComplete, true); assert.equal(result.requests[0]?.phases.length, 3);
});
test("synthetic reused context IDs cannot merge requests or process incarnations", () => {
  const result = join(complete("1", "1", "7") + complete("2", "2", "7"));
  assert.equal(result.requests.length, 2); assert.equal(result.attributionComplete, true);
  const bytes = Buffer.from(complete()); const one = joinRaw(bytes, owner(bytes)), two = joinRaw(bytes, owner(bytes, { instance: "new-process", birthTicks: "100" }));
  assert.notEqual(one.owner.instance, two.owner.instance); assert.equal(two.requests[0]?.id, "1");
});
test("cancelled outer phase does not steal late worker rows into the next request", () => {
  const result = join(announce("1") + begin("1", "1") + begin("1", "2", "native.rename") + end("1", "1", "navigation.request", "1", false)
    + complete("2", "3") + end("1", "2", "native.rename"));
  assert.equal(result.requests.find(r => r.id === "1")?.complete, false);
  assert.equal(result.requests.find(r => r.id === "1")?.phases.find(p => p.phase === "native.rename")?.completed, true);
  assert.equal(result.requests.find(r => r.id === "2")?.complete, true);
});
test("returned error completion flag does not imply protocol success", () => {
  const result = join(complete()); assert.equal(result.requests[0]?.phases[0]?.completed, true);
  assert.match(result.semantics, /returned errors/);
});
test("missing announcement or phase endpoint remains incomplete", () => {
  for (const text of [begin("1", "1") + end("1", "1"), announce("1") + begin("1", "1"), announce("1") + end("1", "1")]) {
    const result = join(text); assert.equal(result.attributionComplete, false); assert.equal(result.requests[0]?.complete, false);
  }
});
test("duplicate announcements, mirrored rows and endpoints are retained/refused", () => {
  for (const text of [announce("1") + complete(), complete() + end("1", "1"), complete() + complete()]) {
    const result = join(text); assert.equal(result.attributionComplete, false); assert(result.requests[0]?.reasons.length);
  }
});
test("reversed phase endpoints remain raw and cannot form an admitted pair", () => {
  const result = join(announce("1") + end("1", "1") + begin("1", "1"));
  assert.equal(result.rows.length, 3); assert.equal(result.requests[0]?.phases.length, 0);
  assert.equal(result.requests[0]?.complete, false); assert.equal(result.attributionComplete, false);
  assert(result.requests[0]?.reasons.includes("reversed phase endpoint order"));
});
test("conflicting phase ownership, labels, context and methods refuse pairs", () => {
  for (const text of [announce("1") + announce("2") + begin("1", "1") + end("2", "1"),
    announce("1") + begin("1", "1") + end("1", "1", "different"), announce("1") + begin("1", "1") + end("1", "1", "navigation.request", "2"),
    announce("1") + begin("1", "1") + end("1", "1", "navigation.request", "1", true, "", "references")]) {
    const result = join(text); assert.equal(result.attributionComplete, false);
  }
});
test("duplicate navigation ancestry is ambiguous while unrelated child fields are not", () => {
  const duplicate = 'navigation_request{request_id=2 method="rename"}:';
  const result = join(announce("1") + begin("1", "1", "navigation.request", "1", duplicate));
  assert.equal(result.unboundRows, 1); assert.match(result.rows[1]?.reason ?? "", /ambiguous/);
});
test("missing, filtered and payload-imitation ancestry cannot bind from adjacency", () => {
  const lines = [
    '2026-10-31T08:00:00.0Z  INFO vize_editor_preparation: phase="navigation.request" id=1 items=1 context_span=1 event="begin"\n',
    '2026-10-31T08:00:00.0Z  INFO vize_editor_preparation: message="navigation_request{request_id=1 method=rename}" event="begin"\n',
    '{"target":"vize_editor_preparation","request_id":1}\n',
  ];
  for (const text of lines) { const result = join(announce("1") + text); assert.equal(result.unboundRows, 1); assert.equal(result.attributionComplete, false); }
});
test("large u64 IDs stay lossless and invalid zero/overflow/rounded forms refuse", () => {
  assert.equal(join(complete("9007199254740993", "18446744073709551615")).requests[0]?.id, "9007199254740993");
  for (const value of ["0", "01", "18446744073709551616", "1e3", "-1"])
    assert.equal(join(complete(value)).unboundRows, 3);
});
test("malformed fields, unknown method and incomplete/overlong/invalidUTF8 lines remain raw", () => {
  for (const text of [announce("1").replace('method="rename"', 'method="rename" request_id=2'), complete().replaceAll('method="rename"', 'method="other"'),
    complete().slice(0, -1), "x".repeat(LINE_LIMIT + 1) + "\n"]) assert(join(text).unboundRows > 0);
  const bytes = Buffer.from([0xff, 10]); assert.equal(joinRaw(bytes, owner(bytes)).rows[0]?.reason, "invalid UTF8");
});
test("capture cap, missing EOF/close, writer error and owner/hash changes never admit distributions", () => {
  for (const changes of [{ eof: false }, { closed: false }, { truncated: true }, { errors: ["write failed"] }, { rawSha256: "0".repeat(64) }, { birthTicks: "0" }])
    assert.equal(join(complete(), changes).captureComplete, false);
  const bytes = Buffer.alloc(RAW_LIMIT + 1, 65); assert.equal(joinRaw(bytes, owner(bytes)).captureComplete, false);
});
test("background/non-target rows are retained and never converted to zero-time requests", () => {
  const result = join("ordinary stderr\n" + '2026-10-31T08:00:00Z  INFO vize_editor_preparation: phase="background" id=9 items=1 event="begin"\n');
  assert.equal(result.otherRows, 1); assert.equal(result.unboundRows, 1); assert.equal(result.requests.length, 0);
});

test("initialize wrapper preserves exact Promise/this/args/dispatch and following calls", async () => {
  const originalArgs = [{ value: 1 }, false], promise = Promise.resolve({ value: 2 }); let seen: unknown[] = [], receiver: unknown; let factories = 0;
  class Wire { launch = { name: "vize" }; initialize(...args: unknown[]): Promise<{ value: number }> { seen = args; receiver = this; return promise; } }
  const wire = new Wire(), restore = instrumentWirePrototype(Wire.prototype, () => { factories += 1; return { seal() {}, dispose() {} }; }, () => {});
  assert.equal(wire.initialize(...originalArgs), promise); assert.equal(receiver, wire); assert.equal(seen[0], originalArgs[0]); assert.equal(seen[1], false); assert.equal(factories, 1);
  restore(); assert.equal(wire.initialize(), promise); assert.equal(factories, 1); await promise;
});
test("factory/refusal exceptions cannot change sync error or rejected Promise", async () => {
  const sync = new Error("original sync"), reject = Promise.reject(new Error("original rejection")); void reject.catch(() => {});
  class Wire { launch = { name: "vize" }; initialize(mode: string): Promise<never> { if (mode === "throw") throw sync; return reject; } }
  const restore = instrumentWirePrototype(Wire.prototype, () => { throw new Error("observer failed"); }, () => { throw new Error("sink failed"); });
  const wire = new Wire(); assert.throws(() => wire.initialize("throw"), error => error === sync); assert.equal(wire.initialize("reject"), reject); await assert.rejects(reject, /original rejection/); restore();
});
test("non-Vize bypass and replacement restore preserve later caller wrappers", () => {
  class Wire { launch = { name: "vue-language-server" }; initialize(): number { return 7; } }
  const restore = instrumentWirePrototype(Wire.prototype, () => { throw new Error("unexpected factory"); }, () => { throw new Error("unexpected refusal"); });
  assert.equal(new Wire().initialize(), 7); Wire.prototype.initialize = () => 8; restore(); assert.equal(new Wire().initialize(), 8);
});
function capture(tail = ""): { root: string; file: string; stream: PassThrough; child: ChildProcess; capture: RawCapture } {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "navigation-raw-controls-")), file = path.join(root, "session-1"), stream = new PassThrough(), child = new ChildProcess();
  const { rawSha256: _hash, rawBytes: _bytes, eof: _eof, closed: _closed, truncated: _truncated, errors: _errors, ...identity } = owner(Buffer.alloc(0));
  return { root, file, stream, child, capture: new RawCapture(file, identity, stream, child, { synthetic: true }, tail) };
}
test("raw tee preserves chunk bytes/UTF8 and existing listeners, closes only own listeners", () => {
  const c = capture(), seen: Buffer[] = []; const existing = (b: Buffer): void => { seen.push(b); }; c.stream.on("data", existing);
  try { const bytes = Buffer.from("雪🌸\n" + complete()); for (let i = 0; i < bytes.length; i += 1) c.stream.emit("data", bytes.subarray(i, i + 1));
    c.stream.emit("end"); c.child.emit("close", 0, null); assert.deepEqual(Buffer.concat(seen), bytes); assert.deepEqual(fs.readFileSync(c.file + ".raw"), bytes);
    assert.equal(c.stream.listenerCount("data"), 1); assert.equal(c.stream.listeners("data")[0], existing); const m: unknown = JSON.parse(fs.readFileSync(c.file + ".json", "utf8"));
    assert.equal(get(get(m, "owner"), "rawSha256"), digest(bytes));
  } finally { c.capture.dispose(); fs.rmSync(c.root, { recursive: true }); }
});
test("raw cap retains exact prefix and never pauses/kills or stops original listeners", () => {
  const c = capture(); let total = 0; c.stream.on("data", (b: Buffer) => { total += b.length; });
  try { const bytes = Buffer.alloc(RAW_LIMIT + 16, 65); c.stream.emit("data", bytes); c.stream.emit("end"); c.child.emit("close", null, "SIGKILL");
    assert.equal(total, bytes.length); assert.equal(fs.statSync(c.file + ".raw").size, RAW_LIMIT); const m: unknown = JSON.parse(fs.readFileSync(c.file + ".json", "utf8"));
    assert.equal(get(get(m, "owner"), "truncated"), true); assert.equal(c.child.killed, false);
  } finally { c.capture.dispose(); fs.rmSync(c.root, { recursive: true }); }
});
test("nonempty early tail retains later raw but refuses prefix completeness", () => {
  const c = capture("already emitted"); try { c.stream.emit("data", Buffer.from("later\n")); c.stream.emit("end"); c.child.emit("close", 0, null);
    assert.equal(fs.readFileSync(c.file + ".raw", "utf8"), "later\n"); const m: unknown = JSON.parse(fs.readFileSync(c.file + ".json", "utf8")); assert(hasErrors(m));
  } finally { c.capture.dispose(); fs.rmSync(c.root, { recursive: true }); }
});
test("wx collision preserves prior evidence and premature seal remains incomplete/idempotent", () => {
  const c = capture(); try { assert.throws(() => new RawCapture(c.file, owner(Buffer.alloc(0)), new PassThrough(), new ChildProcess(), {}, ""), /EEXIST/);
    c.capture.seal(); const first = fs.readFileSync(c.file + ".json"); c.capture.seal(); assert.deepEqual(fs.readFileSync(c.file + ".json"), first);
    const m: unknown = JSON.parse(first.toString("utf8")); assert.equal(get(get(m, "owner"), "closed"), false); assert.equal(get(get(m, "owner"), "eof"), false);
  } finally { c.capture.dispose(); fs.rmSync(c.root, { recursive: true }); }
});
test("writer failure is contained, later existing listeners still see bytes", () => {
  const c = capture(); let observed = 0; c.stream.on("data", () => { observed += 1; });
  try { const fd: unknown = Reflect.get(c.capture, "rawFd"); assert(typeof fd === "number"); fs.closeSync(fd); assert.doesNotThrow(() => c.stream.emit("data", Buffer.from("x"))); c.capture.seal();
    assert.equal(observed, 1); const m: unknown = JSON.parse(fs.readFileSync(c.file + ".json", "utf8")); assert(hasErrors(m));
  } finally { c.capture.dispose(); fs.rmSync(c.root, { recursive: true }); }
});
test("original finite command and actual method-count receipts remain exact", () => {
  assert.deepEqual(originalArguments("/owned/report"), ["exec", "node", INVOKED_BENCHMARK, "--fresh", "1", "--warm", "5", "--out", "/owned/report"]);
  const receipt = (n: number): object => { const r = { queryPackets: n, methodCounts: { "request textDocument/hover": n, "request textDocument/references": n, "request textDocument/prepareRename": n, "request textDocument/rename": n } }; return { products: { vize: r, "vue-language-tools": r } }; };
  const report = { schemaVersion: 3, status: "passed", configuration: { freshSamplesPerProduct: 1, warmSamplesPerProduct: 5 }, wire: { primaryOperationReceipts: { fresh: [receipt(1)], warm: [receipt(5)] } } };
  assert.deepEqual(expectedNavigationCounts(report), { hover: 6, references: 6, prepare_rename: 6, rename: 6 }); assert.throws(() => expectedNavigationCounts({ ...report, status: "failed" }));
});
test("hosted formatter validator checks the finite fixture shape and refuses missing/corrupt proof", () => {
  const req = (r: string, p: string): string => announce(r, r) + begin(r, p, "navigation.request", r);
  const done = (r: string, p: string, finished = true): string => end(r, p, "navigation.request", r, finished);
  const nestedChild = 'child{request_id=999 method="other"}:';
  const fixture = req("1", "101") + begin("1", "201", "fixture.nested", "99", nestedChild) + end("1", "201", "fixture.nested", "99", true, nestedChild)
    + begin("1", "202", "fixture.nested", "99", nestedChild) + end("1", "202", "fixture.nested", "99", true, nestedChild) + done("1", "101")
    + req("2", "102") + begin("2", "203", "fixture.concurrent_a", "2") + req("3", "103") + begin("3", "204", "fixture.concurrent_b", "3")
    + end("3", "204", "fixture.concurrent_b", "3") + done("3", "103") + end("2", "203", "fixture.concurrent_a", "2") + done("2", "102")
    + req("4", "104") + begin("4", "205", "fixture.returned_error", "4") + end("4", "205", "fixture.returned_error", "4") + done("4", "104")
    + req("5", "105") + begin("5", "206", "fixture.pending_drop", "5") + end("5", "206", "fixture.pending_drop", "5", false) + done("5", "105", false)
    + announce("6", "6", "hover") + begin("6", "106", "navigation.request", "6", "", "hover") + begin("6", "207", "fixture.late_worker", "6", "", "hover") + end("6", "106", "navigation.request", "6", false, "", "hover")
    + req("7", "107") + begin("7", "208", "fixture.after_cancel", "7") + end("7", "208", "fixture.after_cancel", "7") + done("7", "107") + end("6", "207", "fixture.late_worker", "6", true, "", "hover")
    + req("8", "108") + begin("8", "209", "fixture.migrating", "8") + end("8", "209", "fixture.migrating", "8") + done("8", "108")
    + '2026-10-31T08:00:00Z  INFO vize_editor_preparation: phase="fixture.background_unbound" id=300 items=1 event="begin"\n'
    + '2026-10-31T08:00:00Z  INFO vize_editor_preparation: phase="fixture.background_unbound" id=300 completed=true elapsed_ms=1 event="end"\n';
  assert.doesNotThrow(() => inspectFormatterFixture(Buffer.from(fixture)));
  const nestedPair = begin("1", "201", "fixture.nested", "99", nestedChild) + end("1", "201", "fixture.nested", "99", true, nestedChild);
  const reversedPair = end("1", "201", "fixture.nested", "99", true, nestedChild) + begin("1", "201", "fixture.nested", "99", nestedChild);
  for (const mutated of [fixture.slice(0, -1), fixture.replace('phase="fixture.pending_drop" id=206 context_span=5 elapsed_ms=1.25 completed=false', 'phase="fixture.pending_drop" id=206 context_span=5 elapsed_ms=1.25 completed=true'), fixture.replace(announce("1", "1"), ""), fixture.replace(nestedPair, reversedPair), fixture.replaceAll('method="hover"', 'method="references"')])
    assert.throws(() => inspectFormatterFixture(Buffer.from(mutated)));
});
test("terminal preload manifest cannot hide extra sessions or attachment/finalization refusals", () => {
  const expected = { sourceRevision: "a".repeat(40), workflowRevision: "c".repeat(40) }, valid = { schema: "vize.navigation-trace-preload", version: 1, ...expected, sessionsAttempted: 2, errors: [], qualificationCredit: 0 };
  assert.doesNotThrow(() => validatePreloadTerminals([valid], expected));
  for (const values of [[], [valid, valid], [null], [{ ...valid, sourceRevision: "d".repeat(40) }], [{ ...valid, workflowRevision: "d".repeat(40) }], [{ ...valid, schema: "wrong" }], [{ ...valid, version: 2 }], [{ ...valid, sessionsAttempted: 3 }], [{ ...valid, errors: ["refused third session"] }], [{ ...valid, errors: undefined }]])
    assert.throws(() => validatePreloadTerminals(values, expected));
});
test("original stdout/stderr writes complete partial progress and refuse zero/invalid progress", () => {
  const bytes = Buffer.from("complete original output"), observed: Buffer[] = []; let calls = 0;
  writeAll(7, bytes, (fd, buffer, offset, remaining) => { assert.equal(fd, 7); calls += 1; const count = Math.min(3, remaining); observed.push(buffer.subarray(offset, offset + count)); return count; });
  assert(calls > 1); assert.deepEqual(Buffer.concat(observed), bytes);
  for (const value of [0, -1, Number.NaN, bytes.length + 1]) assert.throws(() => writeAll(7, bytes, () => value));
  let partial = 0; assert.throws(() => writeAll(7, bytes, () => { partial += 1; return partial === 1 ? 3 : 0; })); assert.equal(partial, 2);
});

test("hover owns all seven diagnostic label pairs without implying exclusive or global attribution", () => {
  assert.equal(HOVER_PHASES.length, 7);
  let text = announce("1", "1", "hover") + begin("1", "1", "navigation.request", "1", "", "hover");
  for (const [index, phase] of HOVER_PHASES.entries()) text += begin("1", String(index + 2), phase, "7", "", "hover") + end("1", String(index + 2), phase, "7", true, "", "hover");
  text += end("1", "1", "navigation.request", "1", true, "", "hover");
  const result = join(text); assert.equal(result.requests.length, 1); assert.equal(result.requests[0]?.method, "hover");
  assert.equal(result.requests[0]?.phases.length, 8); assert.equal(result.requests[0]?.complete, true);
  assert.match(result.semantics, /overlap/);
  const background = join(text + '2026-10-31T08:00:00Z  INFO vize_editor_preparation: phase="documentation" id=300 items=1 event="begin"\n');
  assert.equal(background.unboundRows, 1); assert.equal(background.attributionComplete, false);
});

test("cancelled late hover workers retain hover ancestry and cannot bind to a later rename", () => {
  const text = announce("1", "1", "hover") + begin("1", "1", "navigation.request", "1", "", "hover")
    + begin("1", "2", "navigation.native.hover", "7", "", "hover") + end("1", "1", "navigation.request", "1", false, "", "hover")
    + complete("2", "3") + end("1", "2", "navigation.native.hover", "7", true, "", "hover");
  const result = join(text); assert.equal(result.requests.find(row => row.id === "1")?.method, "hover");
  assert.equal(result.requests.find(row => row.id === "1")?.complete, false); assert.equal(result.requests.find(row => row.id === "2")?.complete, true);
  assert.equal(result.requests.find(row => row.id === "2")?.phases.some(row => row.phase === "navigation.native.hover"), false);
  const conflict = text.replace('phase="navigation.native.hover" id=2 context_span=7', 'phase="navigation.native.hover" id=2 context_span=8');
  assert.equal(join(conflict).requests.find(row => row.id === "1")?.phases.some(row => row.phase === "navigation.native.hover"), false);
});

test("the source-release sibling preserves the entire original benchmark except two custody tokens", () => {
  const original = execFileSync("git", ["show", "a2eae573558c19b1279d80db6f17fd77b1a1e633:" + ORIGINAL_BENCHMARK], { encoding: "utf8" });
  const invoked = shippingBenchmark(original); assert.equal(adapterIdentity(original, invoked).changedCustodyTokens, 2);
  assert.equal(invoked.replace("  SHIPPING_BUILD_RECIPE,\n", "  BUILD_RECIPE,\n").replace("      ? SHIPPING_BUILD_RECIPE\n", "      ? BUILD_RECIPE\n"), original);
  assert.throws(() => shippingBenchmark(original + "\n// changed\n"));
  assert.throws(() => adapterIdentity(original, invoked.replace('"--fresh"', '"--changed"')));
});

test("shipping images refuse other recipes, paths, sources, hashes or nondefault Cargo profiles", () => {
  const source = "a".repeat(40), image = "b".repeat(64), receipt = { recipe: SHIPPING_RECIPE, sourceRevision: source, binaryPath: "target/ci/vize", binarySha256: image };
  const binary = { relativeBinaryPath: "target/ci/vize", binarySha256: image, receipt }; shippingBinary(binary, source, image);
  for (const recipe of ["cargo build --profile ci -p vize", "cargo build --profile ci-opt -p vize --features profiling", "unknown"])
    assert.throws(() => shippingBinary({ ...binary, receipt: { ...receipt, recipe } }, source, image));
  for (const relativeBinaryPath of ["target/release/vize", "../foreign/vize"]) assert.throws(() => shippingBinary({ ...binary, relativeBinaryPath }, source, image));
  assert.throws(() => shippingBinary(binary, "c".repeat(40), image)); assert.throws(() => shippingBinary(binary, source, "d".repeat(64)));
  const profile = { test: false, opt_level: "3", debug_assertions: false }, artifact = { target: { name: "vize", kind: ["bin"] }, profile, features: ["default", "glyph", "maestro"] };
  shippingCargoArtifact(artifact);
  for (const change of [{ test: true }, { opt_level: "0" }, { debug_assertions: true }]) assert.throws(() => shippingCargoArtifact({ ...artifact, profile: { ...profile, ...change } }));
  for (const feature of ["profiling", "micro-profiling", "legacy"]) assert.throws(() => shippingCargoArtifact({ ...artifact, features: [...artifact.features, feature] }));
});
