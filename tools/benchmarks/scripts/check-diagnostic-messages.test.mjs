import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  authored,
  captureMembers,
  createOutputDirectory,
  assertCounters,
  assertPacket,
  candidates,
  cases,
  counters,
  frozenSource,
  largeSource,
  nativeVector,
  pinLexer,
  sha256,
  vector,
} from "./check-diagnostic-messages.mjs";

void test("message corpus reproduces the original complete 128-error source bytes", () => {
  const source = largeSource();
  assert.equal(Buffer.byteLength(source), 1589305);
  assert.equal(sha256(source), "3e665da0ae24805070dd0c982b9e6077cfef5313507393e1ca70326a5eb19f3b");
  assert.equal((source.match(/const document\d+:/gu) ?? []).length, 128);
  assert.equal(Buffer.byteLength(frozenSource), 269);
  assert.equal(
    sha256(frozenSource),
    "0426f2fd026efc85e0513765f9e5c950d4acd6ada07ed4e4f7c640f865b2642c",
  );
});
void test("message corpus preserves separately authored full, bounded and omitted options", () => {
  const rows = cases();
  assert.deepEqual(
    rows.map((r) => r.name),
    [
      "frozen",
      "large-full",
      "large-authored-bounded",
      "large-default",
      "authored-unicode-escape-chains",
    ],
  );
  const [full, bounded, defaults] = rows.slice(1, 4);
  assert.equal(full.options.noErrorTruncation, true);
  assert.equal(bounded.options.noErrorTruncation, false);
  assert.equal(Object.hasOwn(defaults.options, "noErrorTruncation"), false);
  assert.deepEqual(full.sources, bounded.sources);
  assert.deepEqual(full.sources, defaults.sources);
  const edge = rows.at(-1).sources["src/entry.ts"];
  for (const part of [
    "実験🙂",
    "\\u0041",
    "/__vize_authored_vue_ts__",
    ".__vize_authored_vue_ts_alias__",
  ])
    assert.ok(edge.includes(part));
});
for (const [name, message, expected] of [
  ["group order", "Module \"./A.vue.ts\" and './B.vue.ts'.", ["./B.vue.ts", "./A.vue.ts"]],
  ["nested quote", "Module '\"./A.vue.ts\"'.", ["./A.vue.ts"]],
  ["duplicates", "'./A.vue.ts' './A.vue.ts' \"./A.vue.ts\" ‘./A.vue.ts’", ["./A.vue.ts"]],
  ["backtick", "`./A.vue.ts` ./B.vue.ts", []],
  ["whitespace", "'wide\u3000space' 'ordinary prose'", []],
  ["Unicode bytes", "'./実験🙂.vue.ts'", ["./実験🙂.vue.ts"]],
  ["literal escape", "'./\\u0041.vue.ts'", ["./\\u0041.vue.ts"]],
  ["unbalanced", "'unfinished", []],
  ["Rust control whitespace boundary", "'./a\u001c.vue.ts'", ["./a\u001c.vue.ts"]],
  ["Rust BOM whitespace boundary", "'./a\ufeff.vue.ts'", ["./a\ufeff.vue.ts"]],
  ["Rust NEL whitespace boundary", "'./a\u0085.vue.ts'", []],
])
  void test(`native candidate inventory preserves ${String(name)}`, () =>
    assert.deepEqual(candidates(message), expected));

void test("candidate eligibility retains all marker branches and rejects unrelated suffixes", () => {
  for (const [reported, expected] of [
    ["./A.vue.ts", "./A.vue"],
    ["./A.vue.tsx", "./A.vue"],
    ["./A.vue.ts/__vize_authored_vue_ts__", "./A.vue.ts"],
    ["./A.vue.tsx.__vize_authored_vue_ts_alias__", "./A.vue.tsx"],
    ["./A.vue.ts/__vize_missing_vue_import__.js", "./A.vue"],
    ["./util.ts", null],
    ["./A.vue.ts.extra", null],
    ["./plain.ts/__vize_authored_vue_ts__", null],
  ])
    assert.equal(authored(reported), expected);
  assert.deepEqual(vector("\"./A.vue.ts\" and './A.vue.ts/__vize_authored_vue_ts__'"), [
    { reported: "./A.vue.ts/__vize_authored_vue_ts__", authored: "./A.vue.ts" },
    { reported: "./A.vue.ts", authored: "./A.vue" },
  ]);
});
const packet = (messages = ["one🙂", "two違"]) =>
  Buffer.from(
    JSON.stringify({
      files: [{ file: "src/a.ts", diagnostics: messages }],
      errorCount: messages.length,
      warningCount: 0,
    }),
  );
