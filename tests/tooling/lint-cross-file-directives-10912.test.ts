import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { inflateSync } from "node:zlib";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const root = path.resolve(import.meta.dirname, "../..");
const fixture = path.join(root, "tests/_fixtures/differential/linter/cross-file-directives-10912");
const sha256 = (data: string | Buffer) => createHash("sha256").update(data).digest("hex");
const read = (file: string) => JSON.parse(fs.readFileSync(path.join(fixture, file), "utf8"));
const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value));

void test("cross-file directives share authored ranges, actual suppression, unused accounting and normal rule packets", () => {
  const corpus = read("source-cases.json");
  assert.equal(corpus.cases.length, 25);
  const build = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
  );
  validateBuildReceipt(receipt, build);
  const phase = process.env.VIZE_CF_DIRECTIVES_EXPECTED_PHASE ?? "after";
  assert.ok(phase === "before" || phase === "after");
  if (phase === "before") {
    assert.equal(process.env.GITHUB_ACTIONS, "true");
    assert.equal(
      build.sourceRevision,
      "5803734f02597be28a52cc19c15014bc5816b413",
      "before mode belongs only to the immutable combined baseline",
    );
  }
  const versionPacket = spawnSync(path.join(root, build.binaryPath), ["--version"], {
    encoding: "utf8",
    timeout: 10_000,
  });
  assert.equal(versionPacket.error, undefined);
  assert.equal(versionPacket.status, 0);
  assert.equal(versionPacket.stderr, "");
  assert.equal(versionPacket.stdout.trim(), build.cliVersion);
  const temporary = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-cf-directives-10912-")),
  );
  const artifact = path.join(
    root,
    "target/differential/lint-cross-file-directives-10912-" + phase + ".json",
  );
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const runs: any[] = [];
  const evidence = {
    schema: "vize.cross-file-directives-source-cli",
    phase,
    build,
    receipt,
    versionPacket,
    corpusSha256: sha256(fs.readFileSync(path.join(fixture, "source-cases.json"))),
    runs,
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const write = (files: Record<string, string>) => {
    fs.rmSync(path.join(temporary, "src"), { recursive: true, force: true });
    for (const [file, source] of Object.entries(files)) {
      fs.mkdirSync(path.dirname(path.join(temporary, file)), { recursive: true });
      fs.writeFileSync(path.join(temporary, file), source);
    }
  };
  const run = (
    id: string,
    route: string,
    files: Record<string, string>,
    args: string[],
    expected: unknown,
    expectedStatus: number,
  ) => {
    for (let repetition = 0; repetition < 2; repetition++) {
      const result = spawnSync(path.join(root, build.binaryPath), args, {
        cwd: temporary,
        encoding: "utf8",
        timeout: 30_000,
        maxBuffer: 4 * 1024 * 1024,
        env: { ...process.env, NO_COLOR: "1" },
      });
      let packet: unknown;
      try {
        packet = JSON.parse(result.stdout);
      } catch {
        packet = { invalidJson: result.stdout };
      }
      const config = path.join(temporary, "vite.config.mjs");
      runs.push({
        id,
        route,
        repetition,
        args,
        files,
        sourceHashes: Object.fromEntries(
          Object.entries(files).map(([file, source]) => [file, sha256(source)]),
        ),
        config: fs.existsSync(config) ? fs.readFileSync(config, "utf8") : null,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdout: result.stdout,
        stderr: result.stderr,
        packet,
        expected,
        expectedStatus,
      });
      persist();
      for (const [file, source] of Object.entries(files))
        assert.equal(fs.readFileSync(path.join(temporary, file), "utf8"), source, id);
    }
  };
  const counts = (packet: any[]) =>
    packet.map((file) => ({
      ...file,
      errorCount: file.messages.filter((message: any) => message.severity === 2).length,
      warningCount: file.messages.filter((message: any) => message.severity === 1).length,
    }));
  try {
    const archived = new Map<number, any>();
    for (const frozen of [
      {
        run: 40583916685,
        file: "public-0.602.0-before-102.json.zlib.base64.txt",
        sha256: "7703f497e2bebf2f518608a1ad9d136511855df1e21cb3ec25f462166134f558",
        packets: 102,
      },
      {
        run: 40586808222,
        file: "public-0.602.0-controls-before-60.json.zlib.base64.txt",
        sha256: "7136af79cf4f9eba9f9252443016892005a4c19197a88f7af3cac3636cb49ca9",
        packets: 60,
      },
    ]) {
      const raw = inflateSync(
        Buffer.from(fs.readFileSync(path.join(fixture, frozen.file), "utf8").trim(), "base64"),
      );
      assert.equal(sha256(raw), frozen.sha256);
      assert.equal(JSON.parse(raw.toString()).observations.length, frozen.packets);
      archived.set(frozen.run, JSON.parse(raw.toString()));
    }
    for (const original of read("public-before-cases.json.txt").originalSources) {
      const bytes = fs.readFileSync(path.join(fixture, "original", original.path + ".txt"));
      assert.equal(bytes.length, original.bytes);
      assert.equal(sha256(bytes), original.sha256);
      assert.equal(corpus.cases[0].files[original.path], bytes.toString());
    }
    assert.equal(
      sha256(fs.readFileSync(path.join(fixture, "original-issue.md"))),
      corpus.originalBodySha256,
    );
    assert.equal(
      sha256(fs.readFileSync(path.join(fixture, "original-reported.txt"))),
      corpus.originalReportedSha256,
    );
    for (const entry of corpus.cases) {
      if (entry.beforeEvidence.run) {
        const raw = archived.get(entry.beforeEvidence.run);
        assert.ok(raw);
        const before = raw.observations.find(
          (row: any) =>
            row.id === (entry.beforeCaseId ?? entry.id) &&
            row.route === "cli-json" &&
            row.repetition === 0,
        );
        assert.ok(before, entry.id + ": archived whole before packet is required");
        assert.deepEqual(
          before.sourceFiles,
          entry.files,
          entry.id + ": original archived source bytes",
        );
        assert.deepEqual(
          before.packet,
          entry.beforePacket,
          entry.id + ": original archived whole packet",
        );
      }
      write(entry.files);
      const flags = [
        ...(entry.crossFile ? ["--cross-file"] : []),
        ...(entry.reportUnused ? ["--report-unused-disable-directives"] : []),
      ];
      const args = [
        "lint",
        "-f",
        "json",
        "--no-config",
        "--locale",
        "en",
        ...flags,
        "src/**/*.vue",
      ];
      const expected = phase === "before" ? entry.beforePacket : entry.afterPacket;
      const expectedStatus =
        phase === "before"
          ? Number(entry.beforePacket.some((file: any) => file.errorCount > 0))
          : entry.afterStatus;
      run(entry.id, "flag-policy", entry.files, args, expected, expectedStatus);
      if (entry.id === "original-issue" || entry.id === "original-unused-requested") {
        run(
          entry.id,
          "zero-warning-budget",
          entry.files,
          [...args, "--max-warnings", "0"],
          expected,
          expectedStatus,
        );
      }
    }
    const stale = corpus.cases.find(
      (entry: any) => entry.id === "literal-repair-comments-retained",
    );
    assert.ok(stale);
    const base = clone(stale.afterPacket);
    const ordinary = clone(stale.beforePacket);
    // Preserve the unrelated ordinary prop warning, including every range/help field.
    assert.equal(ordinary[0].messages[0].ruleId, "vue/no-unused-properties");
    const withoutUnused = (file: string) => {
      const packet = clone(base);
      const added = stale.wholeChanges.find(
        (change: any) => change.kind === "add-whole-unused-directive" && change.file === file,
      ).after;
      const target = packet.find((row: any) => row.file === file);
      target.messages = target.messages.filter(
        (message: any) => JSON.stringify(message) !== JSON.stringify(added),
      );
      return counts(packet);
    };
    const entries = (setting: boolean) => [
      { files: ["src/Editor.vue"], linter: { reportUnusedDisableDirectives: setting } },
    ];
    const configurations = [
      {
        id: "global-unused",
        linter: { reportUnusedDisableDirectives: true },
        expected: base,
        status: 1,
      },
      {
        id: "entry-explicit-false",
        linter: { reportUnusedDisableDirectives: true },
        entries: entries(false),
        expected: withoutUnused("src/Editor.vue"),
        status: 1,
      },
      {
        id: "cli-forces-entry-false",
        linter: { reportUnusedDisableDirectives: true },
        entries: entries(false),
        flags: ["--report-unused-disable-directives"],
        expected: base,
        status: 1,
      },
      {
        id: "entry-explicit-true",
        linter: { reportUnusedDisableDirectives: false },
        entries: entries(true),
        expected: withoutUnused("src/PageB.vue"),
        status: 1,
      },
      {
        id: "unused-disabled",
        linter: { reportUnusedDisableDirectives: false },
        expected: ordinary,
        status: 0,
      },
      {
        id: "no-config-does-not-inherit-unused",
        linter: { reportUnusedDisableDirectives: true },
        flags: ["--no-config"],
        expected: ordinary,
        status: 0,
      },
      {
        id: "one-cf-rule-disabled",
        linter: {
          reportUnusedDisableDirectives: true,
          rules: { "croquis/cf/spread-breaks-reactivity": "off" },
        },
        expected: withoutUnused("src/Editor.vue"),
        status: 1,
      },
      {
        id: "cf-group-disabled-keeps-normal-warning",
        linter: { reportUnusedDisableDirectives: true, rules: { "cross-file": "off" } },
        expected: ordinary,
        status: 0,
      },
    ];
    for (const config of configurations) {
      write(stale.files);
      writeVueConfigFixture(temporary, {
        linter: { crossFile: true, ...config.linter },
        entries: config.entries,
      });
      const args = [
        "lint",
        "-f",
        "json",
        "--cross-file",
        "--locale",
        "en",
        ...(config.flags ?? []),
        "src/**/*.vue",
      ];
      run(
        config.id,
        "resolved-config",
        stale.files,
        args,
        phase === "before" ? ordinary : config.expected,
        phase === "before" ? 0 : config.status,
      );
    }
    assert.equal(runs.length, 70);
    for (const observed of runs) {
      const label = observed.id + "/" + observed.route + "/" + observed.repetition;
      assert.equal(observed.error, null, label);
      assert.equal(observed.signal, null, label);
      assert.equal(observed.stderr, "", label);
      assert.equal(observed.status, observed.expectedStatus, label);
      assert.deepEqual(observed.packet, observed.expected, label);
    }
    console.log(
      JSON.stringify({
        qualified: true,
        phase,
        issue: 10912,
        sourceHead: build,
        wholePackets: runs.length,
        artifact,
      }),
    );
  } finally {
    persist();
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
