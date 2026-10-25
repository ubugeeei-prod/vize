import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { campaign } from "./css-native-attribution.mjs";
import { phaseOutputAbsent } from "./css-source-map-phases.mjs";

const script = fileURLToPath(import.meta.url);
const read = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
export function streamedDigest(file) {
  const value = createHash("sha256"),
    buffer = Buffer.allocUnsafe(1024 * 1024),
    fd = fs.openSync(file, "r");
  try {
    for (let count; (count = fs.readSync(fd, buffer)) > 0;) value.update(buffer.subarray(0, count));
  } finally {
    fs.closeSync(fd);
  }
  return value.digest("hex");
}
export function validateCompilerReceipt(row, producer) {
  assert.equal(row.source, producer.source);
  assert.equal(row.side, producer.side);
  assert.equal(row.token, producer.token);
  assert.equal(row.target, producer.target);
  assert.ok(row.outDir.startsWith(producer.target + path.sep));
  assert.equal(row.args[row.args.indexOf("--out-dir") + 1], row.outDir);
  for (const flag of ["opt-level=3", "debuginfo=2", "codegen-units=16"])
    assert.ok(
      row.args.some((arg, index) => arg === "-C" && row.args[index + 1] === flag),
      flag,
    );
  assert.equal(
    row.args.some((arg) => arg.startsWith("strip=") && arg !== "strip=none"),
    false,
  );
}
export function validateDebugIdentity(side, producer, actualHash, actualToken) {
  assert.ok(Object.hasOwn(campaign.images, side));
  assert.equal(producer.kind, "debug-source");
  assert.equal(producer.side, side);
  assert.equal(producer.source, campaign.images[side].source);
  assert.equal(producer.targetInitiallyEmpty, true);
  assert.deepEqual(producer.profile, {
    inherits: "release",
    lto: "thin",
    codegenUnits: 16,
    debug: 2,
    strip: "none",
  });
  assert.match(producer.token, /^[a-f0-9]{32}$/u);
  assert.equal(actualToken, producer.token);
  assert.equal(actualHash, producer.binarySha256);
  assert.equal(producer.emitted.sha256, actualHash);
  assert.equal(
    producer.emitted.file,
    path.join(producer.target, "x86_64-unknown-linux-gnu/ci-opt/libvize_vitrine.so"),
  );
  assert.equal(producer.debugInfo, true);
  for (const name of ["vize_vitrine", "vize_atelier_sfc", "vize_croquis", "lightningcss"]) {
    const rows = producer.compilerReceipts.filter((row) => row.crate === name);
    assert.ok(rows.length > 0, `actual ${name} compiler receipt required`);
    for (const row of rows) validateCompilerReceipt(row, producer);
  }
}
function runCommand(argv, cwd, env, log, receipt) {
  const fd = fs.openSync(log, "w");
  const command = { argv, cwd, log, startedAt: new Date().toISOString() };
  receipt.commands.push(command);
  write(receipt.file, receipt);
  console.log(JSON.stringify({ stage: "debug-command-start", argv: argv.slice(0, 5) }));
  let result;
  try {
    result = spawnSync("timeout", ["--signal=TERM", "--kill-after=5s", "1200s", ...argv], {
      cwd,
      env,
      stdio: ["ignore", fd, fd],
      shell: false,
    });
  } finally {
    fs.closeSync(fd);
  }
  Object.assign(command, {
    endedAt: new Date().toISOString(),
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    logSha256: streamedDigest(log),
  });
  write(receipt.file, receipt);
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, log);
}
function run(base, head, referenceFile, destination) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.platform, "linux");
  assert.equal(process.version, campaign.node);
  base = fs.realpathSync(base);
  head = fs.realpathSync(head);
  destination = path.resolve(destination);
  phaseOutputAbsent(destination);
  fs.mkdirSync(destination, { recursive: true });
  const harness = path.resolve(path.dirname(script), "../../.."),
    reference = read(referenceFile);
  assert.equal(
    execFileSync("git", ["rev-parse", "HEAD"], { cwd: harness, encoding: "utf8" }).trim(),
    process.env.GITHUB_SHA,
  );
  for (const file of ["Cargo.lock", "pnpm-lock.yaml"])
    assert.equal(streamedDigest(path.join(base, file)), streamedDigest(path.join(head, file)));
  const receipt = {
    schema: "vize.css.debug-source-producer",
    version: 1,
    status: "attempt",
    file: path.join(destination, "receipt.json"),
    commands: [],
    images: [],
    note: "Separate optimized debug-info images; no original release speed authority is replaced.",
  };
  write(receipt.file, receipt);
  const config = {
    kind: "debug-source",
    harnessSource: process.env.GITHUB_SHA,
    rows: reference.rows,
    images: {},
  };
  try {
    const owner = fs.mkdtempSync(
      path.join(fs.realpathSync(process.env.RUNNER_TEMP), "css-native-debug-targets-"),
    );
    for (const [side, root] of [
      ["base", base],
      ["head", head],
    ]) {
      const source = execFileSync("git", ["rev-parse", "HEAD"], {
        cwd: root,
        encoding: "utf8",
      }).trim();
      assert.equal(source, campaign.images[side].source);
      assert.equal(
        execFileSync("git", ["diff", "--name-only", "HEAD"], {
          cwd: root,
          encoding: "utf8",
        }).trim(),
        "",
      );
      const target = path.join(owner, side),
        image = path.join(destination, `${side}-image`),
        receipts = path.join(destination, `${side}-rustc`);
      phaseOutputAbsent(target);
      fs.mkdirSync(target);
      assert.deepEqual(fs.readdirSync(target), []);
      fs.mkdirSync(image);
      fs.mkdirSync(receipts);
      const token = randomBytes(16).toString("hex");
      const wrapperConfig = { side, source, root, target, token, receipts };
      const wrapperFile = path.join(destination, `${side}-wrapper-config.json`);
      write(wrapperFile, wrapperConfig);
      const wrapper = path.join(harness, "tools/benchmarks/scripts/css-native-rustc-receipt.mjs");
      fs.chmodSync(wrapper, 0o755);
      if (side === "base")
        fs.symlinkSync(
          path.join(head, "npm/native/node_modules"),
          path.join(base, "npm/native/node_modules"),
          "dir",
        );
      const producer = {
        kind: "debug-source",
        side,
        source,
        target,
        targetInitiallyEmpty: true,
        token,
        profile: { inherits: "release", lto: "thin", codegenUnits: 16, debug: 2, strip: "none" },
        toolchain: execFileSync("rustc", ["-Vv"], { cwd: root, encoding: "utf8" }),
        cargoLockSha256: streamedDigest(path.join(root, "Cargo.lock")),
        pnpmLockSha256: streamedDigest(path.join(root, "pnpm-lock.yaml")),
      };
      receipt.images.push(producer);
      write(receipt.file, receipt);
      const env = {
        ...process.env,
        CARGO_INCREMENTAL: "0",
        CARGO_TARGET_DIR: target,
        VIZE_NATIVE_BUILD_TOKEN: token,
        VIZE_CSS_RUSTC_RECEIPT_CONFIG: wrapperFile,
      };
      const flags = [
        "--locked",
        "--config",
        'profile.ci-opt.inherits="release"',
        "--config",
        'profile.ci-opt.lto="thin"',
        "--config",
        "profile.ci-opt.codegen-units=16",
        "--config",
        "profile.ci-opt.debug=2",
        "--config",
        'profile.ci-opt.strip="none"',
        "--config",
        `build.rustc-wrapper=${JSON.stringify(wrapper)}`,
      ];
      runCommand(
        [
          "vp",
          "exec",
          "napi",
          "build",
          "--platform",
          "--profile",
          "ci-opt",
          "--target-dir",
          target,
          "--manifest-path",
          "../../crates/vize_vitrine/Cargo.toml",
          "-p",
          "vize_vitrine",
          "--features",
          "napi,legacy",
          "--output-dir",
          image,
          "--no-js",
          "--",
          ...flags,
        ],
        path.join(root, "npm/native"),
        env,
        path.join(destination, `${side}-build.log`),
        receipt,
      );
      const outputs = fs.readdirSync(image).filter((name) => name.endsWith(".node"));
      assert.equal(outputs.length, 1);
      producer.binary = path.join(image, outputs[0]);
      producer.binarySha256 = streamedDigest(producer.binary);
      const emitted = fs.realpathSync(
        path.join(target, "x86_64-unknown-linux-gnu/ci-opt/libvize_vitrine.so"),
      );
      producer.emitted = { file: emitted, sha256: streamedDigest(emitted) };
      const sections = execFileSync("readelf", ["--sections", "--wide", producer.binary], {
        encoding: "utf8",
      });
      fs.writeFileSync(path.join(destination, `${side}-elf-sections.txt`), sections);
      producer.debugInfo = /\.debug_info\s/u.test(sections) && /\.debug_line\s/u.test(sections);
      producer.compilerReceipts = fs
        .readdirSync(receipts)
        .filter((name) => name.endsWith(".json"))
        .map((name) => read(path.join(receipts, name)));
      producer.declaration = {
        file: path.join(image, "index.d.ts"),
        sha256: streamedDigest(path.join(image, "index.d.ts")),
      };
      // Runtime token and all emitted flags/path checks run before any sampling.
      config.images[side] = { producer, transportBinary: producer.binary };
      write(path.join(destination, "inputs.json"), config);
      runCommand(
        [
          process.execPath,
          script,
          "--qualify",
          path.join(destination, "inputs.json"),
          referenceFile,
          side,
          path.join(destination, `${side}-whole-native.json`),
        ],
        harness,
        process.env,
        path.join(destination, `${side}-whole-native.log`),
        receipt,
      );
      write(receipt.file, receipt);
      if (side === "head") {
        fs.linkSync(producer.binary, path.join(head, "npm/native", path.basename(producer.binary)));
        fs.copyFileSync(producer.declaration.file, path.join(head, "npm/native/index.d.ts"));
        runCommand(
          [
            process.execPath,
            "--test",
            "tests/native/css-compile-source-maps.mjs",
            "tests/native/css-v-bind-source-maps.mjs",
          ],
          head,
          { ...env, VIZE_CSS_MAP_SOURCE_SHA: source, VIZE_CSS_MAP_REQUIRE_SOURCE: "1" },
          path.join(destination, "head-124-native.log"),
          receipt,
        );
        fs.cpSync(
          path.join(head, "target/css-compile-source-maps"),
          path.join(destination, "head-native-oracles"),
          { recursive: true },
        );
        runCommand(
          [
            "cargo",
            "test",
            "--profile",
            "ci-opt",
            "--locked",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--target-dir",
            target,
            "-p",
            "vize_croquis",
            "--test",
            "css_v_bind_origins",
            ...flags.slice(1),
          ],
          head,
          env,
          path.join(destination, "head-42-recorder.log"),
          receipt,
        );
      }
    }
    receipt.status = "complete";
    write(receipt.file, receipt);
  } catch (error) {
    receipt.status = "failed";
    receipt.failure = { name: error.name, message: error.message };
    write(receipt.file, receipt);
    throw error;
  }
}
async function qualify(configFile, referenceFile, side, output) {
  const { createRequire } = await import("node:module");
  const config = read(configFile),
    image = config.images[side],
    native = createRequire(import.meta.url)(image.transportBinary);
  validateDebugIdentity(
    side,
    image.producer,
    streamedDigest(image.transportBinary),
    native.getNativeBuildToken(),
  );
  const reference = read(referenceFile),
    origin = path.dirname(path.dirname(reference.images[side].transportBinary));
  const file = side === "base" ? "0-base.json" : "1-head.json";
  const expected = read(path.join(origin, file)),
    packet = { side, producer: image.producer, calls: [], status: "attempt" };
  write(output, packet);
  for (const call of expected.calls) {
    const result = native.compileCss(call.source, call.options);
    packet.calls.push({ source: call.source, options: call.options, result });
    write(output, packet);
    assert.deepEqual(result, call.result);
  }
  assert.equal(packet.calls.length, 84);
  packet.status = "complete";
  write(output, packet);
}
if (process.argv[1] && path.resolve(process.argv[1]) === script) {
  if (process.argv[2] === "--qualify") await qualify(...process.argv.slice(3));
  else run(...process.argv.slice(2));
}
