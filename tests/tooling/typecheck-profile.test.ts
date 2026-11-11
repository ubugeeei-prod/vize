import assert from "node:assert/strict";
import { test } from "node:test";
import { gzipSync } from "node:zlib";

import {
  type FunctionTable,
  ehFrameStarts,
  functionAt,
  goFunctions,
  vaddrOf,
} from "../../tools/benchmarks/scripts/typecheck-profile-elf.ts";
import { ProfileFolder } from "../../tools/benchmarks/scripts/typecheck-profile-fold.ts";
import { parseStatCsv } from "../../tools/benchmarks/scripts/typecheck-profile-perf.ts";
import { pprofTop } from "../../tools/benchmarks/scripts/typecheck-profile-pprof.ts";
import {
  type LaneResult,
  functionDiff,
  renderLane,
  renderOverview,
} from "../../tools/benchmarks/scripts/typecheck-profile-report.ts";
import {
  cleanSymbol,
  costKind,
  dsoGroup,
  phaseOf,
  processRole,
} from "../../tools/benchmarks/scripts/typecheck-profile-rules.ts";
import { corpusSubmodules, selectIds } from "../../tools/benchmarks/scripts/typecheck-profile.ts";

const TSGO = "/h/node_modules/@typescript/typescript-linux-x64/lib/tsc";
const TSRS = "/h/node_modules/@tsrs/linux-x64-gnu/tsrs";
const VIZE = "/b/npm/native/vize-vitrine.linux-x64-gnu.node";

void test("Go pclntab functab and eh_frame_hdr tables name stripped functions", () => {
  // pcHeader (magic, pad, minLC, ptrSize, nfunc, nfiles, textStart, funcnameOffset, cu, file, pctab, pcOffset)
  const header = Buffer.alloc(8 + 8 * 8);
  header.writeUInt32LE(0xfffffff1, 0);
  header.writeUInt8(1, 6);
  header.writeUInt8(8, 7);
  header.writeBigUInt64LE(2n, 8);
  const names = Buffer.from("main.main\0internal/checker.(*Checker).checkSourceFile\0");
  const funcnameOffset = header.length;
  const pcOffset = funcnameOffset + names.length;
  header.writeBigUInt64LE(BigInt(funcnameOffset), 8 + 3 * 8);
  header.writeBigUInt64LE(BigInt(pcOffset), 8 + 7 * 8);
  const functab = Buffer.alloc(3 * 8 + 2 * 8);
  // Entries: (entryOff, funcOff) x2 + end; _func records follow at funcOff.
  functab.writeUInt32LE(0x0, 0);
  functab.writeUInt32LE(24, 4);
  functab.writeUInt32LE(0x100, 8);
  functab.writeUInt32LE(32, 12);
  functab.writeUInt32LE(0x200, 16);
  functab.writeInt32LE(0, 24 + 4);
  functab.writeInt32LE(10, 32 + 4);
  const go = goFunctions(Buffer.concat([header, names, functab]), 0x401000);
  assert.deepEqual(go.starts, [0x401000, 0x401100]);
  assert.deepEqual(go.names, ["main.main", "internal/checker.(*Checker).checkSourceFile"]);

  const hdr = Buffer.alloc(12 + 2 * 8);
  hdr.set([1, 0x1b, 0x03, 0x3b]);
  hdr.writeUInt32LE(2, 8);
  hdr.writeInt32LE(-0x1000, 12);
  hdr.writeInt32LE(-0x800, 20);
  assert.deepEqual(ehFrameStarts(hdr, 0x5000), [0x4000, 0x4800]);

  const table: FunctionTable = {
    kind: "eh_frame_hdr",
    starts: Float64Array.from([0x4000, 0x4800]),
    names: ["fn@4000", "fn@4800"],
    loads: [{ offset: 0x1000, vaddr: 0x3000, filesz: 0x4000 }],
  };
  assert.equal(functionAt(table, 0x3fff), null);
  assert.equal(functionAt(table, 0x47ff), "fn@4000");
  assert.equal(functionAt(table, 0x9000), "fn@4800");
  assert.equal(vaddrOf(table, 0x1800), 0x3800);
  assert.equal(vaddrOf(table, 0x10), null);
});

