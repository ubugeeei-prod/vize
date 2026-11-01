import assert from "node:assert/strict";
import childProcess from "node:child_process";
import fs from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import {
  nativeHistoryToolImage,
  prepareNativeHistoryToolchain,
  validateNativeHistoryToolchain,
} from "../../npm/native/scripts/formatter-history-toolchain.mts";

// Pure filesystem/process-admission controls. The authored ELF headers and version
// replies below are never executed, compiled, loaded, or credited as native evidence.
function fixture(context: TestContext) {
  const root = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "native-toolchain-admission-")),
  );
  context.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const nativeDir = path.join(root, "npm/native");
  fs.mkdirSync(nativeDir, { recursive: true });
  const bin = path.join(root, "installed/toolchain/bin");
  const proxy = path.join(root, "installed/proxies");
  fs.mkdirSync(bin, { recursive: true });
  fs.mkdirSync(proxy, { recursive: true });
  function image(file: string, label: string) {
    fs.writeFileSync(
      file,
      Buffer.concat([Buffer.from([0x7f, 0x45, 0x4c, 0x46]), Buffer.from(label)]),
      { mode: 0o755 },
    );
    return file;
  }
  const rustup = image(path.join(proxy, "rustup"), "authored rustup image; never executed");
  const rustc = image(path.join(bin, "rustc"), "authored rustc image; never executed");
  const cargo = image(path.join(bin, "cargo"), "authored cargo image; never executed");
  fs.symlinkSync(rustup, path.join(proxy, "rustc"));
  fs.symlinkSync(rustup, path.join(proxy, "cargo"));
  const environment = { PATH: proxy, RUSTUP_TOOLCHAIN: "authored-toolchain" };
  const replies = {
    rustc:
      "rustc authored fixture\nbinary: rustc\ncommit-hash: entire-authored-hash\nhost: authored-host\n",
    cargo: "cargo authored fixture (entire identity)\n",
  };
  const calls: { file: string; args: string[]; cwd: string; environment: NodeJS.ProcessEnv }[] = [];
  context.mock.method(
    childProcess,
    "spawnSync",
    (
      file: string,
      args: string[],
      options: {
        cwd: string;
        env: NodeJS.ProcessEnv;
      },
    ) => {
      assert.equal(options.cwd, fs.realpathSync(nativeDir));
      calls.push({ file, args, cwd: options.cwd, environment: options.env });
      let stdout: Buffer;
      if (file === rustup) {
        assert.equal(args[0], "which");
        assert(args[1] === "rustc" || args[1] === "cargo");
        stdout = Buffer.from(`${args[1] === "rustc" ? rustc : cargo}\n`);
      } else {
        assert(
          file === rustc || file === cargo,
          "pure controls must never execute another provider",
        );
        assert.deepEqual(args, file === rustc ? ["-Vv"] : ["-V"]);
        assert.equal(options.env.RUSTC, rustc);
        assert.equal(options.env.CARGO, cargo);
        assert.equal(options.env.PATH, `${bin}${path.delimiter}${proxy}`);
        stdout = Buffer.from(file === rustc ? replies.rustc : replies.cargo);
      }
      return { pid: 1, output: [], stdout, stderr: Buffer.alloc(0), status: 0, signal: null };
    },
  );
  syncBuiltinESMExports();
  context.after(() => {
    context.mock.restoreAll();
    syncBuiltinESMExports();
  });
  const directory = path.join(nativeDir, ".artifacts/native/formatter-history/toolchain");
  function packets() {
    return fs.readdirSync(directory).map((name) => ({
      name,
      body: JSON.parse(fs.readFileSync(path.join(directory, name), "utf8")) as Record<
        string,
        unknown
      >,
    }));
  }
  return {
    root,
    nativeDir,
    bin,
    proxy,
    image,
    rustup,
    rustc,
    cargo,
    environment,
    replies,
    calls,
    packets,
  };
}

void test("real toolchain admission binds producer cwd, actual NAPI/metadata environment and whole versions", (context) => {
  const f = fixture(context);
  const selected = prepareNativeHistoryToolchain(f.nativeDir, f.environment);
  assert.deepEqual(selected.environment, {
    ...f.environment,
    RUSTC: f.rustc,
    CARGO: f.cargo,
    PATH: `${f.bin}${path.delimiter}${f.proxy}`,
  });
  assert.equal(selected.provider.requested.rustc.realpath, f.rustup);
  assert.equal(selected.provider.requested.rustc.links.length, 1);
  assert.equal(
    selected.provider.before.rustc.sha256,
    nativeHistoryToolImage(f.rustc, f.nativeDir).sha256,
  );
  const after = validateNativeHistoryToolchain(
    f.nativeDir,
    selected.provider,
    "producer-after-build",
  );
  const consumer = validateNativeHistoryToolchain(f.nativeDir, selected.provider, "consumer");
  assert.equal(after.rustc, f.replies.rustc);
  assert.equal(after.cargo, f.replies.cargo);
  assert.deepEqual(after.observation.images, consumer.observation.images);
  assert.deepEqual(after.observation.versions, consumer.observation.versions);
  assert(f.packets().every(({ body }) => body.qualified === true));
  assert.equal(f.calls.length, 8, "two selections and three pairs of real-image version requests");
});

