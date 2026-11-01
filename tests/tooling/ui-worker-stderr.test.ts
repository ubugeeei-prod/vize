import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { test } from "node:test";
import { promisify } from "node:util";
import { retainCapturedWorkerStderr } from "../../npm/ui/scripts/source-quality/lint-sfc.ts";

test("unknown captured stderr stays whole without changing the returned object", async () => {
  const output = {
    stdout: '[{"filename":"src/雪🌸.vue","diagnostics":[]}]',
    stderr: "unknown provider warning\n\u0000雪�\n",
  };
  const retained: string[] = [];
  const actual = await retainCapturedWorkerStderr(Promise.resolve(output), (value) => {
    retained.push(value);
  });
  assert.equal(actual, output);
  assert.deepEqual(retained, [output.stderr]);
  assert.deepEqual(Buffer.from(retained[0]!, "utf8"), Buffer.from(output.stderr, "utf8"));
  // This is Node's already-decoded representation. Upstream raw bytes UNKNOWN.
});

test("retention failures cannot replace a successful result or original rejected object", async () => {
  const output = { stdout: "[]", stderr: "unclassified warning" };
  const original = Object.assign(new Error("original process failure"), {
    stderr: "unknown failure\n",
  });
  const retained: string[] = [];
  const retain = (value: string) => {
    retained.push(value);
    throw new Error("sink unavailable");
  };
  assert.equal(await retainCapturedWorkerStderr(Promise.resolve(output), retain), output);
  await assert.rejects(
    retainCapturedWorkerStderr(Promise.reject(original), retain),
    (error: unknown) => error === original,
  );
  assert.deepEqual(retained, [output.stderr, original.stderr]);
});

test("missing or hostile stderr properties never become an invented empty stream", async () => {
  const retained: string[] = [];
  const original = Object.defineProperty(new Error("original"), "stderr", {
    get() {
      throw new Error("hostile property");
    },
  });
  await assert.rejects(
    retainCapturedWorkerStderr(Promise.reject(original), (value) => {
      retained.push(value);
    }),
    (error: unknown) => error === original,
  );
  const output = { stdout: "[]" };
  assert.equal(
    await retainCapturedWorkerStderr(Promise.resolve(output), (value) => {
      retained.push(value);
    }),
    output,
  );
  assert.deepEqual(retained, []);
});

interface Outcome {
  state: "returned" | "rejected";
  stdout: string;
  stderr: string;
  code: string | number | null;
  killed: boolean | null;
  signal: string | null;
  message: string | null;
}
async function outcome(promise: Promise<{ stdout: string; stderr: string }>): Promise<Outcome> {
  try {
    const value = await promise;
    return { state: "returned", ...value, code: null, killed: null, signal: null, message: null };
  } catch (error) {
    assert(typeof error === "object" && error !== null);
    assert("stdout" in error && typeof error.stdout === "string");
    assert("stderr" in error && typeof error.stderr === "string");
    assert("code" in error && (typeof error.code === "number" || typeof error.code === "string"));
    return {
      state: "rejected",
      stdout: error.stdout,
      stderr: error.stderr,
      code: error.code,
      killed: "killed" in error && typeof error.killed === "boolean" ? error.killed : null,
      signal: "signal" in error && typeof error.signal === "string" ? error.signal : null,
      message: "message" in error && typeof error.message === "string" ? error.message : null,
    };
  }
}

// Real plain-Node counterfactuals. No Vize/SDK process is launched.
// One invalid byte costs three decoded UTF8 bytes: Buffer capture would change
// the original maxBuffer failure. The production encoding is left untouched.
for (const control of [
  { id: "invalid-within-decoded-cap", limit: 4, bytes: [255], failure: false },
  { id: "invalid-over-decoded-cap", limit: 2, bytes: [255], failure: true },
  { id: "unicode-at-cap", limit: 7, bytes: [...Buffer.from("雪🌸")], failure: false },
  { id: "unicode-over-cap", limit: 6, bytes: [...Buffer.from("雪🌸")], failure: true },
  { id: "split-unicode-at-cap", limit: 3, bytes: [233, 155], tail: [170], failure: false },
  { id: "split-unicode-over-cap", limit: 2, bytes: [233, 155], tail: [170], failure: true },
]) {
  test(`original UTF8 execFile gate remains unchanged: ${control.id}`, async () => {
    const script = `process.stdout.write("[]"); process.stderr.write(Buffer.from(${JSON.stringify(control.bytes)})); ${control.tail ? `setTimeout(()=>process.stderr.write(Buffer.from(${JSON.stringify(control.tail)})),5);` : ""} setTimeout(()=>{},40);`;
    const launch = () =>
      promisify(execFile)(process.execPath, ["-e", script], { maxBuffer: control.limit });
    const baseline = await outcome(launch());
    const retained: string[] = [];
    const actual = await outcome(
      retainCapturedWorkerStderr(launch(), (value) => {
        retained.push(value);
      }),
    );
    assert.deepEqual(actual, baseline);
    assert.equal(actual.state, control.failure ? "rejected" : "returned");
    if (control.failure) assert.equal(actual.code, "ERR_CHILD_PROCESS_STDIO_MAXBUFFER");
    assert.deepEqual(retained, [actual.stderr]);
    assert.deepEqual(Buffer.from(retained[0]!, "utf8"), Buffer.from(actual.stderr, "utf8"));
  });
}