/** perf script text of a tsc lane: node launcher -> execve tsgo, a forked node mapper, a tsrs PIE. */
const SCRIPT = [
  "node 100/100 PERF_RECORD_COMM exec: node:100/100",
  "node 100/100 PERF_RECORD_MMAP2 100/100: [0x400000(0x2000000) @ 0 08:01 7 0]: r-xp /usr/bin/node",
  "node 100/100 10.000000:     1000000 cpu-clock:u: ",
  "\t          401000 uv_run (/usr/bin/node)",
  "\t          400100 _start (/usr/bin/node)",
  "",
  "tsc 100/100 PERF_RECORD_COMM exec: tsc:100/100",
  `tsc 100/100 PERF_RECORD_MMAP2 100/100: [0x400000(0x900000) @ 0 08:01 9 0]: r-xp ${TSGO}`,
  "tsc 100/105 PERF_RECORD_FORK(105:105):(100:100)",
  "node 105/105 PERF_RECORD_COMM exec: node:105/105",
  `node 105/107 PERF_RECORD_MMAP2 105/107: [0x7f0000000000(0x100000) @ 0x2000 08:01 11 0]: r-xp ${VIZE}`,
  "tsc 100/101 10.005000:     2000000 cpu-clock:u: ",
  `\t          401150 [unknown] (${TSGO})`,
  `\t          401050 [unknown] (${TSGO})`,
  `\t          401010 [unknown] (${TSGO})`,
  "",
  "tsc 100/102 10.012000:     1000000 cpu-clock:u: ",
  "\tffffffff81000010 futex_wait_queue ([kernel.kallsyms])",
  "\tffffffff81000020 __x64_sys_futex ([kernel.kallsyms])",
  "\tffffffff81000030 entry_SYSCALL_64_after_hwframe ([kernel.kallsyms])",
  `\t          401200 [unknown] (${TSGO})`,
  "",
  "node 105/107 10.025000:     3000000 cpu-clock:u: ",
  "\t    7f00000fe000 malloc (/usr/lib/x86_64-linux-gnu/libc.so.6)",
  `\t    7f0000001000 vize::commands::content_mapper::dispatch::wire::encode_transform::h0123456789abcdef (${VIZE})`,
  `\t    7f0000000800 vize::commands::content_mapper::dispatch::worker (${VIZE})`,
  "\t    7f00000ff000 start_thread (/usr/lib/x86_64-linux-gnu/libc.so.6)",
  "",
  "tsrs 200/200 PERF_RECORD_COMM exec: tsrs:200/200",
  // The loader's whole-image reservation comes first; the code segment is mapped over it.
  `tsrs 200/200 PERF_RECORD_MMAP2 200/200: [0x555555400000(0x2800000) @ 0 08:01 13 0]: r--p ${TSRS}`,
  `tsrs 200/200 PERF_RECORD_MMAP2 200/200: [0x555557a00000(0x100000) @ 0x200000 08:01 13 0]: r-xp ${TSRS}`,
  "tsrs 200/200 10.031000:     1000000 cpu-clock:u: ",
  `\t    555557a04010 [unknown] (${TSRS})`,
  "",
];

const tables = (path: string): FunctionTable => {
  if (path === TSGO) {
    return {
      kind: "gopclntab",
      starts: Float64Array.from([0x401000, 0x401040, 0x401100, 0x401200]),
      names: [
        "runtime.goexit",
        "github.com/microsoft/TypeScript/tsc/internal/checker.(*Checker).checkSourceFile",
        "runtime.mallocgc",
        "runtime.futex",
      ],
      loads: [{ offset: 0, vaddr: 0x400000, filesz: 0x900000 }],
    };
  }
  if (path === TSRS) {
    return {
      kind: "eh_frame_hdr",
      starts: Float64Array.from([0x2600000, 0x2604000]),
      names: ["fn@2600000", "fn@2604000"],
      loads: [{ offset: 0x200000, vaddr: 0x2600000, filesz: 0x100000 }],
    };
  }
  return { kind: "symtab", starts: new Float64Array(), names: [], loads: [] };
};