void test("custom wrapper selections fail closed and retain the actual requested environment", (context) => {
  const f = fixture(context);
  for (const key of ["RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "LD_PRELOAD"] as const) {
    assert.throws(
      () =>
        prepareNativeHistoryToolchain(f.nativeDir, {
          ...f.environment,
          [key]: "/authored/unsupported-wrapper",
        }),
      /unqualified toolchain wrapper/,
    );
    const packet = f.packets().find(({ name }) => name === "selection.json")?.body;
    assert(packet);
    assert.equal((packet.ambient as Record<string, unknown>)[key], "/authored/unsupported-wrapper");
    assert(packet.error);
  }
  assert.equal(f.calls.length, 0);
});

void test("a same-reply custom executable cannot replace the real selected compiler or Cargo", (context) => {
  const f = fixture(context);
  const other = f.image(path.join(f.root, "other"), "different authored ELF provider");
  for (const key of ["RUSTC", "CARGO"] as const) {
    assert.throws(
      () => prepareNativeHistoryToolchain(f.nativeDir, { ...f.environment, [key]: other }),
      /neither the selected real image nor the authenticated rustup proxy/,
    );
    assert(f.packets().find(({ name }) => name === "selection.json")?.body.error);
    assert.throws(
      () =>
        prepareNativeHistoryToolchain(f.nativeDir, {
          ...f.environment,
          [key]: f.rustup,
        }),
      /neither the selected real image nor the authenticated rustup proxy/,
    );
  }
  assert(
    f.calls.every(({ file }) => file === f.rustup),
    "unqualified providers are never launched",
  );
});

void test("changed physical bytes and symlink targets are refused before any new version execution", (context) => {
  const f = fixture(context);
  const selected = prepareNativeHistoryToolchain(f.nativeDir, f.environment);
  const old = fs.readFileSync(f.rustc);
  const count = f.calls.length;
  fs.appendFileSync(f.rustc, "changed physical bytes");
  assert.throws(
    () => validateNativeHistoryToolchain(f.nativeDir, selected.provider, "consumer"),
    /physical rustc changed/,
  );
  assert.equal(f.calls.length, count);
  assert(f.packets().some(({ body }) => body.images && body.error));
  fs.writeFileSync(f.rustc, old);
  fs.unlinkSync(path.join(f.proxy, "rustc"));
  fs.symlinkSync(f.cargo, path.join(f.proxy, "rustc"));
  assert.throws(() => validateNativeHistoryToolchain(f.nativeDir, selected.provider, "consumer"));
  assert.equal(f.calls.length, count);
});

void test("whole version drift is retained before rejection instead of accepting a matching prefix", (context) => {
  const f = fixture(context);
  const selected = prepareNativeHistoryToolchain(f.nativeDir, f.environment);
  f.replies.rustc = f.replies.rustc.replace(
    "entire-authored-hash",
    "different-commit-with-same-first-line",
  );
  assert.throws(
    () => validateNativeHistoryToolchain(f.nativeDir, selected.provider, "consumer"),
    /full rustc version changed/,
  );
  const failed = f.packets().find(({ body }) => body.error && body.versions)?.body;
  assert(failed);
  const versions = failed.versions as { rustc: { stdout: { base64: string } } };
  assert.equal(Buffer.from(versions.rustc.stdout.base64, "base64").toString(), f.replies.rustc);
});

void test("malformed streams, wrong selection and changed build environment never qualify", (context) => {
  const f = fixture(context);
  const selected = prepareNativeHistoryToolchain(f.nativeDir, f.environment);
  const mutations = [
    (value: typeof selected.provider) => {
      value.versions.rustc.stdout.sha256 = "0".repeat(64);
    },
    (value: typeof selected.provider) => {
      value.selection.cargo.exitStatus = 1;
    },
    (value: typeof selected.provider) => {
      value.selection.rustc.signal = "SIGABRT";
    },
    (value: typeof selected.provider) => {
      value.buildEnvironment.RUSTC = f.rustup;
    },
    (value: typeof selected.provider) => {
      value.buildEnvironment.PATH = f.proxy;
    },
    (value: typeof selected.provider) => {
      value.before.cargo.sha256 = "0".repeat(64);
    },
  ];
  for (const mutate of mutations) {
    const changed = structuredClone(selected.provider);
    mutate(changed);
    assert.throws(() => validateNativeHistoryToolchain(f.nativeDir, changed, "consumer"));
  }
  assert.equal(
    validateNativeHistoryToolchain(f.nativeDir, selected.provider, "consumer").rustc,
    f.replies.rustc,
  );
});

void test("a text wrapper with executable permission is retained but cannot qualify as a real image", (context) => {
  const f = fixture(context);
  const script = path.join(f.root, "authored-wrapper");
  fs.writeFileSync(script, "#!/bin/sh\nexit 0\n", { mode: 0o755 });
  assert.throws(
    () => prepareNativeHistoryToolchain(f.nativeDir, { ...f.environment, RUSTC: script }),
    /real Linux ELF toolchain image/,
  );
  assert.equal(f.calls.length, 0);
  assert(f.packets().find(({ name }) => name === "selection.json")?.body.error);
});
