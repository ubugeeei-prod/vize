import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";

const utf8 = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const quotePairs = [
  ["'", "'"],
  ['"', '"'],
  ["‘", "’"],
];
const whitespace = new Set([
  9,
  10,
  11,
  12,
  13,
  32,
  133,
  160,
  5760,
  ...Array.from({ length: 11 }, (_, i) => 8192 + i),
  8232,
  8233,
  8239,
  8287,
  12288,
]);
export function candidates(message) {
  const found = [];
  for (const [open, close] of quotePairs) {
    let rest = message;
    while (rest.includes(open)) {
      rest = rest.slice(rest.indexOf(open) + open.length);
      const end = rest.indexOf(close);
      if (end < 0) break;
      const candidate = rest.slice(0, end);
      rest = rest.slice(end + close.length);
      if (
        candidate &&
        ![...candidate].some((character) => whitespace.has(character.codePointAt(0))) &&
        !/['"`]/u.test(candidate) &&
        !found.includes(candidate)
      )
        found.push(candidate);
    }
  }
  return found;
}
export function authored(reported) {
  const missing = "/__vize_missing_vue_import__.js";
  const mirror = (value) => {
    for (const suffix of [".ts", ".tsx"])
      if (value.endsWith(suffix) && value.slice(0, -suffix.length).endsWith(".vue"))
        return value.slice(0, -suffix.length);
    return null;
  };
  if (reported.endsWith(missing)) {
    const result = mirror(reported.slice(0, -missing.length));
    if (result !== null) return result;
  }
  for (const marker of ["/__vize_authored_vue_ts__", ".__vize_authored_vue_ts_alias__"])
    if (reported.endsWith(marker)) {
      const value = reported.slice(0, -marker.length);
      if (value.endsWith(".vue.ts") || value.endsWith(".vue.tsx")) return value;
    }
  return mirror(reported);
}
export function vector(message) {
  return candidates(message).map((reported) => ({ reported, authored: authored(reported) }));
}
export function largeSource() {
  const header =
    'export {};\ninterface TypedDocumentNode<Data, Variables> {\n  readonly kind: "Document";\n  readonly __apiType?: (variables: Variables) => Data;\n}\n';
  const fields = Array.from({ length: 64 }, (_, index) => {
    const n = String(index).padStart(3, "0");
    return `      readonly profileField${n}: { readonly __typename: "Profile${n}"; readonly label: string; readonly value: string | null };\n`;
  }).join("");
  const annotation =
    'TypedDocumentNode<\n  {\n    readonly __typename: "Query";\n    readonly viewer: {\n      readonly __typename: "User";\n      readonly id: string;\n' +
    fields +
    "    };\n  },\n  { readonly viewerId: string; readonly preview: boolean }\n>";
  const source =
    "/*" +
    "x".repeat(524288) +
    "*/\n" +
    header +
    Array.from(
      { length: 128 },
      (_, i) => `const document${i}: ${annotation} = 1;\nvoid document${i};\n`,
    ).join("");
  assert.equal(Buffer.byteLength(source), 1589305);
  assert.equal(sha256(source), "3e665da0ae24805070dd0c982b9e6077cfef5313507393e1ca70326a5eb19f3b");
  return source;
}
export const frozenSource =
  'type Required = { readonly kind: "expected"; readonly nested: { readonly label: "🙂"; readonly enabled: true } };\nconst value: Required = { kind: "actual", nested: { label: "違う", enabled: false } };\nimport Missing from "./Missing.vue";\nexport { value, Missing };\n';
export function cases() {
  const large = largeSource();
  const options = {
    strict: true,
    target: "ES2022",
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    skipLibCheck: true,
    types: [],
  };
  const entry = [
    'import Missing from "./Missing.vue";',
    'import Unicode from "./実験🙂.vue";',
    'import Escaped from "./\\u0041.vue";',
    'import Authored from "./Literal.vue.ts/__vize_authored_vue_ts__";',
    'type First = "./A.vue.ts";',
    'type Second = "./A.vue.ts/__vize_authored_vue_ts__";',
    "const chain: First = null as unknown as Second;",
    'const alias: "./B.vue.ts.__vize_authored_vue_ts_alias__" = "different";',
    "export { Missing, Unicode, Escaped, Authored, chain, alias };",
    "",
  ].join("\n");
  return [
    {
      name: "frozen",
      sources: { "src/entry.ts": frozenSource },
      options: {
        strict: true,
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "bundler",
        noEmit: true,
        noErrorTruncation: true,
      },
      errors: 4,
    },
    ...[
      ["large-full", true],
      ["large-authored-bounded", false],
      ["large-default", undefined],
    ].map(([name, value]) => ({
      name,
      sources: { "src/case.ts": large },
      options: { ...options, ...(value === undefined ? {} : { noErrorTruncation: value }) },
      errors: 128,
    })),
    {
      name: "authored-unicode-escape-chains",
      sources: { "src/entry.ts": entry, "src/Missing.vue.ts": "export default 1;\n" },
      options: { ...options, noErrorTruncation: true },
      errors: null,
    },
  ];
}
export function assertPacket(result, expected = null) {
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 1);
  assert.deepEqual(result.stderr, Buffer.alloc(0));
  const report = JSON.parse(utf8.decode(result.stdout));
  assert.ok(report.errorCount > 0);
  assert.equal(report.warningCount, 0);
  assert.equal(report.files.flatMap((file) => file.diagnostics).length, report.errorCount);
  if (expected !== null) assert.deepEqual(result.stdout, expected, "complete report bytes changed");
  return report;
}
export function counters(profile) {
  assert.equal(profile.schema_version, 1);
  assert.deepEqual(profile.truncation, { dropped_spans: 0, dropped_counters: 0 });
  const rows = profile.counters.filter((row) =>
    row.key.startsWith("canon.diagnostics.restore_specifiers."),
  );
  assert.equal(rows.length, 8);
  assert.equal(new Set(rows.map((row) => row.key)).size, 8);
  return Object.fromEntries(
    rows.map((row) => [
      row.key.slice("canon.diagnostics.restore_specifiers.".length),
      { total: row.total, samples: row.samples },
    ]),
  );
}
export function assertCounters(base, head, name) {
  for (const key of [
    "calls",
    "message_input_bytes",
    "quoted_candidates",
    "eligible_candidates",
    "rewrites",
  ])
    assert.deepEqual(head[key], base[key], key);
  assert.equal(base.source_searches.total, base.quoted_candidates.total);
  assert.equal(head.source_searches.total, head.eligible_candidates.total);
  assert.ok(head.source_search_input_bytes.total <= base.source_search_input_bytes.total);
  for (const key of ["source_searches", "source_search_input_bytes", "source_search_ns"])
    assert.equal(head[key].samples, base[key].samples);
  if (name === "frozen") {
    for (const [key, value] of Object.entries({
      calls: 8,
      message_input_bytes: 486,
      quoted_candidates: 14,
      eligible_candidates: 1,
      rewrites: 1,
    }))
      assert.equal(base[key].total, value);
    assert.equal(base.source_search_input_bytes.total, 3766);
    assert.equal(head.source_search_input_bytes.total, 269);
  }
  if (name.startsWith("large-")) {
    assert.equal(head.eligible_candidates.total, 0);
    assert.equal(head.source_search_input_bytes.total, 0);
    assert.equal(head.source_search_ns.total, 0);
    if (name === "large-full") assert.equal(base.quoted_candidates.total, 17152);
  }
}
export function pinLexer(base, head) {
  const sections = [
    ["pub const AUTHORED_VUE_TS_SENTINEL", "/// The quote pairs a checker message"],
    ["pub(crate) const QUOTE_PAIRS", "/// Restore virtual Vue"],
    ["fn authored_specifier(", "/// Every distinct specifier-shaped"],
    ["pub(crate) fn quoted_specifiers(", "/// The authored `.vue` spelling"],
    ["fn is_specifier_shaped(", "#[expect(clippy::disallowed_macros"],
    ["    let mut rewritten = message.to_compact_string();", "fn authored_specifier("],
  ];
  return sections.map(([start, end]) => {
    function section(text) {
      const offset = text.indexOf(start);
      assert.ok(offset >= 0 && text.indexOf(start, offset + 1) < 0);
      const finish = text.indexOf(end, offset + start.length);
      assert.ok(finish > offset);
      return text.slice(offset, finish);
    }
    const before = section(base);
    assert.equal(section(head), before, `unchanged lexical/replacement source: ${start}`);
    return { start, end, sha256: sha256(before) };
  });
}
function writeJson(file, value) {
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}
export function createOutputDirectory(directory) {
  assert.equal(
    fs.lstatSync(directory, { throwIfNoEntry: false }),
    undefined,
    "output entry must initially be absent",
  );
  fs.mkdirSync(directory);
}
function inventory(directory) {
  const result = {};
  function visit(relative) {
    const full = path.join(directory, relative);
    for (const item of fs
      .readdirSync(full, { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name, "en"))) {
      const next = path.join(relative, item.name);
      if (item.isDirectory()) visit(next);
      else {
        assert.ok(item.isFile());
        result[next] = sha256(fs.readFileSync(path.join(directory, next)));
      }
    }
  }
  visit("src");
  for (const name of ["package.json", "tsconfig.json"])
    result[name] = sha256(fs.readFileSync(path.join(directory, name)));
  return result;
}
function cleanEnvironment(runtime) {
  const env = Object.fromEntries(
    Object.entries(process.env).filter(
      ([key]) =>
        !/^(VIZE_|CORSA_|TSGO_|NAPI_RS_)/u.test(key) &&
        !["NODE_OPTIONS", "NODE_PATH", "FORCE_COLOR", "CLICOLOR_FORCE"].includes(key),
    ),
  );
  return { ...env, NO_COLOR: "1", CORSA_PATH: runtime };
}
function literalIncludes(config) {
  assert.deepEqual(Object.keys(config).sort(), ["compilerOptions", "exclude", "include"]);
  assert.ok(
    config.compilerOptions &&
      typeof config.compilerOptions === "object" &&
      !Array.isArray(config.compilerOptions),
  );
  assert.deepEqual(config.exclude, []);
  assert.ok(Array.isArray(config.include) && config.include.length > 0);
  const names = config.include;
  assert.deepEqual(
    names,
    [...new Set(names)].sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b))),
    "producer's distinct UTF-8 member order",
  );
  for (const name of names) {
    assert.equal(typeof name, "string");
    assert.ok(
      name.length > 0 &&
        !["*", "?", "[", "]", "{", "}", "!", "\\"].some((special) => name.includes(special)) &&
        !path.isAbsolute(name),
      "only literal relative include members",
    );
  }
  return names;
}
export function captureMembers(configPath, config) {
  const names = literalIncludes(config);
  const resolvedConfig = fs.realpathSync(configPath);
  assert.equal(resolvedConfig, path.resolve(configPath), "config aliases are not admitted");
  let virtualRoot = path.dirname(resolvedConfig);
  while (
    path.dirname(virtualRoot) !== virtualRoot &&
    !(
      /^[a-f0-9]{64}$/u.test(path.basename(virtualRoot)) &&
      path.basename(path.dirname(virtualRoot)) === "projects" &&
      path.basename(path.dirname(path.dirname(virtualRoot))) === "canon"
    )
  )
    virtualRoot = path.dirname(virtualRoot);
  assert.match(path.basename(virtualRoot), /^[a-f0-9]{64}$/u, "unknown Canon config owner");
  assert.equal(path.basename(path.dirname(virtualRoot)), "projects");
  const members = names.map((name) => {
    const file = path.resolve(path.dirname(resolvedConfig), name);
    const relative = path.relative(virtualRoot, file);
    assert.ok(
      relative &&
        !relative.startsWith(`..${path.sep}`) &&
        relative !== ".." &&
        !path.isAbsolute(relative),
      "member escapes virtual owner",
    );
    assert.equal(fs.realpathSync(file), file, "member aliases are not admitted");
    assert.ok(fs.statSync(file).isFile());
    return { name, file, bytes: fs.statSync(file).size };
  });
  return { layout: "producer-ordered-literal-include-empty-exclude", virtualRoot, members };
}
function captureWrapper(output, runtime) {
  const location = path.join(output, "capture-location.txt");
  const wrapper = path.join(output, "capture-native.mjs");
  const code = `#!${process.execPath}
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { captureMembers } from ${JSON.stringify(import.meta.url)};
const runtime = ${JSON.stringify(runtime)};
const dir = fs.readFileSync(${JSON.stringify(location)}, "utf8");
const args = process.argv.slice(2);
const result = spawnSync(runtime, args, { env: process.env, maxBuffer: 64 * 1024 * 1024 });
const stem = path.join(dir, String(process.pid));
fs.writeFileSync(stem + ".stdout", result.stdout ?? "");
fs.writeFileSync(stem + ".stderr", result.stderr ?? "");
const projectIndex = args.indexOf("--project");
if (projectIndex < 0 || args.lastIndexOf("--project") !== projectIndex) throw Error("unknown project argv");
const configPath = path.resolve(args[projectIndex + 1]);
const configBytes = fs.readFileSync(configPath);
fs.writeFileSync(stem + ".tsconfig.json", configBytes);
const declared = captureMembers(configPath, JSON.parse(configBytes));
for (const [index, member] of declared.members.entries()) fs.writeFileSync(stem + ".member-" + String(index) + ".txt", fs.readFileSync(member.file));
fs.writeFileSync(stem + ".json", JSON.stringify({ runtime, args, cwd: process.cwd(), configPath, ...declared, status: result.status, signal: result.signal, error: result.error ? String(result.error) : null }));
process.stdout.write(result.stdout ?? "");
process.stderr.write(result.stderr ?? "");
if (result.error) process.exitCode = 125;
else if (result.signal) process.kill(process.pid, result.signal);
else process.exitCode = result.status;
`;
  fs.writeFileSync(wrapper, code, { mode: 0o755 });
  return { wrapper, location, sha256: sha256(code) };
}
export function nativeVector(directory) {
  const files = fs.readdirSync(directory).sort();
  const expected = new Set();
  const result = files
    .filter((name) => /^\d+\.json$/u.test(name))
    .sort()
    .map((name) => {
      const stem = path.join(directory, name.slice(0, -5));
      const metadata = JSON.parse(fs.readFileSync(`${stem}.json`, "utf8"));
      for (const suffix of [".json", ".stdout", ".stderr", ".tsconfig.json"])
        expected.add(path.basename(stem) + suffix);
      assert.equal(metadata.error, null);
      assert.equal(metadata.signal, null);
      assert.equal(metadata.status, 1);
      const stdout = fs.readFileSync(`${stem}.stdout`);
      assert.deepEqual(fs.readFileSync(`${stem}.stderr`), Buffer.alloc(0));
      const messages = utf8
        .decode(stdout)
        .split("\n")
        .filter(Boolean)
        .map((line) => {
          const match = /^(.*)\((\d+),(\d+)\): (error|warning) TS(\d+): (.*)$/u.exec(line);
          assert.ok(match, "unknown native diagnostic layout");
          return { rawLine: line, reportedCandidates: vector(match[6]) };
        });
      const config = fs.readFileSync(`${stem}.tsconfig.json`);
      assert.equal(metadata.layout, "producer-ordered-literal-include-empty-exclude");
      assert.deepEqual(
        metadata.members.map((member) => member.name),
        literalIncludes(JSON.parse(config)),
      );
      for (const [index, member] of metadata.members.entries()) {
        const name = `${path.basename(stem)}.member-${index}.txt`;
        expected.add(name);
        const bytes = fs.readFileSync(path.join(directory, name));
        assert.equal(bytes.length, member.bytes);
        member.sha256 = sha256(bytes);
      }
      return {
        metadata,
        configSha256: sha256(config),
        stdoutSha256: sha256(stdout),
        stdoutBytes: stdout.length,
        messages,
      };
    });
  assert.deepEqual(
    files,
    [...expected].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0)),
    "unknown or unowned native capture file",
  );
  return result;
}
const median = (values) => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
export function main(args) {
  const [baseInput, headInput, baseSha, headSha, outputInput] = args;
  assert.ok(baseInput && headInput && outputInput);
  for (const sha of [baseSha, headSha]) assert.match(sha, /^[a-f0-9]{40}$/u);
  assert.notEqual(baseSha, headSha);
  const output = path.resolve(outputInput);
  createOutputDirectory(output);
  const repository = fileURLToPath(new URL("../../../", import.meta.url));
  const runtime = fs.realpathSync(
    path.join(
      repository,
      "node_modules/@typescript",
      `typescript-${process.platform}-${process.arch}`,
      "lib/tsc",
    ),
  );
  const runtimePackage = JSON.parse(
    fs.readFileSync(path.join(path.dirname(runtime), "../package.json"), "utf8"),
  );
  assert.equal(runtimePackage.version, "7.0.2");
  const binaries = { base: fs.realpathSync(baseInput), head: fs.realpathSync(headInput) };
  const sourceFile = "crates/vize_canon/src/batch/virtual_specifier_message.rs";
  const source = Object.fromEntries(
    [
      ["base", baseSha],
      ["head", headSha],
    ].map(([side, sha]) => {
      const result = spawnSync("git", ["show", `${sha}:${sourceFile}`], {
        cwd: repository,
        encoding: "utf8",
      });
      assert.equal(result.status, 0, result.stderr);
      return [side, result.stdout];
    }),
  );
  const lexicalAuthority = pinLexer(source.base, source.head);
  writeJson(path.join(output, "lexical-source-authority.json"), lexicalAuthority);
  const provenance = {
    baseSha,
    headSha,
    binaries,
    binaryHashes: Object.fromEntries(
      Object.entries(binaries).map(([side, file]) => [side, sha256(fs.readFileSync(file))]),
    ),
    runtime: {
      path: runtime,
      version: runtimePackage.version,
      sha256: sha256(fs.readFileSync(runtime)),
    },
    node: process.version,
    platform: process.platform,
    arch: process.arch,
    rayonThreads: process.env.RAYON_NUM_THREADS ?? null,
    nativeServers: 1,
    scope:
      "Complete real CLI reports and actual source profile counters. JS-derived ordered native-input token vectors are pinned to unchanged Rust lexical source, not emitted engine traces. Declared literal include roots are captured without claiming transitive native-library closure or recomputing Canon project-key identity. Fresh Cargo targets and warmed direct-runtime CLI timings; wrappers only in separate untimed observations.",
  };
  writeJson(path.join(output, "provenance.json"), provenance);
  assert.notEqual(
    provenance.binaryHashes.base,
    provenance.binaryHashes.head,
    "source roles must have distinct CLI images",
  );
  const capture = captureWrapper(output, runtime);
  writeJson(path.join(output, "capture-wrapper.json"), capture);
  const summaries = [];
  for (const item of cases()) {
    const project = path.join(output, item.name);
    fs.mkdirSync(path.join(project, "src"), { recursive: true });
    for (const [name, text] of Object.entries(item.sources))
      fs.writeFileSync(path.join(project, name), text);
    fs.writeFileSync(path.join(project, "package.json"), '{"private":true,"type":"module"}\n');
    writeJson(path.join(project, "tsconfig.json"), {
      compilerOptions: item.options,
      include: ["src/**/*.ts"],
    });
    const beforeInputs = inventory(project);
    writeJson(path.join(project, "inputs-before.json"), beforeInputs);
    let reference = null;
    const samples = [];
    function run(side, label, profiled = false) {
      const profile = path.join(project, `${label}.profile.json`);
      const native = path.join(project, `${label}.native`);
      if (profiled) {
        fs.mkdirSync(native);
        fs.writeFileSync(capture.location, native);
      }
      const provider = profiled ? capture.wrapper : runtime;
      const argv = [
        "check",
        "--no-config",
        "--tsconfig",
        "tsconfig.json",
        "--servers",
        "1",
        "--quiet",
        "--format",
        "json",
        "--corsa-path",
        provider,
        ...(profiled ? ["--profile-json", profile] : []),
      ];
      const ps = spawnSync("ps", ["-eo", "pid,ppid,pcpu,rss,etime,args"], { encoding: "utf8" });
      assert.equal(ps.status, 0);
      fs.writeFileSync(path.join(project, `${label}.ps.txt`), ps.stdout);
      const started = performance.now();
      const result = spawnSync(binaries[side], argv, {
        cwd: project,
        env: cleanEnvironment(provider),
        timeout: 180000,
        maxBuffer: 64 * 1024 * 1024,
      });
      const elapsedMs = performance.now() - started;
      fs.writeFileSync(path.join(project, `${label}.stdout`), result.stdout ?? "");
      fs.writeFileSync(path.join(project, `${label}.stderr`), result.stderr ?? "");
      writeJson(path.join(project, `${label}.process.json`), {
        side,
        binary: binaries[side],
        argv,
        cwd: project,
        status: result.status,
        signal: result.signal,
        error: result.error ? String(result.error) : null,
        elapsedMs,
        qualifyingTiming: !profiled && !label.includes("warmup"),
      });
      const report = assertPacket(result, reference);
      if (reference === null) {
        reference = result.stdout;
        fs.writeFileSync(path.join(project, "complete-reference.stdout"), reference);
      }
      if (item.errors !== null) assert.equal(report.errorCount, item.errors);
      if (item.name === "large-full") {
        assert.equal(reference.length, 1041941);
        assert.equal(
          sha256(reference),
          "ebaf1af489965e958cf57fac3ce5ca70fc529305a5ea59afc3fe32b8668e3638",
        );
      }
      if (item.name === "large-authored-bounded") {
        assert.equal(reference.length, 50582);
        assert.equal(
          sha256(reference),
          "fe4a19e779f9f79c42a0a5cffb427cb9a9d98c5502524b41b6cede0731db11b8",
        );
      }
      if (item.name === "frozen") {
        assert.equal(reference.length, 861);
        assert.equal(
          sha256(reference),
          "09e8ba0489db8dbea7800f4e04ed5802e5ab78d21b0b276ab7aa5e663447a085",
        );
      }
      const row = {
        side,
        label,
        elapsedMs,
        stdoutSha256: sha256(result.stdout),
        stdoutBytes: result.stdout.length,
      };
      samples.push(row);
      writeJson(path.join(project, "samples.json"), samples);
      if (profiled) {
        const parsed = JSON.parse(fs.readFileSync(profile, "utf8"));
        for (const key of [
          "canon.corsa.cli.parse.decode",
          "canon.corsa.cli.parse.assemble",
          "canon.corsa.cli.parse.restore_paths",
          "canon.corsa.cli.parse.deduplicate",
        ])
          assert.ok(
            parsed.spans.some((span) => span.key === key && span.count === 1),
            key,
          );
        const nativeInputs = nativeVector(native);
        assert.equal(nativeInputs.length, 1, "exactly one native CLI program per fixture");
        writeJson(path.join(project, `${label}.native-vectors.json`), nativeInputs);
        return { counters: counters(parsed), nativeInputs };
      }
      return row;
    }
    for (const side of ["base", "head"]) run(side, `${side}-warmup`);
    const pairs = [];
    for (let pair = 0; pair < 5; pair++) {
      const current = {};
      for (const side of pair % 2 === 0 ? ["base", "head"] : ["head", "base"])
        current[side] = run(side, `${side}-pair-${pair}`);
      pairs.push({ pair, baseMs: current.base.elapsedMs, headMs: current.head.elapsedMs });
    }
    const observed = {};
    for (const side of ["base", "head"]) observed[side] = run(side, `${side}-profile`, true);
    assertCounters(observed.base.counters, observed.head.counters, item.name);
    assert.deepEqual(
      observed.head.nativeInputs,
      observed.base.nativeInputs,
      "complete native argv/packets and derived ordered candidate vectors changed",
    );
    const afterInputs = inventory(project);
    assert.deepEqual(afterInputs, beforeInputs, "author inputs changed");
    writeJson(path.join(project, "inputs-after.json"), afterInputs);
    if (item.name === "frozen") {
      fs.writeFileSync(path.join(project, "src/entry.ts"), "export const fixed: number = 1;\n");
      const repairedInputs = inventory(project);
      writeJson(path.join(project, "repair-inputs.json"), repairedInputs);
      for (const side of ["base", "head"]) {
        const argv = [
          "check",
          "--no-config",
          "--tsconfig",
          "tsconfig.json",
          "--servers",
          "1",
          "--quiet",
          "--format",
          "json",
          "--corsa-path",
          runtime,
        ];
        const result = spawnSync(binaries[side], argv, {
          cwd: project,
          env: cleanEnvironment(runtime),
          timeout: 180000,
          maxBuffer: 64 * 1024 * 1024,
        });
        fs.writeFileSync(path.join(project, `${side}-repaired.stdout`), result.stdout ?? "");
        fs.writeFileSync(path.join(project, `${side}-repaired.stderr`), result.stderr ?? "");
        writeJson(path.join(project, `${side}-repaired.process.json`), {
          argv,
          cwd: project,
          status: result.status,
          signal: result.signal,
          error: result.error ? String(result.error) : null,
          qualifyingTiming: false,
        });
        assert.equal(result.error, undefined);
        assert.equal(result.signal, null);
        assert.equal(result.status, 0);
        assert.deepEqual(result.stderr, Buffer.alloc(0));
        assert.equal(result.stdout.length, 493);
        assert.equal(
          sha256(result.stdout),
          "bd0997ff83bc14f7941a605fcbb09b972cd4fac80e81f673642807a1526f3c82",
        );
        assert.equal(JSON.parse(result.stdout).errorCount, 0);
        assert.deepEqual(inventory(project), repairedInputs);
      }
    }
    summaries.push({
      name: item.name,
      options: item.options,
      fullReportSha256: sha256(reference),
      fullReportBytes: reference.length,
      pairs,
      baseMedianMs: median(pairs.map((p) => p.baseMs)),
      headMedianMs: median(pairs.map((p) => p.headMs)),
      observations: observed,
    });
    writeJson(path.join(output, "results.json"), summaries);
  }
  for (const [side, binary] of Object.entries(binaries))
    assert.equal(sha256(fs.readFileSync(binary)), provenance.binaryHashes[side]);
  assert.equal(sha256(fs.readFileSync(runtime)), provenance.runtime.sha256);
  const lines = [
    "## Complete diagnostic message observations",
    "",
    "Public synthetic and edge-case fixtures; these are not a private consumer workload.",
    "Cargo targets begin absent; CLI timings use per-side warmups and the same physical fixture/runtime. Declared include roots and derived native-input vectors do not claim transitive graph closure or emitted engine traces.",
    "",
    "| Fixture | Base median ms | Head median ms | Complete report bytes |",
    "| --- | ---: | ---: | ---: |",
    ...summaries.map(
      (r) =>
        `| ${r.name} | ${r.baseMedianMs.toFixed(3)} | ${r.headMedianMs.toFixed(3)} | ${r.fullReportBytes} |`,
    ),
  ];
  fs.writeFileSync(path.join(output, "summary.md"), `${lines.join("\n")}\n`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  main(process.argv.slice(2));