void test("the folder splits a lane tree by process, DSO, cost kind, phase and function", () => {
  const folder = new ProfileFolder("host", tables);
  for (const line of SCRIPT) folder.line(line);
  const profile = folder.finish(0);
  assert.equal(profile.samples, 5);
  assert.equal(profile.period, 8_000_000);
  assert.equal(profile.event, "cpu-clock:u");
  assert.deepEqual(profile.roles, {
    "host launcher (node)": 1_000_000,
    tsgo: 3_000_000,
    "vize mapper (node)": 3_000_000,
    tsrs: 1_000_000,
  });
  assert.deepEqual(profile.dsos, {
    node: 1_000_000,
    tsgo: 2_000_000,
    kernel: 1_000_000,
    libc: 3_000_000,
    tsrs: 1_000_000,
  });
  assert.equal(profile.kinds["Go: alloc"], 2_000_000);
  assert.equal(profile.kinds["kernel: futex"], 1_000_000);
  assert.equal(profile.kinds["libc: malloc/free"], 3_000_000);
  assert.deepEqual(profile.syscalls, { futex: 1_000_000 });
  assert.equal(profile.phases.tsgo?.["host: check"]?.["Go: alloc"], 2_000_000);
  assert.equal(
    profile.phases["vize mapper (node)"]?.["vize: reply encode"]?.["libc: malloc/free"],
    3_000_000,
  );
  const fn = (name: string) => profile.functions.find((cost) => cost.name === name);
  assert.deepEqual(fn("runtime.mallocgc"), {
    role: "tsgo",
    dso: "tsgo",
    name: "runtime.mallocgc",
    self: 2_000_000,
    total: 2_000_000,
  });
  assert.equal(fn("internal/checker.(*Checker).checkSourceFile")?.total, 2_000_000);
  assert.equal(
    fn("vize::commands::content_mapper::dispatch::wire::encode_transform")?.total,
    3_000_000,
  );
  // PIE: 0x555557a04010 - 0x555557a00000 + 0x200000 = file 0x204010 -> vaddr 0x2604010;
  // the r--p whole-image reservation would have read it as file 0x2604010.
  assert.equal(fn("fn@2604000")?.self, 1_000_000);
  // The futex sample's user chain stops in runtime.futex, not at an entry frame.
  assert.equal(profile.quality.tsgo?.complete, 2_000_000);
  assert.deepEqual(profile.quality.tsgo?.roots, [["runtime.futex", 1_000_000]]);
  assert.equal(profile.symbolizers[TSGO], "gopclntab");
  assert.deepEqual(profile.symbolization, { resolved: { mapping: 5 }, missed: 0, examples: [] });
  // 10 ms buckets from the first sample.
  assert.deepEqual(profile.timeline.roles.tsgo, [2_000_000, 1_000_000]);
  assert.deepEqual(profile.timeline.roles["vize mapper (node)"], [0, 0, 3_000_000]);
  assert.deepEqual(profile.timeline.phases["host: check"], [2_000_000]);
  const folded = [...folder.folded()];
  assert.equal(
    folded[0],
    "vize mapper (node);start_thread;vize::commands::content_mapper::dispatch::worker;vize::commands::content_mapper::dispatch::wire::encode_transform;malloc 3000000",
  );
  assert.ok(
    folded.includes(
      "tsgo;runtime.futex;entry_SYSCALL_64_after_hwframe_[k];__x64_sys_futex_[k];futex_wait_queue_[k] 1000000",
    ),
  );
});