const processResult = (stdout) => ({
  stdout,
  stderr: Buffer.alloc(0),
  status: 1,
  signal: null,
  error: undefined,
});
void test("packet oracle compares complete ordered bytes rather than diagnostic counts", () => {
  const expected = packet();
  assertPacket(processResult(expected), expected);
  for (const changed of [
    packet(["one🙂", "changed"]),
    packet(["two違", "one🙂"]),
    packet(["one🙂"]),
    Buffer.concat([expected, Buffer.from("\n")]),
  ])
    assert.throws(() => assertPacket(processResult(changed), expected));
  assert.throws(() => assertPacket(processResult(Buffer.from([0xff]))));
  for (const changes of [
    { stderr: Buffer.from("notice") },
    { signal: "SIGTERM" },
    { error: Error("failed") },
    { status: 0 },
  ])
    assert.throws(() => assertPacket({ ...processResult(expected), ...changes }, expected));
});
function frozenCounters(side) {
  const values = {
    calls: 8,
    message_input_bytes: 486,
    quoted_candidates: 14,
    eligible_candidates: 1,
    rewrites: 1,
    source_searches: side === "base" ? 14 : 1,
    source_search_input_bytes: side === "base" ? 3766 : 269,
    source_search_ns: 0,
  };
  return Object.fromEntries(
    Object.entries(values).map(([key, total]) => [key, { total, samples: 8 }]),
  );
}
void test("profile comparison preserves inventory and permits only actual search differences", () => {
  const base = frozenCounters("base");
  const head = frozenCounters("head");
  assertCounters(base, head, "frozen");
  for (const key of [
    "calls",
    "message_input_bytes",
    "quoted_candidates",
    "eligible_candidates",
    "rewrites",
    "source_searches",
    "source_search_input_bytes",
  ]) {
    const changed = structuredClone(head);
    changed[key].total += 1;
    assert.throws(() => assertCounters(base, changed, "frozen"), key);
  }
  const changed = structuredClone(head);
  changed.source_search_ns.samples -= 1;
  assert.throws(() => assertCounters(base, changed, "frozen"));
});
void test("profile parser rejects truncation, missing and duplicate counter rows", () => {
  const rows = Object.entries(frozenCounters("base")).map(([key, value]) => ({
    key: `canon.diagnostics.restore_specifiers.${key}`,
    ...value,
  }));
  const profile = {
    schema_version: 1,
    truncation: { dropped_spans: 0, dropped_counters: 0 },
    counters: rows,
  };
  assert.deepEqual(counters(profile), frozenCounters("base"));
  for (const changed of [
    { ...profile, truncation: { dropped_spans: 1, dropped_counters: 0 } },
    { ...profile, counters: rows.slice(1) },
    { ...profile, counters: [...rows.slice(1), rows[1]] },
  ])
    assert.throws(() => counters(changed));
});
void test("zero-eligible profile requires zero real searches, source input and search time", () => {
  const base = frozenCounters("base");
  const head = frozenCounters("head");
  for (const r of [base, head]) {
    r.eligible_candidates.total = 0;
    r.rewrites.total = 0;
  }
  head.source_searches.total = 0;
  head.source_search_input_bytes.total = 0;
  assertCounters(base, head, "large-authored-bounded");
  for (const key of ["source_searches", "source_search_input_bytes", "source_search_ns"]) {
    const changed = structuredClone(head);
    changed[key].total = 1;
    assert.throws(() => assertCounters(base, changed, "large-authored-bounded"));
  }
});
void test("lexical authority rejects classifier, quote scanner and replacement changes", () => {
  const source = fs.readFileSync(
    new URL("../../../crates/vize_canon/src/batch/virtual_specifier_message.rs", import.meta.url),
    "utf8",
  );
  assert.equal(pinLexer(source, source).length, 6);
  for (const needle of [
    "strip_suffix(MISSING_VUE_IMPORT_SENTINEL)",
    "character.is_whitespace()",
    "/__vize_missing_vue_import__.js",
    "/__vize_authored_vue_ts__",
    ".__vize_authored_vue_ts_alias__",
    "!found.contains(&candidate)",
    "rewritten.replace(quoted.as_str(), restored.as_str())",
    String.raw`('\u{2018}', '\u{2019}')`,
    "fn authored_specifier(",
  ]) {
    assert.ok(source.includes(needle), needle);
    assert.throws(() => pinLexer(source, source.replace(needle, "changed")), needle);
  }
});
void test("native capture retains whole lines, argv, ordered members and rejects lossy layouts", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-message-native-test-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const stem = path.join(root, "123");
  const metadata = {
    runtime: "/runtime",
    layout: "producer-ordered-literal-include-empty-exclude",
    args: ["--project", "/config"],
    cwd: "/cwd",
    configPath: "/config",
    members: [{ name: "entry.ts", file: "/entry.ts", bytes: 9 }],
    status: 1,
    signal: null,
    error: null,
  };
  fs.writeFileSync(`${stem}.json`, JSON.stringify(metadata));
  fs.writeFileSync(
    `${stem}.tsconfig.json`,
    '{"compilerOptions":{},"include":["entry.ts"],"exclude":[]}',
  );
  fs.writeFileSync(`${stem}.member-0.txt`, "const x=1");
  const stdout = "src/a.ts(1,2): error TS2322: Type '違' differs from '🙂'.\n";
  fs.writeFileSync(`${stem}.stdout`, stdout);
  fs.writeFileSync(`${stem}.stderr`, "");
  const [row] = nativeVector(root);
  assert.equal(row.stdoutSha256, sha256(stdout));
  assert.deepEqual(row.messages, [
    {
      rawLine: stdout.trimEnd(),
      reportedCandidates: [
        { reported: "違", authored: null },
        { reported: "🙂", authored: null },
      ],
    },
  ]);
  assert.equal(row.metadata.members[0].sha256, sha256("const x=1"));
  fs.writeFileSync(`${stem}.stdout`, stdout + "unparsed continuation\n");
  assert.throws(() => nativeVector(root));
  fs.writeFileSync(`${stem}.stdout`, stdout);
  fs.writeFileSync(`${stem}.stderr`, "unexpected");
  assert.throws(() => nativeVector(root));
});

