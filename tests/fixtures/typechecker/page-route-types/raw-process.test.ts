import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { ownError, rawCommand } from "./raw-process.ts";
import { fakeFixture, temporary } from "./raw-process-test-workspace.ts";
import { PAGE_PATH, PLAYGROUND, typeofReference } from "./support.ts";

type Packet = Record<string, unknown>;
type Descriptor = {
  key: { type: string; value?: string; description?: string; globalKey?: string | null };
  configurable: boolean;
  enumerable: boolean;
  writable?: boolean;
  value?: unknown;
  get?: { type: string; source: string };
};

function property(snapshot: unknown, name: string | symbol): Descriptor {
  const descriptors = (snapshot as { properties: Descriptor[] }).properties;
  assert.ok(Array.isArray(descriptors));
  const found = descriptors.find(({ key }) =>
    typeof name === "string"
      ? key.type === "string" && key.value === name
      : key.type === "symbol" && key.description === name.description,
  );
  assert.ok(found, `captured own property: ${String(name)}`);
  return found;
}

function decoded(value: unknown): Buffer {
  const bytes = value as { bytes: number; base64: string };
  const result = Buffer.from(bytes.base64, "base64");
  assert.equal(result.length, bytes.bytes);
  return result;
}

function recorder() {
  const packets: Packet[] = [];
  return {
    packets,
    record(this: void, value: unknown) {
      // Exercise the provider's JSON persistence boundary rather than retaining objects.
      packets.push(JSON.parse(JSON.stringify(value)) as Packet);
    },
  };
}

function recorded(packets: Packet[], stage: string, observation?: string): Packet {
  const result = packets.find(
    (packet) => packet.stage === stage && (!observation || packet.observation === observation),
  );
  assert.ok(result, `${stage}/${observation ?? "any"} was saved`);
  return result;
}

await test("binary stdout/stderr and nonzero status are recorded before UTF-8 decoding", (t) => {
  const cwd = temporary(t);
  const stdout = Buffer.from([0, 102, 128, 255, 10]);
  const stderr = Buffer.from([129, 192, 0, 13]);
  const script = `process.stdout.write(Buffer.from(${JSON.stringify([...stdout])}));
process.stderr.write(Buffer.from(${JSON.stringify([...stderr])}));
process.exitCode = process.env.LANG === 'C' && process.env.LC_ALL === 'C'
  && process.cwd() === ${JSON.stringify(fs.realpathSync(cwd))} ? 23 : 24;`;
  const args = ["-e", script];
  const capture = recorder();
  const originalToString = Buffer.prototype.toString;
  let saved = false;
  Buffer.prototype.toString = function (this: Buffer, encoding, start, end) {
    if (
      (encoding === undefined || encoding === "utf8" || encoding === "utf-8") &&
      (this.equals(stdout) || this.equals(stderr))
    ) {
      assert.ok(saved, "the raw record must already be persisted before decoding these bytes");
    }
    return originalToString.call(this, encoding, start, end);
  };
  try {
    const result = rawCommand(
      process.execPath,
      args,
      cwd,
      (value) => {
        capture.record(value);
        if ((value as Packet).observation === "process-result") saved = true;
      },
      { stage: "binary" },
    );
    const raw = recorded(capture.packets, "binary", "process-result");
    assert.deepEqual(decoded(raw.stdout), stdout);
    assert.deepEqual(decoded(raw.stderr), stderr);
    assert.equal(raw.status, 23);
    assert.equal(raw.signal, null);
    assert.equal(raw.command, process.execPath);
    assert.deepEqual(raw.args, args);
    assert.equal(raw.cwd, cwd);
    assert.deepEqual(raw.locale, { LANG: "C", LC_ALL: "C" });
    assert.equal(result.status, 23);
    assert.equal(result.stdout, originalToString.call(stdout, "utf8"));
    assert.equal(result.stderr, originalToString.call(stderr, "utf8"));
    assert.notDeepEqual(Buffer.from(result.stdout, "utf8"), stdout);
  } finally {
    Buffer.prototype.toString = originalToString;
  }
});