void test("rules name DSOs, roles, cost kinds and phases", () => {
  assert.equal(cleanSymbol("vize_canon::emit::h0123456789abcdef"), "vize_canon::emit");
  assert.equal(
    cleanSymbol("github.com/microsoft/TypeScript/tsc/internal/parser.Parse"),
    "internal/parser.Parse",
  );
  assert.equal(dsoGroup("[kernel.kallsyms]"), "kernel");
  assert.equal(dsoGroup("/tmp/perf-123.map"), "node JIT");
  assert.equal(dsoGroup("/lib/x86_64-linux-gnu/libc.so.6"), "libc");
  assert.equal(dsoGroup(TSGO), "tsgo");
  assert.equal(dsoGroup("/x/bin/tsc"), "tsc");
  assert.equal(processRole("tsc", false, "check"), "tsgo (Corsa)");
  assert.equal(processRole("node", true, "check"), "vize check (node)");
  assert.equal(costKind(["__memmove_avx_unaligned_erms"], ["libc"]), "libc: memcpy/memmove");
  assert.equal(
    costKind(
      ["clear_page_erms", "handle_mm_fault", "exc_page_fault", "asm_exc_page_fault"],
      ["kernel", "kernel", "kernel", "kernel"],
    ),
    "kernel: page fault",
  );
  assert.equal(costKind(["runtime.scanobject", "runtime.gcDrain"], ["tsgo", "tsgo"]), "Go: GC");
  assert.equal(costKind(["vize_canon::x"], ["vize native"]), "code");
  assert.equal(
    phaseOf(["encoding/json/v2.Unmarshal", "internal/contentmapper.decodeTransformResult"], "tsgo"),
    "host: mapper reply decode",
  );
  assert.equal(
    phaseOf(
      ["internal/parser.(*Parser).parseStatement", "internal/contentmapper.TransformAndParse"],
      "tsgo",
    ),
    "host: parse mapped SFC",
  );
  assert.equal(phaseOf(["internal/binder.(*Binder).bind"], "tsgo"), "host: bind");
  assert.equal(
    phaseOf(
      ["vize_canon::batch::x", "vize::commands::content_mapper::dispatch::transform_frame"],
      "vize mapper (node)",
    ),
    "vize: virtual TS generation",
  );
  // Parsing inside generation is its own phase, wherever the generator runs.
  assert.equal(
    phaseOf(
      [
        "<oxc_parser::ParserImpl<oxc_parser::config::NoTokensParserConfig>>::parse_program",
        "vize_canon::batch::virtual_project::vue_codegen::generate_vue_virtual_ts",
        "vize_canon::corsa_bridge::session::check",
      ],
      "vize check (node)",
    ),
    "vize: script parse (oxc)",
  );
  assert.equal(
    phaseOf(
      ["<vize_canon::batch::executor::CorsaExecutor>::check_with_servers"],
      "vize check (node)",
    ),
    "vize: Corsa client",
  );
  assert.equal(phaseOf(["uv_run"], "host launcher (node)"), "host launcher (node): other");
});

void test("perf stat CSV, corpus submodules and id selection", () => {
  const stat = parseStatCsv(
    [
      "# started on Wed Nov 11",
      "",
      "1234.50,msec,task-clock:u,1234500000,100.00,3.21,CPUs utilized",
      "<not supported>,,cycles:u,0,100.00,,",
      "42,,context-switches,1234500000,100.00,34.02,/sec",
      "7,,cpu_core/cache-misses/,1,100.00,,",
      "1456903608,,duration_time,1,100.00,120.123,M/sec",
    ].join("\n"),
  );
  assert.deepEqual(stat["task-clock"], { value: 1234.5, unit: "msec" });
  assert.deepEqual(stat.cycles, { value: null, unit: "" });
  assert.equal(stat["context-switches"]?.value, 42);
  assert.equal(stat["cache-misses"]?.value, 7);
  // perf 6.17 writes tool time events as unitless nanoseconds.
  assert.deepEqual(stat.duration_time, { value: 1456.903608, unit: "msec" });
  const gitmodules =
    '[submodule "a"]\n\tpath = tests/_fixtures/_git/misskey\n[submodule "b"]\n\tpath = tests/_fixtures/_git/n8n\n';
  assert.deepEqual(
    corpusSubmodules(
      [{ id: "n8n", source: "tests/_fixtures/_git/n8n/packages/frontend/editor-ui" }],
      gitmodules,
    ),
    ["tests/_fixtures/_git/n8n"],
  );
  assert.throws(
    () => corpusSubmodules([{ id: "x", source: "tests/x" }], gitmodules),
    /no submodule/u,
  );
  const all = [{ id: "misskey" }, { id: "n8n" }];
  assert.deepEqual(selectIds(all, "n8n", "corpus"), [{ id: "n8n" }]);
  assert.deepEqual(selectIds(all, "", "corpus"), all);
  assert.throws(() => selectIds(all, "nuxt", "corpus"), /unknown corpus: nuxt/u);
});

