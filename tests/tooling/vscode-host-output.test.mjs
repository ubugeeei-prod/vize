import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { parse } from "yaml";
import {
  retainHostOutput,
  retainHostOutputBeforeCleanup,
} from "../../editors/vscode/test/host-output.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const runnerPath = path.join(root, "editors/vscode/test/run-extension-host.mjs");
const runner = fs
  .readFileSync(runnerPath, "utf8")
  .replace(/^#![^\n]*\n/, "")
  .replace(/^import[\s\S]*?;\n/gm, "");

function nativeLog(profile, session = "20261011T101600", window = "window1") {
  return path.join(
    profile,
    "user-data/logs",
    session,
    window,
    "exthost/output_logging_1/1-Vize.log",
  );
}

function write(file, bytes) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, bytes);
}

await test("native output retention preserves every byte and every session without copying other logs", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-output-bytes-"));
  const profile = path.join(temporary, "profile");
  const destination = path.join(temporary, "retained");
  const bytes = Buffer.concat([Buffer.alloc(300_001, 0x61), Buffer.from("\n日本語\0\xff", "utf8")]);
  const paths = [nativeLog(profile), nativeLog(profile, "20261011T101700", "window2")];
  try {
    for (const file of paths) write(file, bytes);
    write(path.join(path.dirname(paths[0]), "2-Other.log"), "not Vize");
    write(path.join(profile, "extensions/secret.txt"), "not an output log");
    retainHostOutput(profile, destination);
    for (const file of paths) {
      const relative = path.relative(path.join(profile, "user-data/logs"), file);
      assert.deepEqual(fs.readFileSync(path.join(destination, relative)), bytes);
    }
    assert.equal(fs.existsSync(path.join(destination, "extensions")), false);
    assert.equal(
      fs.existsSync(
        path.join(
          destination,
          path.dirname(path.relative(path.join(profile, "user-data/logs"), paths[0])),
          "2-Other.log",
        ),
      ),
      false,
    );
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

await test("missing logs and symlink escape are explicit retention refusals and still clean the profile", () => {
  for (const mode of ["missing", "symlink", "symlink-root"]) {
    const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-output-refusal-"));
    const profile = path.join(temporary, "profile");
    const destination = path.join(temporary, "retained");
    fs.mkdirSync(profile);
    try {
      if (mode === "symlink") {
        const file = nativeLog(profile);
        fs.mkdirSync(path.dirname(file), { recursive: true });
        write(path.join(temporary, "outside.txt"), "outside profile");
        fs.symlinkSync(path.join(temporary, "outside.txt"), file);
      } else if (mode === "symlink-root") {
        const outside = path.join(temporary, "outside-logs");
        write(path.join(outside, "output_logging_1/1-Vize.log"), "outside profile");
        fs.mkdirSync(path.join(profile, "user-data"));
        fs.symlinkSync(outside, path.join(profile, "user-data/logs"), "dir");
      }
      retainHostOutputBeforeCleanup(profile, destination);
      assert.equal(fs.existsSync(profile), false);
      assert.match(
        fs.readFileSync(path.join(destination, "retention-error.txt"), "utf8"),
        mode.startsWith("symlink") ? /Refusing symbolic link/ : /ENOENT/,
      );
      assert.equal(fs.existsSync(path.join(destination, "outside.txt")), false);
    } finally {
      fs.rmSync(temporary, { recursive: true, force: true });
    }
  }
});

await test("actual runner retains native output on success and failure while preserving original error and cleanup", async () => {
  for (const mode of ["success", "failure", "retention-failure"]) {
    const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-output-runner-"));
    const extension = path.join(temporary, "extension");
    const originalError = new Error("original host failure");
    const bytes = Buffer.from(`[config] actual callback ${mode}\n`);
    let profile;
    write(
      path.join(extension, "test-fixtures/extension-host/basic-vue/src/App.vue"),
      "<template/>",
    );
    write(path.join(extension, "package.json"), '{"version":"0.443.0"}');
    const context = vm.createContext({
      fs,
      os: { ...os, tmpdir: () => temporary },
      path,
      process,
      fileURLToPath: () => path.join(extension, "test/run-extension-host.mjs"),
      retainHostOutputBeforeCleanup,
      async runTests(options) {
        assert.equal(options.version, process.env.VIZE_TEST_VSCODE_VERSION ?? "1.107.1");
        assert.equal(options.extensionTestsEnv.VIZE_TEST_ENABLE_HOST_COMMANDS, "1");
        assert.equal(options.launchArgs[options.launchArgs.indexOf("--log") + 1], "trace");
        const userData = options.launchArgs.find((argument) =>
          argument.startsWith("--user-data-dir="),
        );
        profile = path.dirname(userData.slice("--user-data-dir=".length));
        write(nativeLog(profile), bytes);
        write(
          path.join(profile, "user-data/logs/session/sharedprocess.log"),
          "complete native watcher registration\n",
        );
        if (mode === "retention-failure")
          write(path.join(extension, ".vscode-test/host-smoke/output-channel"), "blocked");
        if (mode !== "success") throw originalError;
      },
    });
    // The actual runner body, including its original catch and finally, executes.
    const execute = new vm.Script(
      `(async () => {${runner.replaceAll("import.meta.url", '"runner"')}\n})()`,
    );
    try {
      if (mode === "success") await execute.runInContext(context);
      else await assert.rejects(execute.runInContext(context), (error) => error === originalError);
      assert.equal(fs.existsSync(profile), false);
      if (mode !== "retention-failure") {
        const output = path.join(extension, ".vscode-test/host-smoke/output-channel");
        assert.equal(
          fs.readFileSync(path.join(output, "session/sharedprocess.log"), "utf8"),
          "complete native watcher registration\n",
        );
        assert.deepEqual(
          fs.readFileSync(
            path.join(
              output,
              path.relative(path.join(profile, "user-data/logs"), nativeLog(profile)),
            ),
          ),
          bytes,
        );
      }
      if (mode !== "success") {
        const failure = path.join(extension, ".vscode-test/host-smoke/failure");
        assert.equal(
          JSON.parse(fs.readFileSync(path.join(failure, "error.json"), "utf8")).message,
          String(originalError),
        );
        assert.equal(
          fs.readFileSync(path.join(failure, "workspace/src/App.vue"), "utf8"),
          "<template/>",
        );
      }
    } finally {
      fs.rmSync(temporary, { recursive: true, force: true });
    }
  }
});

await test("complete diagnostic logs preserve native bytes while excluding the rest of the profile", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-output-native-"));
  const profile = path.join(temporary, "profile");
  const destination = path.join(temporary, "retained");
  const logs = [
    "session/main.log",
    "session/sharedprocess.log",
    "session/window1/renderer.log",
    "session/window1/exthost/exthost.log",
  ];
  const bytes = Buffer.from("native watcher\0\xff\n");
  try {
    write(nativeLog(profile), "Vize output");
    for (const log of logs) write(path.join(profile, "user-data/logs", log), bytes);
    write(path.join(profile, "user-data/User/settings.json"), "not a log");
    write(path.join(profile, "extensions/private.txt"), "not a log");
    retainHostOutput(profile, destination, true);
    for (const log of logs) assert.deepEqual(fs.readFileSync(path.join(destination, log)), bytes);
    assert.equal(fs.existsSync(path.join(destination, "User")), false);
    assert.equal(fs.existsSync(path.join(destination, "extensions")), false);
    fs.symlinkSync(
      path.join(profile, "extensions/private.txt"),
      path.join(profile, "user-data/logs/escaped.log"),
    );
    assert.throws(() => retainHostOutput(profile, destination, true), /Refusing symbolic link/);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

await test("existing host routes retain configuration packets and native logs unconditionally", () => {
  const action = parse(
    fs.readFileSync(path.join(root, ".github/actions/vscode-host-smoke/action.yml"), "utf8"),
  );
  const upload = action.runs.steps.find((step) => step.with?.name === "vscode-real-host-failure");
  assert.equal(upload.if, "${{ always() }}");
  for (const required of [
    "editors/vscode/.vscode-test/host-smoke/output-channel/",
    "editors/vscode/.vscode-test/host-smoke/fake-server/existing-config-events/",
  ]) {
    assert.ok(upload.with.path.split("\n").includes(required));
  }
  const workflow = parse(
    fs.readFileSync(path.join(root, ".github/workflows/editor-conformance.yml"), "utf8"),
  );
  const hostSteps = Object.values(workflow.jobs)
    .flatMap((job) => job.steps ?? [])
    .filter(
      (step) => step.run === "xvfb-run -a vp run --workspace-root test:vscode-extension:host",
    );
  assert.equal(hostSteps.length, 1);
  assert.equal(hostSteps[0].if, "matrix.client == 'vscode'");
});
