import assert from "node:assert/strict";
import { test } from "node:test";
import { PassThrough, Writable } from "node:stream";
import { finished } from "node:stream/promises";
import {
  nativeProxyLauncher,
  classifyNativeInvocation,
  nativeObserverCommand,
  teeNativeBytes,
} from "../../tools/benchmarks/scripts/canon-helper-native-proxy.ts";
void test("native stats distinguish compile, probe and explicit empty options validation", () => {
  const cfg = { include: ["src/**/*"] };
  assert.equal(classifyNativeInvocation(["-p", "tsconfig.json"], cfg, true).actualProgram, true);
  assert.equal(classifyNativeInvocation(["--version"], cfg, true).actualProgram, false);
  assert.equal(
    classifyNativeInvocation(["--showConfig", "-p", "x"], cfg, true).actualProgram,
    false,
  );
  assert.equal(
    classifyNativeInvocation(["-p", "x"], { files: [], include: [] }, true).classification,
    "options-validation",
  );
  assert.equal(classifyNativeInvocation(["-p", "x"], null, false).actualProgram, false);
  const args = ["--checkers", "1", "-p", "tsconfig.json"],
    env = {
      GOGC: "75",
      GOMEMLIMIT: "2GiB",
      GODEBUG: "madvdontneed=1,gctrace=0",
    },
    argsBefore = [...args],
    envBefore = { ...env };
  const observed = nativeObserverCommand(args, env, { actualProgram: true });
  assert.deepEqual(observed.args, [...args, "--extendedDiagnostics"]);
  assert.equal(observed.env.GODEBUG, "madvdontneed=1,gctrace=1");
  assert.deepEqual(args, argsBefore);
  assert.deepEqual(env, envBefore);
  assert.deepEqual(nativeObserverCommand(["--version"], env, { actualProgram: false }).args, [
    "--version",
  ]);
  assert.deepEqual(nativeObserverCommand(["--version"], env, { actualProgram: false }).env, env);
});
void test("proxy tee forwards exact binary bytes with backpressure independently to both sinks", async () => {
  const input = new PassThrough(),
    received: Buffer[] = [],
    recorded: Buffer[] = [];
  const sink = new Writable({
    highWaterMark: 1,
    write(chunk: Buffer, _encoding, done) {
      received.push(Buffer.from(chunk));
      setTimeout(done, 1);
    },
  });
  const archive = new Writable({
    highWaterMark: 1,
    write(chunk: Buffer, _encoding, done) {
      recorded.push(Buffer.from(chunk));
      setTimeout(done, 2);
    },
  });
  const bytes = Buffer.concat([
    Buffer.from([0, 255, 13, 10]),
    Buffer.from("Content-Length: 7\r\n\r\n雪👋\n"),
  ]);
  const tee = teeNativeBytes(input, sink, archive, true);
  for (let index = 0; index < bytes.length; index++) input.write(bytes.subarray(index, index + 1));
  input.end();
  await Promise.all([tee.done, finished(sink)]);
  assert.deepEqual(Buffer.concat(received), bytes);
  assert.deepEqual(Buffer.concat(recorded), bytes);
  assert.equal(tee.snapshot().bytes, bytes.length);
  assert.match(tee.snapshot().sha256, /^[0-9a-f]{64}$/u);
});
void test("POSIX launcher keeps the actual SDK separate from its detection basename", () => {
  const script = nativeProxyLauncher({
    node: "/node path/node",
    module: "/code/proxy's.ts",
    out: "/output path",
    executable: "/sdk/tsc",
    mode: "stats",
  });
  assert.match(script, /proxy'\\''s\.ts/u);
  assert.ok(script.endsWith(' "$@"\n'));
  assert.ok(script.includes("'/sdk/tsc'"));
});
