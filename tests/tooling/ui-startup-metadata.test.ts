import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { asArray, asRecord, parseJson } from "../../tools/support/json-value.ts";
import {
  flushStartupWorker,
  normalizedError,
  startupInput,
  startupParent,
  startupProvider,
} from "../../npm/ui/scripts/source-quality/startup-metadata.ts";

function fixture(run: (root: string, output: string) => void): void {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "ui-startup-metadata-control-"))),
    output = join(root, "metadata");
  mkdirSync(output);
  const keys = [
    "GITHUB_ACTIONS",
    "RUNNER_TEMP",
    "VIZE_UI_STARTUP_METADATA_DIR",
    "VIZE_UI_STARTUP_CONTEXT",
  ] as const;
  const previous = new Map(keys.map((name) => [name, process.env[name]] as const));
  process.env.GITHUB_ACTIONS = "true";
  process.env.RUNNER_TEMP = root;
  process.env.VIZE_UI_STARTUP_METADATA_DIR = output;
  try {
    run(root, output);
  } finally {
    for (const [name, value] of previous)
      if (value === undefined) delete process.env[name];
      else process.env[name] = value;
    rmSync(root, { recursive: true, force: true });
  }
}
const json = (path: string) => asRecord(parseJson(readFileSync(path, "utf8")));

test("error normalization never retains payloads or invokes a code getter/coercion", () => {
  const secret = "sensitive-source-or-transport-payload";
  let touched = 0;
  const value = {
    message: secret,
    get code() {
      touched++;
      throw new Error(secret);
    },
  };
  assert.deepEqual(normalizedError(value), { kind: "other", code: "other", redacted: true });
  const error = Object.assign(new Error(secret), {
    code: {
      toString() {
        touched++;
        throw new Error(secret);
      },
    },
  });
  assert.deepEqual(normalizedError(error), { kind: "Error", code: "other", redacted: true });
  assert.equal(touched, 0);
  assert.equal(JSON.stringify(normalizedError(error)).includes(secret), false);
});

test("invalid observer setup cannot replace an original return or throw", () =>
  fixture((root) => {
    process.env.VIZE_UI_STARTUP_METADATA_DIR = join(root, "missing");
    const value = {},
      original = new Error("original-lint-failure");
    let calls = 0;
    assert.equal(
      startupInput("input.vue", "source", 0, { index: 0, total: 1 }, () => {
        calls++;
        return value;
      }),
      value,
    );
    assert.throws(
      () =>
        startupInput("input.vue", "source", 1, { index: 0, total: 1 }, () => {
          calls++;
          throw original;
        }),
      (error: unknown) => error === original,
    );
    assert.doesNotThrow(() => flushStartupWorker());
    assert.doesNotThrow(() => startupParent());
    assert.equal(calls, 2);
  }));

test(
  "worker output has paths only and fail-fast parent output leaves image endpoints UNKNOWN",
  { skip: process.platform !== "linux" },
  () =>
    fixture((root, output) => {
      const provider = join(root, "plain-provider.bin");
      writeFileSync(provider, "never executed\n");
      startupProvider(provider);
      const finish = startupParent();
      assert.ok(finish);
      process.env.VIZE_UI_STARTUP_CONTEXT = "prior-private-context";
      assert.equal(
        startupInput(
          join(root, "app.vue"),
          "private source contents",
          0,
          { index: 0, total: 1 },
          () => 7,
        ),
        7,
      );
      assert.equal(process.env.VIZE_UI_STARTUP_CONTEXT, "prior-private-context");
      flushStartupWorker();
      const workerName = readdirSync(output).find((name) => name.startsWith("ui-worker-"));
      assert.ok(workerName);
      const worker = json(join(output, workerName));
      assert.equal(worker.selected_provider_path, provider);
      assert.equal(worker.content_hashes_deferred_to_parent, true);
      assert.equal(Object.hasOwn(worker, "selected_provider"), false);
      assert.equal(Object.hasOwn(worker, "already_loaded_node_images"), false);
      assert.equal(JSON.stringify(worker).includes("private source contents"), false);
      assert.equal(JSON.stringify(worker).includes("prior-private-context"), false);
      assert.equal(asRecord(asArray(worker.inputs)[0]).wire_request_id, null);
      finish("threw");
      const parent = json(join(output, `ui-parent-${process.pid}.json`));
      assert.deepEqual(parent.executable_hashes, []);
      assert.deepEqual(parent.build_manifests, []);
      assert.equal(parent.hashes_omitted_reason, "siblings_may_still_be_active");
      assert.equal(parent.sample_window, "prefix_until_original_fail_fast");
      assert.equal(parent.expected_worker_census, "UNKNOWN_not_validated");
      assert.equal(Object.hasOwn(parent, "worker_metadata_complete"), false);
      assert.equal(parent.physical_same_image_custody, "UNPROVEN_post_outcome_hashes_only");
    }),
);

test(
  "content hashing observes bytes changed AFTER worker flush and only at returned parent outcome",
  { skip: process.platform !== "linux" },
  () =>
    fixture((root, output) => {
      const provider = join(root, "plain-provider.bin");
      writeFileSync(provider, "before worker\n");
      startupProvider(provider);
      const finish = startupParent();
      assert.ok(finish);
      startupInput(join(root, "app.vue"), "source", 0, { index: 0, total: 1 }, () => 7);
      flushStartupWorker();
      const later = Buffer.from("after worker\n");
      writeFileSync(provider, later);
      finish("returned");
      const parent = json(join(output, `ui-parent-${process.pid}.json`));
      const observed = asArray(parent.executable_hashes)
        .map((row) => asRecord(row))
        .find((row) => row.path === provider);
      assert.ok(observed);
      assert.equal(observed.sha256, createHash("sha256").update(later).digest("hex"));
      assert.equal(observed.hash_timing, "after_all_original_workers_returned");
      assert.equal(parent.hashes_omitted_reason, null);
      assert.equal(parent.discovered_worker_files_parsed, true);
      assert.equal(parent.expected_worker_census, "UNKNOWN_not_validated");
    }),
);
