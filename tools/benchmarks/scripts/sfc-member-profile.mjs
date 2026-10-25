import assert from "node:assert/strict";
import { createHash, randomBytes } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync, execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { percentiles, memberCases, controlCases } from "./sfc-member-fixtures.mjs";

export function comparePackets(left, right) {
  assert.deepEqual(
    left.rows.map((row) => row.id),
    memberCases().map((row) => row.id),
  );
  assert.deepEqual(
    right.rows.map((row) => row.id),
    left.rows.map((row) => row.id),
  );
  assert.deepEqual(
    left.controls.map((row) => row.id),
    controlCases().map((row) => row.id),
  );
  assert.deepEqual(
    right.controls.map((row) => row.id),
    left.controls.map((row) => row.id),
  );
  for (const [index, row] of left.rows.entries()) {
    const other = right.rows[index];
    assert.equal(other.source, row.source, row.id + " source drift");
    assert.equal(other.sourceSha256, row.sourceSha256, row.id + " source hash drift");
    assert.equal(row.falseProfileBytes, JSON.stringify(row.falseProfile));
    assert.equal(other.falseProfileBytes, JSON.stringify(other.falseProfile));
    assert.equal(other.falseProfileBytes, row.falseProfileBytes, row.id + " whole metadata drift");
  }
  for (const [index, row] of left.controls.entries()) {
    const other = right.controls[index];
    assert.equal(other.source, row.source);
    assert.equal(row.falseProfileBytes, JSON.stringify(row.falseProfile));
    assert.equal(other.falseProfileBytes, JSON.stringify(other.falseProfile));
    assert.equal(
      other.falseProfileBytes,
      row.falseProfileBytes,
      row.id + " refusal/cooked metadata drift",
    );
  }
}
const git = (root, ...args) =>
  execFileSync("git", ["-C", root, ...args], { encoding: "utf8" }).trim();
