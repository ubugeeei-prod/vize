import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  verifySourceSdkPair,
  verifyStrictDebug,
  prepareStrictOutput,
  verifyStrictSourceInputs,
  strictPairSources,
  strictPairPaths,
  type SdkScanner,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-strict-pair.ts";
import { coldScalePacketView } from "../../tools/benchmarks/scripts/cold-typechecker-scale-packet.ts";
import { coldScaleAffinity } from "../../tools/benchmarks/scripts/cold-typechecker-scale-affinity.ts";
import {
  nearestRank,
  linuxPeakRss,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-paired-stats.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));

test("only the frozen fifteen-path input and whole SDK4 Cargo identities are admitted", () => {
  const read = (file: string) =>
    execFileSync("git", ["show", `${strictPairSources.base}:${file}`], { cwd: root });
  const manifest = read("Cargo.toml"),
    lock = read("Cargo.lock");
  const make = (): Parameters<typeof verifyStrictSourceInputs>[0] => ({
    sources: { ...strictPairSources },
    changedPaths: [...strictPairPaths],
    cargo: {
      base: { "Cargo.toml": Buffer.from(manifest), "Cargo.lock": Buffer.from(lock) },
      head: { "Cargo.toml": Buffer.from(manifest), "Cargo.lock": Buffer.from(lock) },
    },
  });
  assert.deepEqual(verifyStrictSourceInputs(make()), {
    sdkVersion: "2.0.0-beta.4",
    files: [
      {
        file: "Cargo.toml",
        bytes: 13168,
        sha256: "6400cafce2abb1c16bb3c43dfc6498d0a0c695c88849bd5a6bdd4063b91377de",
      },
      {
        file: "Cargo.lock",
        bytes: 128818,
        sha256: "e9a8659e79a919541b6cf29340886dabad787a18fa3051bd77873fc3a76194a4",
      },
    ],
  });
  for (const control of [
    "base",
    "head",
    "extra",
    "missing",
    "order",
    "duplicate",
    "manifest-beta3",
    "lock-beta3",
    "same-extra-lock-byte",
  ]) {
    const input = make();
    if (control === "base" || control === "head") input.sources[control] = "a".repeat(40);
    else if (control === "extra") input.changedPaths.push("crates/vize/src/unreviewed.rs");
    else if (control === "missing") input.changedPaths.pop();
    else if (control === "order") input.changedPaths.reverse();
    else if (control === "duplicate") input.changedPaths.push(input.changedPaths[0]);
    else
      for (const role of ["base", "head"] as const) {
        if (control === "same-extra-lock-byte")
          input.cargo[role]["Cargo.lock"] = Buffer.concat([lock, Buffer.from("\n")]);
        else {
          const file = control === "manifest-beta3" ? "Cargo.toml" : "Cargo.lock";
          const before = input.cargo[role][file].toString("utf8");
          const changed = before.replaceAll("2.0.0-beta.4", "2.0.0-beta.3");
          assert.notEqual(changed, before);
          input.cargo[role][file] = Buffer.from(changed);
        }
      }
    assert.throws(() => verifyStrictSourceInputs(input), control);
  }
});