const varint = (value: number): number[] => {
  const out: number[] = [];
  let rest = value;
  while (rest >= 0x80) {
    out.push((rest & 0x7f) | 0x80);
    rest = Math.floor(rest / 128);
  }
  out.push(rest);
  return out;
};
const field = (number: number, payload: number | number[] | Buffer): number[] =>
  typeof payload === "number"
    ? [...varint(number << 3), ...varint(payload)]
    : [...varint((number << 3) | 2), ...varint(payload.length), ...payload];
const message = (...parts: number[][]) => parts.flat();

void test("pprof profiles decode to flat and cumulative time per function", () => {
  const strings = ["", "samples", "count", "cpu", "nanoseconds", "main.leaf", "main.caller"];
  const profile = Buffer.from(
    message(
      field(1, message(field(1, 1), field(2, 2))),
      field(1, message(field(1, 3), field(2, 4))),
      // Leaf-first locations; values packed.
      field(
        2,
        message(field(1, [...varint(1), ...varint(2)]), field(2, [...varint(1), ...varint(30)])),
      ),
      field(2, message(field(1, [...varint(2)]), field(2, [...varint(1), ...varint(10)]))),
      field(4, message(field(1, 1), field(4, message(field(1, 1))))),
      field(4, message(field(1, 2), field(4, message(field(1, 2))))),
      field(5, message(field(1, 1), field(2, 5))),
      field(5, message(field(1, 2), field(2, 6))),
      ...strings.map((text) => field(6, Buffer.from(text))),
    ),
  );
  const top = pprofTop(gzipSync(profile));
  assert.equal(top.sampleType, "cpu");
  assert.equal(top.unit, "nanoseconds");
  assert.equal(top.total, 40);
  assert.deepEqual(
    top.functions.sort((a, b) => a.name.localeCompare(b.name)),
    [
      { name: "main.caller", flat: 10, cum: 40 },
      { name: "main.leaf", flat: 30, cum: 30 },
    ],
  );
});

void test("the report renders a lane against its base and ranks function moves", () => {
  const lane = (side: "base" | "head", malloc: number): LaneResult => {
    const folder = new ProfileFolder("host", tables);
    for (const line of SCRIPT) {
      folder.line(line.replace("3000000 cpu-clock", `${malloc} cpu-clock`));
    }
    return {
      corpus: "n8n",
      lane: "tsc",
      label: "tsc (tsgo) + Vize mapper",
      side,
      sha: side.repeat(10),
      callGraph: "fp",
      frequency: 999,
      wallMs: 1500,
      diagnostics: 3,
      counters: { "task-clock": { value: 8, unit: "msec" }, cycles: { value: null, unit: "" } },
      hostStats: [["Check time", "1.20s"]],
      profile: folder.finish(0),
      pprof: null,
      notes: [],
    };
  };
  const base = lane("base", 3_000_000);
  const head = lane("head", 1_000_000);
  const moves = functionDiff(base, head, "self", 1);
  assert.equal(moves[0]?.cost.name, "malloc");
  assert.equal(moves[0]?.baseMs, 3);
  assert.equal(moves[0]?.headMs, 1);
  const text = renderLane(head, base);
  assert.match(text, /### n8n: tsc \(tsgo\) \+ Vize mapper/u);
  assert.match(text, /\| vize mapper \(node\) \| 3 ms \| 1 ms \| -2 ms \(-66\.7%\) \| 16\.7% \|/u);
  assert.match(text, /\| Check time \| 1\.20s \| 1\.20s \|/u);
  assert.match(text, /Largest self-time moves/u);
  // Busy cores: 1 ms of the mapper in the 20–30 ms slice is 0.1 cores.
  assert.match(text, /\| 20–30 \| 0\.0 \| 0\.0 \| 0\.1 \| 0\.0 \| vize: reply encode 0\.1 \|/u);
  assert.match(
    renderOverview([base, head]),
    /\| n8n \| tsc \(tsgo\) \+ Vize mapper \| 6 ms \| -2 ms \(-25\.0%\)/u,
  );
});