const digest = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
export function buildTarget(root, sha, side) {
  assert.match(sha, /^[a-f0-9]{40}$/);
  assert.ok(["base", "head"].includes(side));
  assert.ok(path.isAbsolute(root));
  return path.join(root, `${side}-${sha}`);
}
function command(args, cwd, env, log) {
  const fd = fs.openSync(log, "w");
  try {
    const value = spawnSync(args[0], args.slice(1), { cwd, env, stdio: ["ignore", fd, fd] });
    assert.equal(value.error, undefined);
    assert.equal(value.status, 0, `${args[0]} failed; see ${log}`);
    assert.equal(value.signal, null);
  } finally {
    fs.closeSync(fd);
  }
}
export function runProfile(base, head, output) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.platform, "linux");
  base = fs.realpathSync(base);
  head = fs.realpathSync(head);
  output = path.resolve(output);
  fs.mkdirSync(output, { recursive: true });
  const baseSha = git(base, "rev-parse", "HEAD"),
    headSha = git(head, "rev-parse", "HEAD");
  for (const sha of [baseSha, headSha]) assert.match(sha, /^[a-f0-9]{40}$/);
  assert.equal(baseSha, process.env.MEMBER_BASE_SHA);
  assert.equal(headSha, process.env.MEMBER_HEAD_SHA);
  assert.equal(headSha, process.env.GITHUB_SHA);
  execFileSync("git", ["-C", head, "merge-base", "--is-ancestor", baseSha, headSha]);
  for (const file of ["Cargo.lock", "pnpm-lock.yaml"])
    assert.equal(digest(path.join(base, file)), digest(path.join(head, file)), file + " differs");
  // Never restore or share compiled dependencies: Cargo freshness across two
  // checkouts is not proof that an rlib belongs to the requested source.
  assert.ok(process.env.RUNNER_TEMP && path.isAbsolute(process.env.RUNNER_TEMP));
  const targets = fs.mkdtempSync(
    path.join(fs.realpathSync(process.env.RUNNER_TEMP), "sfc-members-"),
  );
  const productionDiff = git(
    head,
    "diff",
    "--name-only",
    baseSha,
    headSha,
    "--",
    "crates",
    "davinci",
    "vendor",
    "Cargo.toml",
    "Cargo.lock",
    "npm/native",
  );
  const receipt = {
    baseSha,
    headSha,
    productionIdentical: productionDiff === "",
    productionDiff: productionDiff.split("\n").filter(Boolean),
    builds: [],
    order: ["base", "head", "head", "base"],
    profile: "ci-opt: opt3/thinLTO/cgu16",
    note: "report-only optimized source evidence; no normal CI or ceiling qualification",
    toolchain: execFileSync("rustc", ["-Vv"], { encoding: "utf8" }),
    node: process.version,
    driverSha256: digest(fileURLToPath(import.meta.url)),
    fixtureSha256: digest(fileURLToPath(new URL("./sfc-member-fixtures.mjs", import.meta.url))),
  };
  const retain = () =>
    fs.writeFileSync(path.join(output, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
  retain();
  try {
    for (const [side, root] of [
      ["base", base],
      ["head", head],
    ]) {
      assert.equal(git(root, "diff", "--name-only", "HEAD"), "", "source dirty before build");
      const native = path.join(root, "npm/native"),
        token = randomBytes(16).toString("hex");
      const sha = side === "base" ? baseSha : headSha;
      const target = buildTarget(targets, sha, side);
      assert.equal(fs.existsSync(target), false, "compiled target must start absent");
      fs.mkdirSync(target);
      assert.deepEqual(fs.readdirSync(target), []);
      if (side === "base") {
        const modules = path.join(native, "node_modules");
        assert.equal(fs.existsSync(modules), false);
        fs.symlinkSync(path.join(head, "npm/native/node_modules"), modules, "dir");
      }
      const args = [
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
        ".",
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
        root,
        sha,
        token,
        target,
        targetInitiallyEmpty: true,
        sourceTrees: Object.fromEntries(
          ["crates", "davinci", "vendor", "npm/native"].map((entry) => [
            entry,
            git(root, "rev-parse", `HEAD:${entry}`),
          ]),
        ),
        cargoLockSha256: digest(path.join(root, "Cargo.lock")),
        command: args,
      };
      receipt.builds.push(identity);
      retain();
      command(
        args,
        native,
        { ...process.env, CARGO_TARGET_DIR: target, VIZE_NATIVE_BUILD_TOKEN: token },
        path.join(output, side + "-build.log"),
      );
      const binaries = fs.readdirSync(native).filter((file) => file.endsWith(".node"));
      assert.equal(binaries.length, 1);
      identity.binary = path.join(native, binaries[0]);
      identity.binarySha256 = digest(identity.binary);
      const changed = git(root, "diff", "--name-only", "HEAD").split("\n").filter(Boolean);
      assert.ok(
        changed.every((file) => ["npm/native/index.js", "npm/native/index.d.ts"].includes(file)),
        "authored source changed while building",
      );
      fs.writeFileSync(
        path.join(output, side + "-identity.json"),
        JSON.stringify(identity, null, 2) + "\n",
      );
      retain();
    }
    const packets = [];
    for (const [index, side] of receipt.order.entries()) {
      const filename = path.join(output, `${index}-${side}.json`);
      command(
        [
          process.execPath,
          fileURLToPath(new URL("./sfc-member-worker.mjs", import.meta.url)),
          side === "base" ? base : head,
          filename,
          path.join(output, side + "-identity.json"),
        ],
        head,
        process.env,
        path.join(output, `${index}-${side}.log`),
      );
      packets.push(JSON.parse(fs.readFileSync(filename, "utf8")));
    }
    for (const packet of packets.slice(1)) comparePackets(packets[0], packet);
    const summary = memberCases().map((vector, index) =>
      Object.fromEntries([
        ["id", vector.id],
        ...["base", "head"].map((side) => {
          const selected = packets
            .filter((packet) => packet.identity.side === side)
            .map((packet) => packet.rows[index]);
          const profiles = selected.flatMap((row) => row.coldProfiles);
          return [
            side,
            {
              literalParserCalls: [
                ...new Set(profiles.map((profile) => profile.literalMemberParseCalls)),
              ],
              analyzeNs: percentiles(profiles.map((profile) => profile.analyzeNs)),
              coldWallMs: selected.map((row) => row.coldWallMs),
              warmWallMs: selected.map((row) => row.warmWallMs),
            },
          ];
        }),
      ]),
    );
    fs.writeFileSync(
      path.join(output, "summary.json"),
      JSON.stringify({ baseSha, headSha, summary, metadataByteEqual: true }, null, 2) + "\n",
    );
    receipt.complete = true;
    receipt.metadataByteEqual = true;
    retain();
    console.log(
      JSON.stringify({
        baseSha,
        headSha,
        productionIdentical: receipt.productionIdentical,
        summary,
      }),
    );
  } catch (error) {
    receipt.error = { message: error.message, stack: error.stack };
    retain();
    throw error;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 5);
  runProfile(...process.argv.slice(2));
}
