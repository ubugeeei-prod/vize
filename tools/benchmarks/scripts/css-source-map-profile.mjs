import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { buildPhaseDriver, runPhaseDriver } from "./css-source-map-phases.mjs";

const script = fileURLToPath(import.meta.url);
const digest = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const git = (root, ...args) =>
  execFileSync("git", ["-C", root, ...args], { encoding: "utf8" }).trim();
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");

export function validateCargoDelta(
  base,
  head,
  source = "b0033883518f364cfbd3dd0ab9464715e9f51758",
) {
  if (source === "3a3c8460f966fddbd8e1be4c083d460b24202f90") {
    assert.equal(
      head,
      base,
      "the binding composition campaign changes no Cargo package or dependency",
    );
    return;
  }
  assert.equal(
    source,
    "b0033883518f364cfbd3dd0ab9464715e9f51758",
    "unregistered CSS source campaign",
  );
  const marker = /(name = "vize_atelier_sfc"\n[\s\S]*? "vize_oxc_parser",\n)/u;
  const admitted = base.replace(marker, '$1 "vize_parcel_sourcemap",\n');
  assert.notEqual(admitted, base, "the exact producer package must exist");
  assert.equal(head, admitted, "only the existing source-map package dependency may be added");
}

export function validateIdentity(identity, source, side, actualToken, actualHash) {
  assert.equal(identity.source, source);
  assert.equal(identity.side, side);
  assert.match(source, /^[0-9a-f]{40}$/u);
  assert.match(identity.token, /^[0-9a-f]{32}$/u);
  assert.equal(actualToken, identity.token);
  assert.equal(actualHash, identity.binarySha256);
  assert.equal(identity.targetInitiallyEmpty, true);
}