await test("ENOENT saves absent streams and original own error fields before rethrow", (t) => {
  const cwd = temporary(t);
  const command = path.join(cwd, "nonexistent-owned-executable");
  const args = ["--owned-control"];
  const original = spawnSync(command, args, { cwd });
  assert.ok(original.error);
  assert.equal(Reflect.get(original.error, "code"), "ENOENT");
  const capture = recorder();
  assert.throws(
    () => rawCommand(command, args, cwd, capture.record, { stage: "missing" }),
    (error) => {
      assert.ok(error instanceof Error);
      const raw = recorded(capture.packets, "missing", "process-result");
      assert.deepEqual(raw.stdout, original.stdout === undefined ? { type: "undefined" } : null);
      assert.deepEqual(raw.stderr, original.stderr === undefined ? { type: "undefined" } : null);
      assert.equal(raw.output, original.output);
      assert.equal(raw.status, null);
      assert.equal(raw.signal, null);
      assert.equal(property(raw.error, "code").value, "ENOENT");
      assert.deepEqual(
        (raw.error as { properties: Descriptor[] }).properties.map(({ key }) => key.value),
        Object.getOwnPropertyNames(error),
      );
      for (const name of ["message", "errno", "code", "syscall", "path"]) {
        assert.equal(property(raw.error, name).value, Reflect.get(error, name));
      }
      const capturedArgs = property(raw.error, "spawnargs").value;
      args.forEach((argument, index) =>
        assert.equal(property(capturedArgs, String(index)).value, argument),
      );
      assert.deepEqual(raw.args, args);
      assert.equal(raw.command, command);
      return true;
    },
  );
});

await test("JSON error transport retains causes, cycles, symbols and descriptors without getters", () => {
  const cause = new TypeError("owned nested cause");
  const error = new Error("owned outer error", { cause });
  // Keep this control's native lazy stack separate from its intentional name accessor.
  Object.defineProperty(error, "stack", {
    value: error.stack,
    writable: true,
    enumerable: false,
    configurable: true,
  });
  const symbol = Symbol.for("vize-owned-custody-control");
  let getters = 0;
  const forbiddenGetter = () => {
    getters += 1;
    throw new Error("getter must remain inert");
  };
  Object.defineProperties(error, {
    name: { get: forbiddenGetter, enumerable: false, configurable: true },
    omittedByJson: { value: undefined, writable: false, enumerable: false },
    specialNumbers: { value: [NaN, Infinity, -Infinity, -0], enumerable: true },
  });
  Object.defineProperty(error, symbol, {
    value: 9007199254740993n,
    enumerable: false,
    writable: false,
    configurable: true,
  });
  Object.defineProperty(cause, "outer", { value: error, enumerable: false });
  const snapshot = JSON.parse(JSON.stringify(ownError(error))) as { id: number };
  assert.equal(getters, 0);
  assert.equal(property(snapshot, "message").value, "owned outer error");
  const name = property(snapshot, "name");
  assert.equal(name.enumerable, false);
  assert.equal(name.configurable, true);
  assert.equal(Object.hasOwn(name, "value"), false);
  assert.equal(name.get?.type, "function");
  assert.match(name.get?.source ?? "", /getter must remain inert/);
  const capturedCause = property(snapshot, "cause");
  assert.equal(
    capturedCause.enumerable,
    Object.getOwnPropertyDescriptor(error, "cause")!.enumerable,
  );
  assert.equal(property(capturedCause.value, "message").value, cause.message);
  assert.equal(
    (property(capturedCause.value, "outer").value as { reference: number }).reference,
    snapshot.id,
  );
  const capturedSymbol = property(snapshot, symbol);
  assert.equal(capturedSymbol.key.globalKey, Symbol.keyFor(symbol));
  assert.equal(capturedSymbol.enumerable, false);
  assert.equal(capturedSymbol.writable, false);
  assert.equal(capturedSymbol.configurable, true);
  assert.equal((capturedSymbol.value as { value: string }).value, "9007199254740993");
  assert.equal((property(snapshot, "omittedByJson").value as { type: string }).type, "undefined");
  const numbers = property(snapshot, "specialNumbers").value;
  assert.deepEqual(
    ["0", "1", "2", "3"].map(
      (index) => (property(numbers, index).value as { value: string }).value,
    ),
    ["NaN", "Infinity", "-Infinity", "-0"],
  );
});

