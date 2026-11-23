import assert from "node:assert/strict";
import childProcess from "node:child_process";
import { readFileSync } from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import { inspect } from "node:util";

const input = JSON.parse(readFileSync(0, "utf8"));
const worker = readFileSync(new URL("./sandbox-worker.js", input.moduleUrl), "utf8");
process.env.SANDBOX_CUSTODY_ENV = "environment-must-not-be-copied";
let state;
childProcess.spawnSync = (command, args, options) => {
  assert.equal(command, "docker");
  state.calls.push({ args, options });
  if (args[0] === "image") return { status: 0, stdout: "a".repeat(64), stderr: "" };
  if (args[0] === "run") return state.result;
  if (args[0] === "rm") return { status: state.cleanupFails ? 1 : 0, stderr: "daemon reply" };
  assert.equal(args[0], "container");
  return {
    error: Object.assign(new Error("inspect unavailable"), { code: "EIO" }),
    status: null,
    stderr: "daemon unavailable",
  };
};
syncBuiltinESMExports();
const { createSandboxRunner, SandboxRuntimeError, SANDBOX_IMAGE } = await import(input.moduleUrl);
const definition = {
  name: "custody-proof",
  version: "1",
  configuration: "request-must-not-be-copied",
  rules: { check: "() => []" },
};
const batch = '{"schema":1,"plugin":"custody-proof"}';
for (const mode of ["ETIMEDOUT", "ENOBUFS", "ENOENT", "cleanup", "success", "exit", "json"]) {
  try {
    const code = mode === "cleanup" ? "ETIMEDOUT" : mode;
    const stopped = ["ETIMEDOUT", "ENOBUFS", "ENOENT", "cleanup"].includes(mode);
    const cause = Object.assign(new Error("stopped docker"), { code });
    const stdout = mode === "ENOENT" ? null : "\nAVAILABLE_STDOUT 日本語😀\n ";
    const stderr = mode === "ENOENT" ? null : "\tAVAILABLE_STDERR 🎨\n ";
    state = {
      calls: [],
      cleanupFails: mode === "cleanup",
      result: stopped
        ? {
            error: cause,
            status: null,
            signal: mode === "ENOENT" ? null : "SIGKILL",
            stdout,
            stderr,
          }
        : {
            status: mode === "exit" ? 2 : 0,
            signal: null,
            stdout: mode === "json" ? "not-json" : "[]",
            stderr: "child failed\n",
          },
    };
    const runner = createSandboxRunner(definition);
    let failure;
    try {
      assert.equal(runner.run(batch), "[]");
    } catch (error) {
      failure = error;
    }
    if (mode === "success") assert.equal(failure, undefined);
    else if (mode === "exit") {
      assert.ok(failure instanceof SandboxRuntimeError);
      assert.equal(failure.code, "execution_failed");
      assert.equal(failure.execution, undefined);
    } else if (mode === "json") assert.ok(failure instanceof SyntaxError);
    else {
      assert.ok(failure instanceof SandboxRuntimeError);
      if (state.cleanupFails) {
        assert.equal(failure.code, "cleanup_failed");
        assert.ok(failure.cause instanceof AggregateError);
        assert.equal(failure.cause.errors.length, 2);
        failure = failure.cause.errors[0];
      }
      assert.equal(failure.code, "execution_stopped");
      assert.equal(failure.message, "sandbox execution stopped: " + code);
      assert.equal(failure.cause, cause);
      const observation = failure.execution;
      assert.deepEqual(Object.keys(observation), [
        "phase",
        "elapsedMs",
        "status",
        "signal",
        "stdout",
        "stderr",
      ]);
      assert.equal(observation.phase, "run");
      assert.ok(Number.isFinite(observation.elapsedMs) && observation.elapsedMs >= 0);
      assert.equal(observation.status, state.result.status);
      assert.equal(observation.signal, state.result.signal);
      assert.equal(observation.stdout, stdout);
      assert.equal(observation.stderr, stderr);
      assert.ok(Object.isFrozen(observation));
      assert.throws(() => {
        observation.stderr = "changed";
      }, TypeError);
      assert.throws(() => {
        failure.execution = {};
      }, TypeError);
      state.result.stdout = "changed after capture";
      assert.equal(observation.stdout, stdout, "snapshot must not alias the mutable result object");
      assert.deepEqual(Object.getOwnPropertyDescriptor(failure, "execution"), {
        value: observation,
        writable: false,
        enumerable: false,
        configurable: false,
      });
      assert.ok(!Object.keys(failure).includes("execution"));
      assert.doesNotMatch(
        JSON.stringify(failure) + inspect(failure, { depth: null }),
        /AVAILABLE_STDOUT|AVAILABLE_STDERR|request-must-not-be-copied|environment-must-not-be-copied/,
      );
      assert.doesNotMatch(
        JSON.stringify(observation),
        /request-must-not-be-copied|environment-must-not-be-copied/,
      );
    }
    const [prerequisite, run, remove, confirm] = state.calls;
    assert.equal(state.calls.length, state.cleanupFails ? 4 : 3, "one execution, never a retry");
    assert.deepEqual(prerequisite.args, ["image", "inspect", "--format={{.Id}}", SANDBOX_IMAGE]);
    assert.deepEqual(prerequisite.options, { encoding: "utf8", timeout: 5000, maxBuffer: 16384 });
    const name = run.args[4];
    assert.match(name, /^vize-plugin-[0-9a-f-]{36}$/);
    assert.deepEqual(run.args, [
      "run",
      "--rm",
      "--pull=never",
      "--name",
      name,
      "--label",
      "vize.plugin-sandbox=1",
      "--label",
      "vize.plugin-sandbox.host=" + process.pid,
      "--network=none",
      "--read-only",
      "--user=65534:65534",
      "--cap-drop=ALL",
      "--security-opt=no-new-privileges:true",
      "--pids-limit=32",
      "--memory=128m",
      "--memory-swap=128m",
      "--cpus=1",
      "--ulimit=nofile=64:64",
      "--ipc=none",
      "--log-driver=none",
      "--workdir=/",
      "--entrypoint=node",
      "-i",
      SANDBOX_IMAGE,
      "--max-old-space-size=64",
      "--input-type=module",
      "--eval",
      worker,
    ]);
    const { input: request, ...options } = run.options;
    assert.deepEqual(options, {
      encoding: "utf8",
      timeout: 5000,
      maxBuffer: 1048576,
      killSignal: "SIGKILL",
      windowsHide: true,
    });
    const payload = JSON.parse(request);
    assert.deepEqual(payload.definition, { ...definition, family: "rule" });
    assert.equal(payload.batchJson, batch);
    assert.match(payload.sdk, /^data:text\/javascript;base64,/);
    assert.deepEqual(remove.args, ["rm", "--force", name]);
    assert.equal(remove.options.encoding, "utf8");
    assert.equal(remove.options.maxBuffer, 16384);
    assert.equal(remove.options.killSignal, "SIGKILL");
    assert.ok(remove.options.timeout >= 1 && remove.options.timeout <= 5000);
    if (confirm) assert.deepEqual(confirm.args, ["container", "inspect", "--format={{.Id}}", name]);
  } catch (error) {
    throw new Error("sandbox custody mode " + mode + " failed", { cause: error });
  }
}
process.stdout.write("seven sandbox custody controls passed\n");