function command(argv, cwd, env, log) {
  const fd = fs.openSync(log, "w");
  try {
    const result = spawnSync(argv[0], argv.slice(1), {
      cwd,
      env,
      stdio: ["ignore", fd, fd],
      shell: false,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, `failed ${argv[0]}; retained ${log}`);
  } finally {
    fs.closeSync(fd);
  }
}

export function profileVectors() {
  return [
    { id: "two-rules", source: ".雪😀{color:red}.b{opacity:.5}", batch: 500 },
    {
      id: "400-rules",
      source: Array.from(
        { length: 400 },
        (_, i) => `.r${i}雪😀{opacity:${i / 1000};background:url('../asset.png')}`,
      ).join("\r\n"),
      batch: 10,
    },
    {
      id: "400-bindings",
      source: Array.from(
        { length: 400 },
        (_, i) => `.r${i}雪😀{width:v-bind(width${i});opacity:${i / 1000}}`,
      ).join("\r\n"),
      batch: 10,
    },
  ];
}

export function profileModes(mode, sample = 0) {
  assert.ok(["interleaved", "isolated-off"].includes(mode));
  return mode === "isolated-off" ? [false] : sample % 2 === 0 ? [false, true] : [true, false];
}

export function validateNativeTimingPlan(packet, mode) {
  assert.equal(packet.measurementMode, mode);
  assert.deepEqual(
    packet.timings.map(({ id, source, batch }) => ({ id, source, batch })),
    profileVectors(),
  );
  for (const row of packet.timings) {
    assert.equal(row.samples.length, mode === "isolated-off" ? 20 : 40);
    for (const enabled of profileModes(mode))
      assert.equal(row.samples.filter((sample) => sample.sourceMap === enabled).length, 20);
    for (const sample of row.samples) {
      assert.equal(sample.calls, row.batch);
      assert.ok(Number.isFinite(sample.elapsedNs) && sample.elapsedNs > 0);
      assert.ok(Number.isFinite(sample.bytes) && sample.bytes > 0);
      if (mode === "isolated-off") assert.equal(sample.sourceMap, false);
    }
  }
}

function worker(identityFile, fixtureRoot, destination, mode = "interleaved") {
  const identity = JSON.parse(fs.readFileSync(identityFile, "utf8"));
  const native = createRequire(import.meta.url)(identity.binary);
  assert.ok(["base", "head"].includes(identity.side));
  const source =
    identity.side === "base"
      ? process.env.VIZE_CSS_MAP_BASE_SHA
      : process.env.VIZE_CSS_MAP_SOURCE_SHA;
  validateIdentity(
    identity,
    source,
    identity.side,
    native.getNativeBuildToken(),
    digest(identity.binary),
  );
  const packet = {
    identity,
    measurementMode: mode,
    workCounterUnits: "utf16-code-units (legacy bytes field)",
    calls: [],
    timings: [],
  };
  const fixtureModes = profileModes(mode);
  const retain = () => write(destination, packet);
  retain();
  const fixtures = ["public-0.571.0.json", "public-error-neighbor.json"].flatMap(
    (file) => JSON.parse(fs.readFileSync(path.join(fixtureRoot, file), "utf8")).captures,
  );
  for (const entry of fixtures) {
    for (const sourceMap of fixtureModes) {
      const result = native.compileCss(entry.source, { ...entry.options, sourceMap });
      packet.calls.push({ source: entry.source, options: { ...entry.options, sourceMap }, result });
      retain();
      const { map, ...preserved } = result;
      assert.deepEqual(preserved, entry.output);
      if (
        !sourceMap ||
        (identity.side === "base" &&
          process.env.VIZE_CSS_MAP_BASE_SHA === "b0033883518f364cfbd3dd0ab9464715e9f51758")
      )
        assert.equal(map, undefined);
      if (
        entry.id === "v-bind" &&
        sourceMap &&
        process.env.VIZE_CSS_MAP_BASE_SHA === "3a3c8460f966fddbd8e1be4c083d460b24202f90"
      ) {
        if (identity.side === "base") assert.equal(map, undefined);
        else assert.equal(typeof map, "string");
      }
    }
  }
  const vectors = profileVectors();
  for (const vector of vectors) {
    const row = { ...vector, samples: [] };
    packet.timings.push(row);
    const off = { filename: "profile-雪😀.css", minify: false, sourceMap: false };
    const on = { ...off, sourceMap: true };
    const plain = native.compileCss(vector.source, off);
    row.outputs = { off: plain };
    if (mode === "interleaved") {
      const mapped = native.compileCss(vector.source, on);
      row.outputs.on = mapped;
      assert.deepEqual((({ map: _, ...rest }) => rest)(mapped), plain);
    }
    assert.equal(plain.map, undefined);
    for (let warm = 0; warm < 100; warm++) {
      for (const sourceMap of fixtureModes) native.compileCss(vector.source, sourceMap ? on : off);
    }
    for (let sample = 0; sample < 20; sample++) {
      for (const sourceMap of profileModes(mode, sample)) {
        const options = sourceMap ? on : off;
        const start = process.hrtime.bigint();
        let bytes = 0;
        for (let call = 0; call < vector.batch; call++) {
          const result = native.compileCss(vector.source, options);
          bytes += result.code.length + (result.map?.length ?? 0);
        }
        const elapsedNs = Number(process.hrtime.bigint() - start);
        row.samples.push({ sourceMap: options.sourceMap, calls: vector.batch, elapsedNs, bytes });
      }
    }
    retain();
  }
}

function run(base, head, destination) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.platform, "linux");
  base = fs.realpathSync(base);
  head = fs.realpathSync(head);
  destination = path.resolve(destination);
  fs.mkdirSync(destination, { recursive: true });
  const sources = { base: git(base, "rev-parse", "HEAD"), head: git(head, "rev-parse", "HEAD") };
  assert.equal(sources.base, process.env.VIZE_CSS_MAP_BASE_SHA);
  assert.equal(sources.head, process.env.VIZE_CSS_MAP_SOURCE_SHA);
  for (const source of Object.values(sources)) assert.match(source, /^[0-9a-f]{40}$/u);
  execFileSync("git", ["-C", head, "merge-base", "--is-ancestor", sources.base, sources.head]);
  validateCargoDelta(
    fs.readFileSync(path.join(base, "Cargo.lock"), "utf8"),
    fs.readFileSync(path.join(head, "Cargo.lock"), "utf8"),
    sources.base,
  );
  assert.equal(
    digest(path.join(base, "pnpm-lock.yaml")),
    digest(path.join(head, "pnpm-lock.yaml")),
  );
  const targetOwner = fs.mkdtempSync(
    path.join(fs.realpathSync(process.env.RUNNER_TEMP), "css-source-map-targets-"),
  );
  const receipt = {
    schema: "vize.css.source-map.source-profile",
    version: 1,
    sources,
    driverSha256: digest(script),
    builds: [],
    order: ["base", "head", "head", "base"],
    note: "Report-only optimized source comparison and map-off/on overhead; no instruction-ceiling or universal performance claim.",
    toolchain: execFileSync("rustc", ["-Vv"], { encoding: "utf8" }),
    node: process.version,
  };
  const retain = () => write(path.join(destination, "receipt.json"), receipt);
  retain();
  for (const [side, root] of [
    ["base", base],
    ["head", head],
  ]) {
    assert.equal(git(root, "diff", "--name-only", "HEAD"), "");
    const source = sources[side],
      token = randomBytes(16).toString("hex");
    const target = path.join(targetOwner, `${side}-${source}`);
    assert.equal(fs.existsSync(target), false);
    fs.mkdirSync(target);
    assert.deepEqual(fs.readdirSync(target), []);
    const nativeDir = path.join(root, "npm/native");
    const imageDir = path.join(destination, `${side}-image`);
    assert.equal(fs.existsSync(imageDir), false);
    fs.mkdirSync(imageDir);
    if (side === "base") {
      assert.equal(fs.existsSync(path.join(nativeDir, "node_modules")), false);
      fs.symlinkSync(
        path.join(head, "npm/native/node_modules"),
        path.join(nativeDir, "node_modules"),
        "dir",
      );
    }
    const argv = [
      "vp",
      "exec",
      "napi",
      "build",
      "--platform",
      "--profile",
      "ci-opt",
      "--manifest-path",
      "../../crates/vize_vitrine/Cargo.toml",
      "-p",
      "vize_vitrine",
      "--features",
      "napi,legacy",
      "--output-dir",
      imageDir,
      "--no-js",
      "--",
      "--locked",
      "--config",
      'profile.ci-opt.inherits="release"',
      "--config",
      'profile.ci-opt.lto="thin"',
      "--config",
      "profile.ci-opt.codegen-units=16",
    ];
    const identity = {
      side,
      source,
      token,
      target,
      targetInitiallyEmpty: true,
      command: argv,
      sourceTrees: Object.fromEntries(
        ["crates", "davinci", "vendor", "npm/native"].map((entry) => [
          entry,
          git(root, "rev-parse", `HEAD:${entry}`),
        ]),
      ),
      cargoLockSha256: digest(path.join(root, "Cargo.lock")),
    };
    receipt.builds.push(identity);
    retain();
    command(
      argv,
      nativeDir,
      { ...process.env, CARGO_TARGET_DIR: target, VIZE_NATIVE_BUILD_TOKEN: token },
      path.join(destination, `${side}-build.log`),
    );
    assert.equal(git(root, "diff", "--name-only", "HEAD"), "");
    const binaries = fs.readdirSync(imageDir).filter((name) => name.endsWith(".node"));
    assert.equal(binaries.length, 1);
    identity.binary = path.join(imageDir, binaries[0]);
    identity.binarySha256 = digest(identity.binary);
    const declaration = path.join(imageDir, "index.d.ts");
    identity.declaration = { file: declaration, sha256: digest(declaration) };
    write(path.join(destination, `${side}-identity.json`), identity);
    retain();
  }
  const isolatedPackets = [];
  for (const [index, side] of receipt.order.entries()) {
    const file = path.join(destination, `isolated-${index}-${side}.json`);
    command(
      [
        process.execPath,
        script,
        "--off-only-worker",
        path.join(destination, `${side}-identity.json`),
        path.join(head, "tests/fixtures/css/compile-source-maps"),
        file,
      ],
      head,
      process.env,
      path.join(destination, `isolated-${index}-${side}.log`),
    );
    const packet = JSON.parse(fs.readFileSync(file, "utf8"));
    validateNativeTimingPlan(packet, "isolated-off");
    assert.deepEqual(
      packet.identity,
      receipt.builds.find((image) => image.side === side),
    );
    assert.equal(packet.calls.length, 42);
    for (const call of packet.calls) {
      assert.equal(call.options.sourceMap, false);
      assert.equal(call.result.map, undefined);
    }
    isolatedPackets.push(packet);
  }
  for (const packet of isolatedPackets)
    assert.deepEqual(
      packet.calls,
      isolatedPackets[0].calls,
      "all isolated off-only whole packets must remain exact",
    );
  receipt.isolatedOffSummary = profileVectors().map((vector) => ({
    id: vector.id,
    results: ["base", "head"].map((side) => {
      const samples = isolatedPackets
        .filter((packet) => packet.identity.side === side)
        .flatMap((packet) =>
          packet.timings
            .find((row) => row.id === vector.id)
            .samples.map((sample) => {
              assert.equal(sample.sourceMap, false);
              return sample.elapsedNs / sample.calls;
            }),
        )
        .sort((a, b) => a - b);
      assert.equal(samples.length, 40);
      return {
        side,
        samples,
        p50Ns: samples[Math.floor(samples.length / 2)],
        p95Ns: samples[Math.floor(samples.length * 0.95)],
      };
    }),
  }));
  receipt.isolatedOffCompleted = true;
  retain();
  console.log(
    JSON.stringify({ stage: "isolated-off-native-complete", summary: receipt.isolatedOffSummary }),
  );
  const packets = [];
  for (const [index, side] of receipt.order.entries()) {
    const file = path.join(destination, `${index}-${side}.json`);
    command(
      [
        process.execPath,
        script,
        "--worker",
        path.join(destination, `${side}-identity.json`),
        path.join(head, "tests/fixtures/css/compile-source-maps"),
        file,
      ],
      head,
      process.env,
      path.join(destination, `${index}-${side}.log`),
    );
    const packet = JSON.parse(fs.readFileSync(file, "utf8"));
    validateNativeTimingPlan(packet, "interleaved");
    assert.deepEqual(
      packet.identity,
      receipt.builds.find((image) => image.side === side),
    );
    packets.push(packet);
  }

  for (const packet of packets) {
    assert.equal(packet.calls.length, 84);
    assert.deepEqual(
      packet.calls.map(({ source, options, result }) => ({
        source,
        options,
        result: (({ map: _, ...rest }) => rest)(result),
      })),
      packets[0].calls.map(({ source, options, result }) => ({
        source,
        options,
        result: (({ map: _, ...rest }) => rest)(result),
      })),
    );
  }
  for (const side of ["base", "head"]) {
    const same = packets.filter((packet) => packet.identity.side === side);
    assert.deepEqual(
      same[0].calls,
      same[1].calls,
      "repeat processes must preserve every whole result, including maps",
    );
  }
  assert.equal(
    receipt.builds[0].declaration.sha256,
    receipt.builds[1].declaration.sha256,
    "native public declarations must remain exact",
  );
  const summary = packets[0].timings.map((vector) => ({
    id: vector.id,
    results: ["base", "head"].flatMap((side) =>
      [false, true].map((sourceMap) => {
        const samples = packets
          .filter((packet) => packet.identity.side === side)
          .flatMap((packet) =>
            packet.timings
              .find((row) => row.id === vector.id)
              .samples.filter((sample) => sample.sourceMap === sourceMap)
              .map((sample) => sample.elapsedNs / sample.calls),
          )
          .sort((a, b) => a - b);
        return {
          side,
          sourceMap,
          samples,
          p50Ns: samples[Math.floor(samples.length / 2)],
          p95Ns: samples[Math.floor(samples.length * 0.95)],
        };
      }),
    ),
  }));
  for (const [index, packet] of packets.entries())
    for (const vector of packet.timings)
      assert.deepEqual(
        vector.outputs.off,
        isolatedPackets[index].timings.find((row) => row.id === vector.id).outputs.off,
      );
  receipt.summary = summary;
  receipt.nativeCompleted = true;
  retain();
  console.log(JSON.stringify({ stage: "interleaved-native-complete", summary }));
  const phaseDrivers = new Map(),
    phasePackets = [];
  if (sources.base === "3a3c8460f966fddbd8e1be4c083d460b24202f90") {
    const inputs = path.join(destination, "phase-inputs.json");
    write(inputs, { filename: "profile-雪😀.css", vectors: profileVectors() });
    for (const image of receipt.builds)
      phaseDrivers.set(
        image.side,
        buildPhaseDriver({
          image,
          root: head,
          producerRoot: image.side === "base" ? base : head,
          destination,
          inputs,
          receipt,
        }),
      );
  }
  for (const [index, side] of receipt.order.entries()) {
    const packet = packets[index];
    if (phaseDrivers.has(side)) {
      const phases = runPhaseDriver(phaseDrivers.get(side), destination, index);
      for (const row of phases.vectors) {
        const native = packet.timings.find((vector) => vector.id === row.id);
        assert.deepEqual(
          row.outputs.fullCompile,
          native.outputs.off,
          "isolated whole Rust output must equal actual same-source NAPI output",
        );
        assert.equal(row.source, native.source);
      }
      phasePackets.push({ side, packet: phases });
    }
  }
  if (phasePackets.length) {
    assert.equal(phasePackets.length, 4);
    const values = (packet) => packet.vectors.map(({ samples: _, ...row }) => row);
    for (const entry of phasePackets)
      assert.deepEqual(
        values(entry.packet),
        values(phasePackets[0].packet),
        "all full phase inputs/results and logical work must remain exact across both roles/repeats",
      );
    receipt.phaseSummary = profileVectors().map((vector) => ({
      id: vector.id,
      results: ["base", "head"].flatMap((side) =>
        ["scanner", "processed-input-compile", "full-compile"].map((phase) => {
          const samples = phasePackets
            .filter((entry) => entry.side === side)
            .flatMap((entry) =>
              entry.packet.vectors
                .find((row) => row.id === vector.id)
                .samples.filter((sample) => sample.phase === phase)
                .map((sample) => sample.elapsedNs / sample.calls),
            )
            .sort((a, b) => a - b);
          assert.equal(samples.length, 40);
          return {
            side,
            phase,
            samples,
            p50Ns: samples[Math.floor(samples.length / 2)],
            p95Ns: samples[Math.floor(samples.length * 0.95)],
          };
        }),
      ),
    }));
  }
  receipt.summary = summary;
  receipt.completed = true;
  retain();
  console.log(JSON.stringify(summary, null, 2));
}

if (process.argv[1] && path.resolve(process.argv[1]) === script) {
  if (process.argv[2] === "--worker") worker(...process.argv.slice(3));
  else if (process.argv[2] === "--off-only-worker")
    worker(...process.argv.slice(3), "isolated-off");
  else run(...process.argv.slice(2));
}
