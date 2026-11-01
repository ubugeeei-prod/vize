import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const ORDER = ["baseline", "candidate", "candidate", "baseline"] as const;
const IMAGE = "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70";
const SELECTOR = "lsp_client::editor_lsp::configured_project::shared_transport::tests::native::shared_contexts::production::resource::native_two_config_owner_resource_discriminator";
const MEMORY_LIMIT_KIB = 2 * 1024 * 1024;
const OUTPUT_LIMIT = 8 * 1024 * 1024;
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
const required = (name: string): string => { const value = process.env[name]; assert.ok(value, `missing ${name}`); return value; };
const write = (file: string, value: unknown): void => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
const object = (value: unknown): Record<string, unknown> => { assert.ok(value !== null && typeof value === "object" && !Array.isArray(value)); return value as Record<string, unknown>; };
const read = (file: string): Record<string, unknown> => object(JSON.parse(fs.readFileSync(file, "utf8")));
function stat(value: unknown): { pid: number; ppid: number; birth: number; state: string } {
  assert.equal(typeof value, "string"); const text = value as string;
  const split = text.lastIndexOf(") "); assert.ok(split > 2);
  const pid = Number(text.slice(0, text.indexOf(" ("))); assert.ok(Number.isSafeInteger(pid) && pid > 0);
  const fields = text.slice(split + 2).trim().split(/\s+/); assert.ok(fields.length >= 22);
  const state = fields[0]!; assert.match(state, /^[A-Z]$/); assert.notEqual(state, "Z");
  const ppid = Number(fields[1]), birth = Number(fields[19]);
  assert.ok(Number.isSafeInteger(ppid) && ppid >= 0 && Number.isSafeInteger(birth) && birth > 0);
  return { pid, ppid, birth, state };
}
function kib(value: unknown, field: string): number {
  assert.equal(typeof value, "string");
  const lines = (value as string).split("\n").filter(line => line.startsWith(field + ":")); assert.equal(lines.length, 1);
  const match = new RegExp(`^${field}:\\s+([0-9]+) kB$`).exec(lines[0]!); assert.ok(match);
  const result = Number(match[1]); assert.ok(Number.isSafeInteger(result) && result >= 0); return result;
}
function memory(row: Record<string, unknown>, expectedNative: number, executable: string, testHash: string, native: string, input: string, expectedPid: number, sourceRoot: string): void {
  assert.ok(Array.isArray(row.rows)); const members = row.rows.map(object); assert.equal(members.length, expectedNative + 1); assert.equal(row.processCount, members.length);
  assert.ok(Number(row.collectionEndUnixUs) >= Number(row.collectionStartUnixUs));
  let rss = 0, pss = 0; const ids = new Set<number>(); let rootPid = 0;
  for (const [index, member] of members.entries()) {
    const before = stat(member.statBefore), after = stat(member.statAfter);
    assert.equal(before.pid, after.pid); assert.equal(before.birth, after.birth); assert.equal(before.ppid, after.ppid);
    assert.equal(member.pid, before.pid); assert.equal(member.birth, before.birth); assert.ok(!ids.has(before.pid)); ids.add(before.pid);
    assert.ok(Number(member.collectionStartUnixUs) >= Number(row.collectionStartUnixUs)); assert.ok(Number(member.collectionEndUnixUs) <= Number(row.collectionEndUnixUs));
    const rollupRss = kib(member.smapsRollup, "Rss"), rollupPss = kib(member.smapsRollup, "Pss");
    assert.equal(member.rssKiB, rollupRss); assert.equal(member.pssKiB, rollupPss); assert.ok(rollupPss <= rollupRss); kib(member.status, "VmRSS"); rss += rollupRss; pss += rollupPss;
    assert.ok(Array.isArray(member.cmdlineBytes)); const argv = Buffer.from(member.cmdlineBytes as number[]).toString("utf8").split("\0");
    if (index === 0) { rootPid = before.pid; assert.equal(rootPid, expectedPid); assert.equal(member.cwd, sourceRoot); assert.equal(member.executable, executable); assert.equal(member.executableSHA256, testHash); assert.ok(argv.includes(SELECTOR)); }
    else { assert.equal(before.ppid, rootPid); assert.equal(member.executable, native); assert.equal(member.executableSHA256, IMAGE); assert.equal(member.cwd, input); assert.ok(argv.includes("--lsp")); }
  }
  assert.equal(row.treeRssKiB, rss); assert.equal(row.treePssKiB, pss); assert.ok(rss < MEMORY_LIMIT_KIB, "prospective 2GiB checkpoint ceiling");
}
function controls(): void {
  let passed = 0;
  const yes = (fn: () => void): void => { fn(); passed++; }, no = (fn: () => void): void => { assert.throws(fn); passed++; };
  const fixture = "42 (test (nested) name) S 10 " + Array.from({ length: 20 }, (_, i) => i === 17 ? "1234" : "0").join(" ");
  yes(() => assert.deepEqual(stat(fixture), { pid: 42, ppid: 10, birth: 1234, state: "S" }));
  for (const value of [null, {}, "", "42 (test) S 1", fixture.replace(") S ", ") Z "), fixture.replace("1234", "NaN"), fixture.replace("42 (", "0 ("), fixture.replace(") S 10", ") S -1")]) no(() => stat(value));
  yes(() => assert.equal(kib("Rss:  123 kB\nPss: 42 kB\n", "Pss"), 42));
  for (const value of [null, "Pss: 4 B", "Pss: -4 kB", "Pss: NaN kB", "Pss: 4 kB\nPss: 4 kB", "Rss: 4 kB"]) no(() => kib(value, "Pss"));
  yes(() => assert.deepEqual(ORDER, ["baseline", "candidate", "candidate", "baseline"]));
  yes(() => assert.equal(SELECTOR.endsWith("::native_two_config_owner_resource_discriminator"), true));
  for (const value of [null, [], 42, "x"]) no(() => object(value));
  const proc = (pid: number, ppid: number, birth: number): string => `${pid} (test) S ${ppid} ` + Array.from({ length: 20 }, (_, i) => i === 17 ? String(birth) : "0").join(" ");
  const member = (pid: number, ppid: number, exe: string, image: string, cwd: string, argv: string[]): Record<string, unknown> => ({ pid, birth: 1234, executable: exe, executableSHA256: image, cwd, cmdlineBytes: [...Buffer.from(argv.join("\0") + "\0")], statBefore: proc(pid, ppid, 1234), statAfter: proc(pid, ppid, 1234), status: "VmRSS: 20 kB\n", smapsRollup: "Rss: 20 kB\nPss: 10 kB\n", rssKiB: 20, pssKiB: 10, collectionStartUnixUs: 2, collectionEndUnixUs: 3 });
  const valid: Record<string, unknown> = { collectionStartUnixUs: 1, collectionEndUnixUs: 4, processCount: 2, rows: [member(42, 1, "/test", "testhash", "/source", ["/test", SELECTOR]), member(43, 42, "/native", IMAGE, "/input", ["/native", "--lsp", "--stdio"])], treeRssKiB: 40, treePssKiB: 20 };
  const verify = (row: Record<string, unknown>): void => memory(row, 1, "/test", "testhash", "/native", "/input", 42, "/source");
  yes(() => verify(valid));
  const change = (fn: (row: Record<string, unknown>, members: Record<string, unknown>[]) => void): void => { const copy = structuredClone(valid); fn(copy, copy.rows as Record<string, unknown>[]); verify(copy); };
  for (const mutate of [
    (row: Record<string, unknown>) => { row.processCount = 3; },
    (row: Record<string, unknown>) => { row.treeRssKiB = 41; },
    (row: Record<string, unknown>) => { row.treePssKiB = 19; },
    (row: Record<string, unknown>) => { row.collectionEndUnixUs = 0; },
  ]) no(() => change(mutate));
  for (const [key, value] of [["pid", 42], ["birth", 999], ["executable", "/foreign"], ["executableSHA256", "foreign"], ["cwd", "/foreign"], ["rssKiB", 21], ["pssKiB", 21], ["statAfter", proc(43, 42, 999)], ["statBefore", proc(43, 99, 1234)], ["collectionStartUnixUs", 0], ["collectionEndUnixUs", 99], ["cmdlineBytes", [...Buffer.from("/native\0--api\0")]]]) no(() => change((_row, members) => { members[1]![String(key)] = value; }));
  console.log(JSON.stringify({ passed, native: false, network: false }));
}
function main(): void {
  assert.equal(required("ROLE"), "after"); assert.equal(required("VIZE_NATIVE_DTO_ROLE"), "after"); assert.equal(required("SDK_VERSION"), "7.1.0-beta");
  const source = required("HEAD_SHA"), root = required("VIZE_NATIVE_DTO_SOURCE_ROOT"), capture = required("VIZE_NATIVE_DTO_CAPTURE");
  assert.match(source, /^[0-9a-f]{40}$/); assert.equal(required("VIZE_NATIVE_DTO_SOURCE_SHA"), source);
  assert.equal(execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(), source); assert.equal(fs.realpathSync(root), fs.realpathSync(required("GITHUB_WORKSPACE")));
  const previous = path.join(capture, "shared-contexts"); assert.equal(read(path.join(previous, "source.json")).source, source);
  assert.equal(read(path.join(previous, "execution.json")).status, 0); assert.match(fs.readFileSync(path.join(previous, "stdout.bin"), "utf8"), /test result: ok\. 1 passed; 0 failed; 0 ignored;/);
  const native = required("VIZE_NATIVE_DTO_ACTUAL_RUNTIME_PATH"); assert.equal(hash(fs.readFileSync(native)), IMAGE);
  const artifacts = fs.readFileSync(path.join(capture, "build.jsonl"), "utf8").trim().split("\n").map(line => object(JSON.parse(line))).filter(row => row.reason === "compiler-artifact" && object(row.target).name === "vize_canon" && object(row.profile).test === true && row.executable);
  assert.equal(artifacts.length, 1); const artifact = artifacts[0]!, executable = String(artifact.executable), testHash = hash(fs.readFileSync(executable)); assert.equal(artifact.fresh, false);
  assert.equal(fs.realpathSync(String(object(artifact.target).src_path)), fs.realpathSync(path.join(root, "crates/vize_canon/src/lib.rs")));
  const relative = path.relative(fs.realpathSync(required("CARGO_TARGET_DIR")), fs.realpathSync(executable)); assert.ok(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  const output = path.join(capture, "shared-contexts-resource"); assert.ok(!fs.existsSync(output)); fs.mkdirSync(output);
  const space = fs.statfsSync(output); assert.ok(space.bavail * space.bsize >= 1024 * 1024 * 1024 + OUTPUT_LIMIT);
  const files = ["Cargo.toml", "Cargo.lock", ".github/workflows/native-beta-diagnostic-dto.yml", "tests/tooling/support/native-beta-dto/shared-contexts-run.ts", "tests/tooling/support/native-beta-dto/shared-contexts-resource-run.ts", "crates/vize_canon/src/lsp_client/editor_lsp/completion_resolve.rs", "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native.rs", "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts.rs", "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts/tests.rs", "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts/production.rs", "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts/production/resource.rs"];
  const sourceRows = files.map(file => { const bytes = fs.readFileSync(path.join(root, file)); assert.deepEqual(bytes, execFileSync("git", ["show", `${source}:${file}`], { cwd: root, maxBuffer: 16 * 1024 * 1024 })); return { path: file, bytes: bytes.length, sha256: hash(bytes) }; });
  const input = path.join(output, "input"); fs.mkdirSync(input); fs.mkdirSync(path.join(input, "a")); fs.mkdirSync(path.join(input, "b"));
  const fixture = { "a/source.ts": "export const answer: number = 'wrong';\nanswer.toFixed();\n", "b/source.ts": "export const label: string = 'clean';\nlabel.toUpperCase();\n", "tsconfig.json": JSON.stringify({ compilerOptions: { strict: true, noEmit: true, types: [], target: "ESNext", module: "ESNext", moduleResolution: "Bundler" }, files: ["a/source.ts"] }), "b/tsconfig.json": JSON.stringify({ compilerOptions: { strict: false, noEmit: true, types: [], target: "ESNext", module: "ESNext", moduleResolution: "Bundler" }, files: ["source.ts"] }) };
  const fixtureRows = Object.entries(fixture).map(([file, text]) => { fs.writeFileSync(path.join(input, file), text, { flag: "wx" }); return { path: file, bytes: Buffer.byteLength(text), sha256: hash(Buffer.from(text)) }; });
  write(path.join(output, "source.json"), { source, sdkVersion: "7.1.0-beta", native, nativeSHA256: IMAGE, buildArtifact: artifact, testExecutable: executable, testExecutableSHA256: testHash, files: sourceRows, fixtureRoot: input, fixture: fixtureRows, selector: SELECTOR, order: ORDER, scope: "two configured projects in actual workspace clients; Rust test-root plus native descendants, not Maestro pool; no trace; OS page cache uncontrolled" });
  const commonEnvironment = { ...process.env, CORSA_PATH: native, VIZE_TEST_REQUIRE_TSGO: "1", VIZE_RESOURCE_INPUT_ROOT: input, VIZE_RESOURCE_TEST_IMAGE_SHA256: testHash };
  const results: unknown[] = []; let decoded = 0;
  for (const [index, mode] of ORDER.entries()) {
    const directory = path.join(output, `${index + 1}-${mode}`); fs.mkdirSync(directory);
    const stdout = fs.openSync(path.join(directory, "stdout.bin"), "wx"), stderr = fs.openSync(path.join(directory, "stderr.bin"), "wx");
    const args = [SELECTOR, "--ignored", "--exact", "--nocapture", "--test-threads=1"];
    const started = Date.now(); let result: ReturnType<typeof spawnSync>;
    try { result = spawnSync(executable, args, { cwd: root, env: { ...commonEnvironment, VIZE_RESOURCE_MODE: mode, VIZE_PRIVATE_SHARED_CONTEXT_CAPTURE: path.join(directory, "packets") }, stdio: ["ignore", stdout, stderr], timeout: 120_000 }); }
    finally { fs.closeSync(stdout); fs.closeSync(stderr); }
    write(path.join(directory, "execution.json"), { executable, args, mode, source, native, pid: result.pid, image: IMAGE, testImage: testHash, input, startedUnixMs: started, endedUnixMs: Date.now(), status: result.status, signal: result.signal, error: result.error?.message ?? null, caseEnvironmentDifferences: ["VIZE_RESOURCE_MODE", "VIZE_PRIVATE_SHARED_CONTEXT_CAPTURE"] });
    assert.equal(result.status, 0); assert.equal(result.signal, null); assert.equal(result.error, undefined);
    assert.match(fs.readFileSync(path.join(directory, "stdout.bin"), "utf8"), /test result: ok\. 1 passed; 0 failed; 0 ignored;/); assert.equal(fs.statSync(path.join(directory, "stderr.bin")).size, 0);
    assert.equal(hash(fs.readFileSync(native)), IMAGE); assert.equal(hash(fs.readFileSync(executable)), testHash);
    for (const row of fixtureRows) assert.equal(hash(fs.readFileSync(path.join(input, row.path))), row.sha256);
    const packets = path.join(directory, "packets"); const roster = fs.readdirSync(packets).sort(); const memoryRows: unknown[] = [];
    for (const file of roster) { assert.match(file, /^[a-z0-9-]+\.json$/); const bytes = fs.readFileSync(path.join(packets, file)); decoded += bytes.length; assert.ok(bytes.length < 1024 * 1024 && decoded <= OUTPUT_LIMIT); if (file.startsWith("resource-memory-")) { const row = object(JSON.parse(bytes.toString("utf8"))); const phase = String(row.milestone); memory(row, ["root-before", "owners-retired", "final"].includes(phase) ? 0 : mode === "baseline" ? 2 : 1, executable, testHash, native, input, result.pid, root); memoryRows.push(row); } }
    assert.equal(memoryRows.length, 11);
    assert.deepEqual(memoryRows.map(value => String(object(value).milestone)).sort(), ["root-before", "initial", "strict-null", "repaired", "current-boolean", "a-detached", "a-reopened-b-error", "repaired-leases-dropped", "owners-retired", "restarted", "final"].sort());
    write(path.join(directory, "roster.json"), roster.map(file => { const bytes = fs.readFileSync(path.join(packets, file)); return { file, bytes: bytes.length, sha256: hash(bytes) }; }));
    results.push({ index: index + 1, mode, memoryRows, firstUsable: read(path.join(packets, "resource-first-usable.json")), packetCount: roster.length });
  }
  write(path.join(output, "result.json"), { source, order: ORDER, results, decodedPacketBytes: decoded, scope: "prospective same-image finite memory discriminator; all raw packets preserved; not a production pool, cold OS cache, peak or speedup claim" });
  console.log(JSON.stringify({ source, order: ORDER, functionalCases: 4, rawPacketBytes: decoded }));
}
if (process.argv[2] === "--controls") controls(); else main();