await test("failed capture prevents an unrecorded owned subprocess from starting", (t) => {
  const cwd = temporary(t);
  const marker = path.join(cwd, "must-not-exist");
  const error = new Error("owned capture failure");
  assert.throws(
    () =>
      rawCommand(
        process.execPath,
        ["-e", `require('node:fs').writeFileSync(${JSON.stringify(marker)}, 'started')`],
        cwd,
        () => {
          throw error;
        },
        { stage: "failed-capture" },
      ),
    (thrown) => thrown === error,
  );
  assert.equal(fs.existsSync(marker), false);
});

await test(
  "primary failure packets retain malformed output and restore reference inputs",
  {
    skip: process.platform === "win32" ? "owned executable shebang controls require POSIX" : false,
  },
  async (t) => {
    await t.test("malformed JSON is saved before the parser throws", (t) => {
      const { source, stub, assertRestored, observer } = fakeFixture(t);
      const bytes = Buffer.from([123, 255, 125]);
      const cli = stub(`process.stdout.write(Buffer.from(${JSON.stringify([...bytes])}));`);
      const capture = recorder();
      const observe = observer(capture, cli);
      assert.throws(
        () => observe(source, [], PAGE_PATH, () => assert.fail("reference must not run")),
        SyntaxError,
      );
      const raw = recorded(capture.packets, "product-raw", "process-result");
      assert.deepEqual(decoded(raw.stdout), bytes);
      assert.equal(raw.command, cli);
      const failure = recorded(capture.packets, "product-failure");
      assert.ok(capture.packets.indexOf(raw) < capture.packets.indexOf(failure));
      assertRestored();
    });
    await t.test(
      "throwing transformation restores authored configuration and optional absence",
      (t) => {
        const { fixture, source, stub, assertRestored, observer } = fakeFixture(t);
        const cli = stub("process.stdout.write('{}');");
        const capture = recorder();
        const observe = observer(capture, cli);
        const failure = new Error("owned reference transform failure");
        assert.throws(
          () =>
            observe(source, [], PAGE_PATH, () => {
              const before = recorded(capture.packets, "reference-input");
              assert.deepEqual(decoded((before.inputs as Packet)[PAGE_PATH]), Buffer.from(source));
              for (const file of [
                PAGE_PATH,
                "tsconfig.json",
                "vize.config.json",
                `${PLAYGROUND}/compiler-options.json`,
                "project-root-map.d.ts",
              ])
                fixture.write(file, "changed");
              throw failure;
            }),
          (error) => error === failure,
        );
        recorded(capture.packets, "product-raw", "process-result");
        recorded(capture.packets, "reference-failure");
        assertRestored();
      },
    );
    await t.test(
      "reference spawn failure retains transformed typeof input before restoration",
      (t) => {
        const { fixture, source, stub, assertRestored, observer } = fakeFixture(t);
        const cli = stub("process.stdout.write('{}');");
        const capture = recorder();
        const missingReference = fixture.resolve("nonexistent-owned-reference");
        const observe = observer(capture, cli, missingReference);
        assert.throws(
          () => observe(source, [], PAGE_PATH, typeofReference),
          (error) => error instanceof Error && Reflect.get(error, "code") === "ENOENT",
        );
        const raw = recorded(capture.packets, "reference-raw", "process-result");
        assert.deepEqual(
          decoded((raw.inputs as Packet)[PAGE_PATH]),
          Buffer.from(typeofReference(source)),
        );
        assert.equal(raw.oracleSource, typeofReference(source));
        assert.equal(raw.command, missingReference);
        assert.deepEqual(raw.args, ["--noEmit", "--pretty", "false", "-p", "tsconfig.json"]);
        assert.equal(property(raw.error, "code").value, "ENOENT");
        assertRestored();
      },
    );
  },
);
