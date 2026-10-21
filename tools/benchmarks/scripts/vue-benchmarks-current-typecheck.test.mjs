import assert from "node:assert/strict";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  captureProcess,
  decodeCapture,
  errorEvidence,
  sha256,
} from "./vue-benchmarks-current-typecheck-capture.mjs";
import {
  assertOutside,
  materializePinnedProvider,
} from "./vue-benchmarks-current-typecheck-fixture.mjs";
import { vizeTypecheckConfig } from "./vize-typecheck-config.mjs";

await test("binary streams and failed status are retained before malformed JSON is rejected", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-current-typecheck-raw-"));
  try {
    const script =
      "process.stdout.write(Buffer.from([0,255,195,40]));process.stderr.write(Buffer.from([254,0,128]));process.exit(7)";
    const captured = captureProcess(root, "stub", process.execPath, ["-e", script], root);
    assert.equal(captured.observation.status, 7);
    assert.equal(captured.observation.signal, null);
    assert.deepEqual(readFileSync(join(root, "stub.stdout.bin")), Buffer.from([0, 255, 195, 40]));
    assert.deepEqual(readFileSync(join(root, "stub.stderr.bin")), Buffer.from([254, 0, 128]));
    assert.equal(
      captured.observation.streams.stdout.sha256,
      sha256(Buffer.from([0, 255, 195, 40])),
    );
    const decoded = decodeCapture(captured);
    assert.equal(decoded.status, 7);
    assert.throws(() => JSON.parse(decoded.stdout), SyntaxError);
    assert.equal(JSON.parse(readFileSync(join(root, "stub.json"))).status, 7);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("missing executable preserves ENOENT and null outputs without becoming a clean run", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-current-typecheck-enoent-"));
  try {
    const command = join(root, "does-not-exist");
    const captured = captureProcess(root, "missing", command, ["literal-argument"], root);
    const stored = JSON.parse(readFileSync(join(root, "missing.json")));
    assert.equal(stored.status, null);
    assert.deepEqual(stored.streams.stdout, { present: false });
    assert.deepEqual(stored.streams.stderr, { present: false });
    assert.equal(stored.command, command);
    assert.deepEqual(stored.args, ["literal-argument"]);
    const properties = stored.error.properties;
    const code = properties.find(({ key }) => key.value === "code");
    assert.equal(code.value.value, "ENOENT");
    assert.throws(() => decodeCapture(captured), /raw process error was retained/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("terminated stub retains the actual signal before parsing can proceed", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-current-typecheck-signal-"));
  try {
    const script = "process.kill(process.pid, 'SIGTERM')";
    const captured = captureProcess(root, "terminated", process.execPath, ["-e", script], root);
    const stored = JSON.parse(readFileSync(join(root, "terminated.json")));
    assert.equal(stored.status, null);
    assert.equal(stored.signal, "SIGTERM");
    assert.throws(() => decodeCapture(captured), /raw termination signal was retained/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("error evidence traverses causes/cycles and distinct symbols without invoking getters", () => {
  let getterCalls = 0;
  const first = Symbol("same");
  const second = Symbol("same");
  const cause = new Error("original cause");
  const error = new Error("failure", { cause });
  cause.parent = error;
  error[first] = -0;
  error[second] = NaN;
  Object.defineProperty(error, "sensitiveGetter", {
    get() {
      getterCalls++;
      throw new Error("called");
    },
  });
  const evidence = errorEvidence(error);
  assert.equal(getterCalls, 0);
  const symbols = evidence.properties.filter(({ key }) => key.type === "symbol");
  assert.equal(symbols.length, 2);
  assert.notEqual(symbols[0].key.id, symbols[1].key.id);
  assert.deepEqual(
    symbols.map(({ value }) => value.value),
    ["-0", "NaN"],
  );
  const causeEvidence = evidence.properties.find(({ key }) => key.value === "cause").value;
  assert.equal(
    causeEvidence.properties.find(({ key }) => key.value === "parent").value.reference,
    evidence.id,
  );
  const accessor = evidence.properties.find(({ key }) => key.value === "sensitiveGetter");
  assert.equal(accessor.enumerable, false);
  assert.equal(accessor.getter.type, "function");
  assert.equal(accessor.setter, null);
});

await test("output containment guard rejects fixture ancestors and descendants without prefix confusion", () => {
  assert.throws(() => assertOutside("/tmp/fixture/work", "/tmp/fixture"));
  assert.throws(() => assertOutside("/tmp/fixture/..cache", "/tmp/fixture"));
  assert.throws(() => assertOutside("/tmp/fixture", "/tmp/fixture"));
  assert.throws(() => assertOutside("/tmp", "/tmp/fixture"));
  assert.doesNotThrow(() => assertOutside("/tmp/fixture-other", "/tmp/fixture"));
});

await test("pinned Golar packages are materialized only inside the copied workspace", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-current-typecheck-provider-"));
  try {
    const workspace = join(root, "workspace");
    const provider = join(root, "provider", "node_modules");
    mkdirSync(workspace);
    for (const [name, relativePath] of [
      ["golar", "golar"],
      ["@golar/vue", "@golar/vue"],
    ]) {
      const directory = join(provider, relativePath);
      mkdirSync(directory, { recursive: true });
      writeFileSync(
        join(directory, "package.json"),
        `${JSON.stringify({ name, version: "0.1.10" })}\n`,
      );
    }

    const receipt = materializePinnedProvider(workspace, provider);

    assert.deepEqual(
      receipt.versions.map(({ name, version }) => [name, version]),
      [
        ["golar", "0.1.10"],
        ["@golar/vue", "0.1.10"],
      ],
    );
    for (const name of ["golar", "@golar"]) {
      const link = join(workspace, "node_modules", name);
      assert.equal(lstatSync(link).isSymbolicLink(), true);
      assert.equal(realpathSync(link), realpathSync(join(provider, name)));
    }
    assert.throws(
      () => materializePinnedProvider(workspace, provider),
      /copied workspace already has node_modules/u,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("current 154-case modes retain the strict component attribute adapter", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-current-typecheck-config-"));
  try {
    const modes = [
      {
        config: "tsconfig.json",
        translated: "tsconfig.vize-tsconfig.json",
        options: {
          checkUnknownProps: true,
          fallthroughAttributes: false,
          strictComponentAttrs: true,
        },
        artifactCases: [
          "attrs-aria-data-unknown",
          "attrs-unknown-fallthrough",
          "inherit-attrs-default-unknown",
          "inherit-attrs-false-unknown",
          "unknown-prop-strict",
        ],
      },
      {
        config: "tsconfig.fallthrough.json",
        translated: "tsconfig.vize-tsconfig.fallthrough.json",
        options: {
          checkUnknownProps: true,
          fallthroughAttributes: true,
          strictComponentAttrs: true,
        },
        artifactCases: [
          "fallthrough-mono-false-bad",
          "fallthrough-multi-bad",
          "fallthrough-multi-false-bad",
          "fallthrough-vif-both-mono-false-bad",
          "fallthrough-vif-mono-multi-bad",
          "fallthrough-vif-static-multi-bad",
        ],
      },
    ];
    for (const { config, translated, options } of modes) {
      writeFileSync(
        join(root, config),
        `${JSON.stringify({
          compilerOptions: { strict: true },
          vueCompilerOptions: {
            strictTemplates: true,
            ...(options.fallthroughAttributes ? { fallthroughAttributes: true } : {}),
          },
          include: ["cases/**/*.vue"],
        })}\n`,
      );
      const before = readFileSync(join(root, config));
      assert.equal(vizeTypecheckConfig(root, config), translated);
      assert.deepEqual(JSON.parse(readFileSync(join(root, translated))), {
        extends: `./${config}`,
        vueCompilerOptions: options,
      });
      assert.deepEqual(readFileSync(join(root, config)), before);
    }
    assert.deepEqual(
      modes.flatMap(({ artifactCases }) => artifactCases),
      [
        "attrs-aria-data-unknown",
        "attrs-unknown-fallthrough",
        "inherit-attrs-default-unknown",
        "inherit-attrs-false-unknown",
        "unknown-prop-strict",
        "fallthrough-mono-false-bad",
        "fallthrough-multi-bad",
        "fallthrough-multi-false-bad",
        "fallthrough-vif-both-mono-false-bad",
        "fallthrough-vif-mono-multi-bad",
        "fallthrough-vif-static-multi-bad",
      ],
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
