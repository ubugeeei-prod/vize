import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { apiRoot, gitBlob } from "../../tools/support/release/typed_defaults/custody.mjs";
import {
  loadFixtures,
  pins,
  scenarios,
} from "../../tools/support/release/lint_reports/fixtures.mjs";
import {
  extraOverrides,
  reportOptions,
} from "../../tools/support/release/lint_reports/custody.mjs";
import { verifyReportPacket } from "../../tools/support/release/lint_reports/observe.ts";
import { writeTarGz } from "../../tools/support/archive/tar.ts";
const root = fileURLToPath(new URL("../../", import.meta.url));
const source = new Map(
  Object.keys(pins).map((name) => [name, fs.readFileSync(path.join(root, name))]),
);
const vectors = scenarios(source);
const packet = (actual, status = 0) => ({
  error: null,
  signal: null,
  status,
  stderrBase64: "",
  stdoutBase64: Buffer.from(JSON.stringify(actual)).toString("base64"),
  inputs: [],
});

await test("released reports preserve every whole original, unsafe neighbor and computed control", async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-report-laws-"));
  try {
    const treeSha = "a".repeat(40),
      head = "b".repeat(40);
    const entries = [...source].map(([name, bytes]) => ({
      path: name,
      mode: "100644",
      type: "blob",
      sha: gitBlob(bytes),
    }));
    const blobs = new Map([...source.values()].map((bytes) => [gitBlob(bytes), bytes]));
    const read = async (url) => {
      if (url === `${apiRoot}/git/trees/${treeSha}?recursive=1`)
        return { sha: treeSha, truncated: false, tree: entries };
      const sha = url.split("/").at(-1),
        bytes = blobs.get(sha);
      assert(bytes);
      return { sha, encoding: "base64", size: bytes.length, content: bytes.toString("base64") };
    };
    const fixtures = await loadFixtures(head, treeSha, path.join(directory, "valid"), read);
    assert.equal(fixtures.vectors.length, 65);
    assert.equal(fixtures.vectors.filter((v) => v.suite === "template").length, 29);
    assert.equal(fixtures.vectors.filter((v) => v.suite === "computed").length, 36);
    assert.equal(
      fixtures.vectors.filter((v) => v.role === "original-config-oracles.json").length,
      4,
    );
    const unsafe = fixtures.vectors
      .filter((v) => v.suite === "template")
      .reduce((total, v) => total + (v.afterCli ?? v.afterFormatter)[0].warningCount, 0);
    assert.equal(unsafe, 13);
    const original = entries[0].sha;
    entries[0].sha = gitBlob(Buffer.from("mutated"));
    blobs.set(entries[0].sha, Buffer.from("mutated"));
    await assert.rejects(
      loadFixtures(head, treeSha, path.join(directory, "mutated"), read),
      /whole frozen fixture changed/,
    );
    entries[0].sha = original;
    await assert.rejects(
      loadFixtures(head, treeSha, path.join(directory, "truncated"), async (url) => ({
        ...(await read(url)),
        truncated: true,
      })),
      /complete immutable H tree/,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

await test("whole template packets reject missing unsafe diagnostics, ranges, extra findings and status failures", () => {
  const scenario = vectors.find(
    (v) => v.suite === "template" && (v.afterCli ?? v.afterFormatter)[0].warningCount,
  );
  const expected = scenario.afterCli ?? scenario.afterFormatter;
  verifyReportPacket(scenario, packet(expected));
  for (const mutate of [
    (v) => {
      v[0].messages = [];
      v[0].warningCount = 0;
    },
    (v) => {
      v[0].messages[0].column++;
    },
    (v) => {
      v[0].messages.push({ ...v[0].messages[0], ruleId: "unexpected/rule" });
    },
  ]) {
    const changed = structuredClone(expected);
    mutate(changed);
    assert.throws(
      () => verifyReportPacket(scenario, packet(changed)),
      /whole frozen template formatter/,
    );
  }
  assert.throws(() => verifyReportPacket(scenario, packet(expected, 1)));
  const error = packet(expected);
  error.error = { message: "missing provider" };
  assert.throws(() => verifyReportPacket(scenario, error));
});

await test("computed finite controls refuse lost real snapshot warnings and retained async errors", () => {
  const scenario = vectors.find(
    (v) => v.suite === "computed" && v.expectedSnapshotAndSpreadWarnings === 2,
  );
  const actual = [
    {
      file: scenario.filename,
      messages: [
        {
          ruleId: "type/no-reactivity-loss",
          severity: 1,
          message: "Assigning 'held.value' to 'h' stores a plain snapshot",
        },
        {
          ruleId: "type/no-reactivity-loss",
          severity: 1,
          message: "Spreading 'h.shape' creates a non-reactive copy",
        },
      ],
      errorCount: 0,
      warningCount: 2,
    },
  ];
  verifyReportPacket(scenario, packet(actual));
  actual[0].messages.pop();
  actual[0].warningCount--;
  assert.throws(() => verifyReportPacket(scenario, packet(actual)));
  const asyncOwner = vectors.find((v) => v.suite === "computed" && v.expectedCliErrors?.length);
  assert(asyncOwner);
  assert.throws(() =>
    verifyReportPacket(
      asyncOwner,
      packet([{ file: asyncOwner.filename, messages: [], errorCount: 0, warningCount: 0 }]),
    ),
  );
});

await test("source runtime overrides cannot satisfy an installed report observation", () => {
  const env = {
    PUBLIC_CUT: "a".repeat(40),
    PUBLIC_HEAD: "b".repeat(40),
    PUBLIC_TAG: "v0.449.0",
    PUBLIC_SOURCE_PR: "1",
    PUBLIC_RELEASE_RUN: "2",
  };
  assert.equal(reportOptions(env).tag, env.PUBLIC_TAG);
  for (const key of [...extraOverrides, "CORSA_PATH", "VIZE_BINDING_PATH"])
    assert.throws(() => reportOptions({ ...env, [key]: "foreign" }), /runtime override/);
});

await test("registry archive members require the actual unchanged installed bytes", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-package-law-"));
  try {
    fs.mkdirSync(path.join(directory, "installed"));
    fs.writeFileSync(path.join(directory, "installed/package.json"), "original");
    fs.writeFileSync(
      path.join(directory, "package.tgz"),
      writeTarGz([{ name: "package/package.json", data: Buffer.from("original") }]),
    );
    const argv = [
      path.join(root, "tools/support/release/lint_reports/package_members.ts"),
      path.join(directory, "package.tgz"),
      path.join(directory, "installed"),
    ];
    assert.equal(spawnSync(process.execPath, argv).status, 0);
    fs.writeFileSync(path.join(directory, "installed/package.json"), "changed!");
    assert.notEqual(spawnSync(process.execPath, argv).status, 0);
    fs.writeFileSync(path.join(directory, "installed/package.json"), "original");
    fs.writeFileSync(path.join(directory, "installed/injected.js"), "injected");
    assert.notEqual(spawnSync(process.execPath, argv).status, 0);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