void test("native member capture reads the actual literal include producer shape without globbing", (t) => {
  const temporary = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-message-members-")),
  );
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const root = path.join(temporary, "canon", "projects", "a".repeat(64));
  fs.mkdirSync(path.join(root, "src"), { recursive: true });
  fs.writeFileSync(path.join(root, "src/entry.ts"), "const label = '🙂';\n");
  fs.writeFileSync(path.join(root, "__vize_shared.d.ts"), "export {};\n");
  const configPath = path.join(root, "tsconfig.json");
  const config = {
    compilerOptions: { strict: true },
    include: ["__vize_shared.d.ts", "src/entry.ts"],
    exclude: [],
  };
  fs.writeFileSync(configPath, JSON.stringify(config));
  const captured = captureMembers(configPath, config);
  assert.equal(captured.virtualRoot, root);
  assert.deepEqual(captured.members, [
    { name: "__vize_shared.d.ts", file: path.join(root, "__vize_shared.d.ts"), bytes: 11 },
    {
      name: "src/entry.ts",
      file: path.join(root, "src/entry.ts"),
      bytes: Buffer.byteLength("const label = '🙂';\n"),
    },
  ]);
  for (const changed of [
    { ...config, include: ["src/**/*.ts"] },
    { ...config, include: ["../escape.ts"] },
    { ...config, include: ["/absolute.ts"] },
    { ...config, include: ["src/entry.ts", "src/entry.ts"] },
    { ...config, include: [...config.include].reverse() },
    { ...config, include: ["missing.ts"] },
    { ...config, exclude: ["src"] },
    { ...config, files: config.include },
    { ...config, references: [] },
    { ...config, include: [] },
  ])
    assert.throws(() => captureMembers(configPath, changed));
  const unknown = path.join(temporary, "tsconfig.json");
  fs.writeFileSync(unknown, JSON.stringify(config));
  assert.throws(() => captureMembers(unknown, config));
  fs.symlinkSync(path.join(root, "src/entry.ts"), path.join(root, "src/alias.ts"));
  assert.throws(() => captureMembers(configPath, { ...config, include: ["src/alias.ts"] }));
});

void test("evidence ownership requires an absent entry including dangling symlinks", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-message-output-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const absent = path.join(root, "absent");
  createOutputDirectory(absent);
  assert.ok(fs.statSync(absent).isDirectory());
  assert.throws(() => createOutputDirectory(absent));
  const file = path.join(root, "file");
  fs.writeFileSync(file, "retained");
  assert.throws(() => createOutputDirectory(file));
  const dangling = path.join(root, "dangling");
  fs.symlinkSync(path.join(root, "missing"), dangling);
  assert.throws(() => createOutputDirectory(dangling));
  assert.ok(fs.lstatSync(dangling).isSymbolicLink());
  assert.equal(fs.readFileSync(file, "utf8"), "retained");
});
