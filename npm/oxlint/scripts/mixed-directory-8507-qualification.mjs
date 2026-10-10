// Additive exact-1.81 qualification, reusing the existing staged source receipt.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { installMixedHost, qualifyHistoricalMixedDirectory } from "./mixed-directory-history.mjs";
import {
  joinMixedCaptures,
  runMixedWorkers,
} from "../src/test-support/mixed-directory-8507-workers.mjs";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const errorPacket = (error) =>
  error == null
    ? null
    : {
        name: error.name,
        ...Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]])),
      };
export async function qualifyMixedDirectory8507({ root, packageDir, artifacts, receipt, binary }) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(receipt.source.head, process.env.GITHUB_SHA);
  assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
  const output = path.join(artifacts, "mixed-directory-8507");
  fs.mkdirSync(output);
  const invocation = {
    complete: false,
    source: receipt.source,
    toolchain: receipt.toolchain,
    binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
    phase: "install",
  };
  const save = () =>
    fs.writeFileSync(
      path.join(output, "invocation.json"),
      JSON.stringify(invocation, null, 2) + "\n",
    );
  save();
  const temporary = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-8507-source-host-")),
  );
  let qualified;
  try {
    const provider = installMixedHost(temporary, path.join(output, "source"));
    const custody = {
      schema: "vize.oxlint.source-native",
      version: 1,
      source: receipt.source,
      toolchain: receipt.toolchain,
      binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
      calls: path.join(output, "source-native-calls.jsonl"),
    };
    const configuration = path.join(output, "source-custody.json");
    fs.writeFileSync(configuration, JSON.stringify(custody, null, 2) + "\n");
    const preload = fileURLToPath(new URL("./project-html-custody.cjs", import.meta.url));
    invocation.phase = "source-after";
    save();
    const records = await runMixedWorkers({
      packageDir,
      output,
      custody,
      preload,
      environment: {
        ...process.env,
        VIZE_OXLINT_TEST_ENTRYPOINT: provider.engine,
      },
    });
    const plan = JSON.parse(
      fs.readFileSync(
        path.join(
          root,
          "tests/_fixtures/differential/lint/oxlint-mixed-directory-8507/controls.json",
        ),
      ),
    );
    const captures = records.map((record) => JSON.parse(fs.readFileSync(record.capture)));
    const journals = records.map((record) => fs.readFileSync(record.calls));
    fs.writeFileSync(custody.calls, Buffer.concat(journals));
    for (const [index, journal] of journals.entries()) {
      assert.deepEqual(
        journal
          .toString("utf8")
          .trim()
          .split("\n")
          .map((line) => JSON.parse(line)),
        [
          ...captures[index].initialEvents,
          ...captures[index].observations.flatMap((record) => record.events),
        ],
      );
    }
    const capture = joinMixedCaptures(plan, custody, captures);
    fs.writeFileSync(
      path.join(output, "source-after.json"),
      JSON.stringify(capture, null, 2) + "\n",
    );
    assert.equal(capture.complete, true);
    assert.deepEqual(capture.source, receipt.source);
    assert.deepEqual(capture.passed, { cli: 45, native: 8 });
    invocation.phase = "historical-before";
    save();
    qualified = {
      sourceAfter: capture.passed,
      provider,
      historicalBefore: qualifyHistoricalMixedDirectory({
        root,
        output: path.join(output, "before"),
      }),
    };
  } catch (error) {
    invocation.failure = errorPacket(error);
    save();
    throw error;
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
  assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
  fs.writeFileSync(
    path.join(output, "qualification.json"),
    JSON.stringify(
      {
        source: receipt.source,
        toolchain: receipt.toolchain,
        binarySha256: receipt.frozen.sha256,
        ...qualified,
        limits: [
          "source-built Linux Actions, not the reporter's Darwin installation",
          "public installed acceptance requires a genuinely included published release",
        ],
      },
      null,
      2,
    ) + "\n",
  );
  invocation.complete = true;
  invocation.phase = "complete";
  save();
  return qualified;
}