test("a fresh owned output exists before receipts and any reused attempt is refused", () => {
  const parent = fs.mkdtempSync(path.join(os.tmpdir(), "vize-paired-output-"));
  try {
    const output = path.join(parent, "observations");
    prepareStrictOutput(output);
    fs.writeFileSync(path.join(output, "source-closure.json"), "{}\n");
    assert.throws(() => prepareStrictOutput(output));
    assert.equal(fs.readFileSync(path.join(output, "source-closure.json"), "utf8"), "{}\n");
    assert.throws(() => prepareStrictOutput("relative"));
  } finally {
    fs.rmSync(parent, { recursive: true, force: true });
  }
});
test("whole source SDK10 and all thirteen local edges refuse a changed provider declaration", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-sdk-pair-"));
  try {
    const base = path.join(temporary, "base"),
      head = path.join(temporary, "head");
    const typescript = createRequire(path.join(root, "npm/cli/package.json"))(
      "typescript",
    ) as SdkScanner;
    const original = verifySourceSdkPair(root, root, typescript);
    for (const role of [base, head])
      for (const file of ["package.json", ...original.declarations.map((row) => row.file)]) {
        const target = path.join(role, "npm/cli", file);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.copyFileSync(path.join(root, "npm/cli", file), target);
      }
    assert.deepEqual(verifySourceSdkPair(base, head, typescript), original);
    for (const file of ["virtual-ts-slots.d.ts", "package.json"]) {
      const target = path.join(head, "npm/cli", file),
        before = fs.readFileSync(target);
      fs.appendFileSync(target, "\n");
      assert.throws(() => verifySourceSdkPair(base, head, typescript));
      fs.writeFileSync(target, before);
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

test("debug comparison uses source absolute-byte order and refuses extra output or programs", () => {
  const cwd = path.resolve("/owned/project/apps/web");
  const files = [
    { file: "z.ts", virtualTs: "Z" },
    { file: "../../a.ts", virtualTs: "A" },
  ];
  const packet = { programs: [{}], files };
  const debug = Buffer.from(
    `\n=== __vize_helpers.d.ts ===\nhelper\n` +
      `\n=== ${path.resolve(cwd, files[1].file)} ===\nA\n` +
      `\n=== ${path.resolve(cwd, files[0].file)} ===\nZ\n`,
  );
  verifyStrictDebug(debug, packet, cwd);
  for (const changed of [
    Buffer.concat([debug, Buffer.from("extra")]),
    Buffer.from(debug.toString().replace("\nZ\n", "\nX\n")),
  ])
    assert.throws(() => verifyStrictDebug(changed, packet, cwd));
  assert.throws(() => verifyStrictDebug(debug, { ...packet, programs: [{}, {}] }, cwd));
});

test("timed identity projection changes only declared filenames and retains unknown fields", () => {
  const corpus = {
    root: "/owned/project",
    app: "/owned/project/apps/web",
    entries: [{ path: "apps/web/a.ts" }],
  };
  const packet = {
    fileCount: 1,
    files: [{ file: "a.ts", diagnostics: [], extra: { owned: true } }],
    extra: 3,
  };
  assert.deepEqual(coldScalePacketView(packet, corpus), {
    ...packet,
    files: [{ ...packet.files[0], file: "apps/web/a.ts" }],
  });
  for (const file of ["b.ts", "../../../outside.ts"])
    assert.throws(() =>
      coldScalePacketView({ ...packet, files: [{ ...packet.files[0], file }] }, corpus),
    );
  assert.throws(() =>
    coldScalePacketView({ ...packet, files: [...packet.files, ...packet.files] }, corpus),
  );
});

test("paired dispatch is isolated and the complete full-default job remains byte-identical", () => {
  const file = ".github/workflows/cold-typechecker-scale.yml";
  const before = execFileSync("git", ["show", "a193c99c824b54fbe7e900bb88a19f549cd25ea7:" + file], {
    cwd: root,
  }).toString();
  const after = fs.readFileSync(path.join(root, file), "utf8");
  const condition = "    if: github.event_name != 'workflow_dispatch' || inputs.mode == 'full'\n";
  assert.equal(after.split(condition).length, 2);
  assert.equal(
    after.slice(after.indexOf("  profile:\n")).replace(condition, ""),
    before.slice(before.indexOf("  profile:\n")),
  );
  assert(after.includes("options: [full, paired-comparison]\n        default: full"));
  assert(
    after.includes(
      "if: github.event_name == 'workflow_dispatch' && inputs.mode == 'paired-comparison'",
    ),
  );
  assert(after.includes("inputs.mode == 'paired-comparison' && '-paired-comparison' || ''"));
  assert(after.includes("npm ci --ignore-scripts --no-audit --no-fund"));
  assert(after.includes('[[ "$BASE_SHA" == ec8474e0ea9ed05062542c57c857f36b9467ac15 ]]'));
  assert(after.includes('[[ "$HEAD_SHA" == d2579c761944c5a1480f74443188bb4ba10c3913 ]]'));
  const fetch =
    '          git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" "$BASE_SHA" "$HEAD_SHA" a193c99c824b54fbe7e900bb88a19f549cd25ea7';
  const pure =
    "          node --test tests/tooling/cold-typechecker-scale-native-forward.test.ts tests/tooling/cold-typechecker-scale-paired.test.ts tests/tooling/cold-typechecker-scale-strict-pair.test.ts";
  const verifyOracleFetch = (workflow: string) => {
    const job = workflow.slice(
      workflow.indexOf("  paired-comparison:\n"),
      workflow.indexOf("  profile:\n"),
    );
    assert.deepEqual(
      job.split("\n").filter((line) => line.trimStart().startsWith("git fetch ")),
      [fetch],
    );
    assert.deepEqual(
      job.split("\n").filter((line) => line.trimStart().startsWith("node --test ")),
      [pure],
    );
    assert(
      job.indexOf(fetch) < job.indexOf(pure),
      "fetch the unchanged historical oracle before its test",
    );
  };
  verifyOracleFetch(after);
  for (const changed of [
    after.replace(fetch, fetch.replace(" a193c99c824b54fbe7e900bb88a19f549cd25ea7", "")),
    after.replace(fetch, fetch.replace("a193c99c824b54fbe7e900bb88a19f549cd25ea7", "a".repeat(40))),
    after.replace(fetch + "\n", "").replace(pure, pure + "\n" + fetch),
  ])
    assert.throws(() => verifyOracleFetch(changed));
});

test("affinity and RSS controls retain sparse CPU sets and explicit GNU-time scope", () => {
  assert.equal(coldScaleAffinity("Cpus_allowed_list:\t16-19,32-39\n"), "16,17,18,19,32,33,34,35");
  for (const value of ["", "Cpus_allowed_list:\t9-3\n", "Cpus_allowed_list:\t1,1\n"])
    assert.throws(() => coldScaleAffinity(value));
  assert.equal(nearestRank([80, 10, 70, 20, 60, 30, 50, 40], 0.95), 80);
  assert.equal(linuxPeakRss("Maximum resident set size (kbytes): 1234\n"), 1263616);
  assert.throws(() => linuxPeakRss(""));
  assert.throws(() =>
    linuxPeakRss("Maximum resident set size (kbytes): 1\nMaximum resident set size (kbytes): 2\n"),
  );
});
