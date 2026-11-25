import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { workflowProgramsDirectory } from "./support/automation-source.ts";

const workflowFile = (entry: string) => {
  const [owner, file] = entry.split("/") as [string, string];
  return path.join(workflowProgramsDirectory(owner), file);
};

function runFilter(tool: string, file: string, input: string, options: string[] = []) {
  const result = spawnSync(tool, [...options, "-f", file], { input, encoding: "utf8" });
  assert.ifError(result.error);
  assert.equal(result.signal, null);
  return { status: result.status, stdout: result.stdout.trimEnd() };
}

const hasAwk = !spawnSync("awk", ["-f", process.platform === "win32" ? "NUL" : "/dev/null"], {
  input: "",
}).error;
const hasJq = !spawnSync("jq", ["--version"]).error;

test(
  "the benchmark reads the same exact package keys and pins as the workspace catalog",
  { skip: !hasAwk },
  () => {
    const file = workflowFile("05-lsp-vs-volar/catalog-version.awk");
    const pin = (text: string, name: string) =>
      runFilter("awk", file, text, ["-v", "package_name=" + name]);
    const source = readFileSync("pnpm-workspace.yaml", "utf8");
    const { catalogs } = parse(source) as { catalogs: Record<string, Record<string, string>> };
    for (const name of ["vue-tsc", "typescript", "@typescript/typescript-linux-x64"]) {
      const expected = Object.values(catalogs).find((catalog) => Object.hasOwn(catalog, name))?.[
        name
      ];
      assert.ok(expected, name);
      assert.deepEqual(pin(source, name), { status: 0, stdout: expected });
    }
    const example = ["    pkg-extra: 4", '    "pkg": "1.2.3" # original', "    pkg: 2.0.0"].join(
      "\n",
    );
    assert.deepEqual(pin(example, "pkg"), { status: 0, stdout: "1.2.3" });
    assert.deepEqual(pin(example, "missing"), { status: 0, stdout: "" });
  },
);

test(
  "process budgets and benchmark reports retain their units, columns and row limits",
  { skip: !hasAwk },
  () => {
    const threads = ".github/actions/check-vue-parity/scripts/sum-thread-counts.awk";
    assert.deepEqual(runFilter("awk", threads, "Threads: 3\nThreads: 7", ["-v", "column=2"]), {
      status: 0,
      stdout: "10",
    });
    assert.deepEqual(runFilter("awk", threads, " 2\n 5\n", ["-v", "column=1"]), {
      status: 0,
      stdout: "7",
    });
    assert.deepEqual(runFilter("awk", threads, "", ["-v", "column=2"]), { status: 0, stdout: "0" });
    assert.deepEqual(
      runFilter(
        "awk",
        workflowFile("03-typecheck-compare/mapper-median-memory-row.awk"),
        "1024 1 2 3\n2048 2.5 1 2.75\n4096 3 4 5\n",
        ["-v", "label=head"],
      ),
      { status: 0, stdout: "| head | 2.0 | 2.50 | 3.75 |" },
    );
    assert.deepEqual(
      runFilter(
        "awk",
        workflowFile("03-typecheck-compare/replay-perf-report-lines.awk"),
        "# perf\n\nfirst\n# skip\nsecond\nthird\n",
        ["-v", "limit=2"],
      ),
      { status: 0, stdout: "first\nsecond" },
    );
    const syscalls = [
      'write(1, "a", 1) = 1',
      'write(1, "b", 4096) = 4096',
      'read(2, "c", 3) = 3',
    ].join("\n");
    assert.deepEqual(
      runFilter("awk", workflowFile("03-typecheck-compare/replay-syscall-io.awk"), syscalls, [
        "-v",
        "side=head",
        "-v",
        "corpus=small",
      ]),
      { status: 0, stdout: "| small | head | write(1) | 2 | 0.0 | 1 |" },
    );
  },
);

test(
  "Alloy qualification rejects an unexpected command, missing model, or counterexample",
  { skip: !hasJq },
  () => {
    const checks = [
      [
        "07-ir-graph/check-directed-graph.jq",
        "Exact",
        ["DisconnectedCycle", "MissedSuccessor", "ParallelCycle"],
      ],
      [
        "07-ir-graph/check-edge-trails.jq",
        "ExactTrails",
        ["DistinctSelfLoops", "ParallelReturn", "SingleEdgeReturnsOnlyOnReuse"],
      ],
      [
        "07-declaration-products/check-immutable-bindings.jq",
        "SameTrace",
        ["LaterWrite", "NonemptyTrace", "RepeatedName"],
      ],
    ] as const;
    for (const [file, theorem, witnesses] of checks) {
      const commands = Object.fromEntries(
        witnesses.map((name) => [name, { type: "run", solution: [{}] }]),
      );
      const original = { ...commands, [theorem]: { type: "check", solution: [] } };
      const check = (model: object) =>
        runFilter("jq", workflowFile(file), JSON.stringify({ commands: model }), ["-e"]).status;
      assert.equal(check(original), 0, file);
      assert.equal(check({ ...original, Extra: {} }), 1, file);
      assert.equal(check({ ...original, [theorem]: { type: "check", solution: [{}] } }), 1, file);
      assert.equal(check({ ...original, [witnesses[0]]: { type: "run", solution: [] } }), 1, file);
    }
  },
);

test("The Cargo selector returns the last matching record", { skip: !hasJq }, () => {
  const artifact = (name: string, executable: string | null) => ({
    reason: "compiler-artifact",
    target: { name },
    executable,
  });
  const records = [
    artifact("vize", "/candidate/vize"),
    artifact("vize", null),
    artifact("foreign", "/foreign"),
  ]
    .map((event) => JSON.stringify(event))
    .join("\n");
  const options = ["-sr", "--arg", "name", "vize"];
  assert.deepEqual(
    runFilter("jq", "tools/ci/lib/scripts/cargo-target-executable.jq", records, options),
    { status: 0, stdout: "null" },
  );
});
