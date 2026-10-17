import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { rustSmokeInvocation } from "../support/release-smoke-reference.ts";
import { runCommand } from "../../../tools/support/release/smoke_release_install/process.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
// Immutable public product fixture, independent of the next workspace version.
// This qualifies the installer migration, not a new release's public acceptance.
const version = "0.457.0";
const targets: Record<string, string> = {
  "linux/x64": "linux-x64-gnu",
  "linux/arm64": "linux-arm64-gnu",
  "darwin/x64": "darwin-x64",
  "darwin/arm64": "darwin-arm64",
  "win32/x64": "win32-x64-msvc",
  "win32/arm64": "win32-arm64-msvc",
};
function object(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}

test(
  "Rust and TypeScript smoke the same fresh public native image and compiled SFC",
  { timeout: 240_000 },
  async () => {
    const temporary = fs.realpathSync.native(
      fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-smoke-")),
    );
    const retained: string[] = [];
    try {
      const target = targets[`${process.platform}/${process.arch}`];
      assert.ok(target, "public native fixture requires a release target");
      const names = ["@vizejs/native", `@vizejs/native-${target}`];
      const packages: string[] = [];
      const sourceImages: { sha256: string; bytes: number }[] = [];
      const evidence: unknown[] = [];
      for (const [index, name] of names.entries()) {
        const response = await fetch(
          `https://registry.npmjs.org/${encodeURIComponent(name)}/${version}`,
          { signal: AbortSignal.timeout(30_000) },
        );
        assert.equal(response.status, 200);
        const metadata = object(await response.json());
        assert.equal(metadata.name, name);
        assert.equal(metadata.version, version);
        const dist = object(metadata.dist);
        assert.ok(typeof dist.integrity === "string" && dist.integrity.startsWith("sha512-"));
        const pack = JSON.parse(
          runCommand(
            process.platform === "win32" ? "npm.cmd" : "npm",
            [
              "pack",
              `${name}@${version}`,
              "--json",
              "--ignore-scripts",
              "--pack-destination",
              temporary,
            ],
            temporary,
          ),
        ) as unknown;
        assert.ok(Array.isArray(pack) && pack.length === 1);
        const entry = object(pack[0]);
        assert.ok(typeof entry.filename === "string");
        const archive = path.join(temporary, entry.filename),
          bytes = fs.readFileSync(archive);
        assert.equal(
          `sha512-${createHash("sha512").update(bytes).digest("base64")}`,
          dist.integrity,
        );
        const directory = path.join(temporary, `package-${index}`);
        fs.mkdirSync(directory);
        // Git for Windows supplies GNU tar, whose absolute C: archive syntax
        // denotes a remote host. Both paths stay relative to this fresh cwd;
        // extraction still uses the exact independently SRI-verified archive.
        runCommand(
          "tar",
          ["-xzf", entry.filename, "--strip-components=1", "-C", path.basename(directory)],
          temporary,
        );
        const manifest = object(
          JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8")),
        );
        assert.equal(manifest.name, name);
        assert.equal(manifest.version, version);
        for (const filename of fs.readdirSync(directory).filter((file) => file.endsWith(".node"))) {
          const body = fs.readFileSync(path.join(directory, filename));
          sourceImages.push({
            sha256: createHash("sha256").update(body).digest("hex"),
            bytes: body.length,
          });
        }
        evidence.push({
          name,
          version,
          integrity: dist.integrity,
          archiveSha256: createHash("sha256").update(bytes).digest("hex"),
        });
        packages.push(directory);
      }
      assert.equal(sourceImages.length, 1, "one actual platform native image required");
      const observations: unknown[] = [];
      for (const driver of ["rust", "typescript"]) {
        const entry = path.join(
          root,
          `tools/commands/release/npm/smoke-release-install.${driver === "rust" ? "rs" : "ts"}`,
        );
        const invocation =
          driver === "rust"
            ? rustSmokeInvocation(["--runtime-checks", "--keep-temp", ...packages])
            : {
                command: process.execPath,
                args: [entry, "--runtime-checks", "--keep-temp", ...packages],
              };
        const result = spawnSync(invocation.command, invocation.args, {
          cwd: root,
          encoding: "utf8",
          maxBuffer: Infinity,
          timeout: 180_000,
          env: {
            ...process.env,
            NODE_BIN: process.execPath,
            NPM_BIN: process.platform === "win32" ? "npm.cmd" : "npm",
            VIZE_NATIVE_BUILD_TOKEN: "11111111111111111111111111111111",
          },
        });
        assert.ifError(result.error);
        assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
        const kept = /^kept (.+)$/m.exec(result.stdout)?.[1];
        assert.ok(kept);
        retained.push(kept);
        assert.equal(
          result.stdout.replaceAll(kept, "$SMOKE"),
          [
            `install: @vizejs/native@${version}`,
            `install: ${names[1]}@${version}`,
            "runtime: @vizejs/native require/import compileSfc",
            "smoked 2/2 package tarballs",
            "kept $SMOKE",
            "",
          ].join("\n"),
        );
        assert.equal(result.stderr, "");
        const loaded = spawnSync(
          process.execPath,
          [
            "-e",
            `
const assert = require("node:assert/strict"), fs = require("node:fs"), path = require("node:path"), crypto = require("node:crypto");
const installed = process.argv[1];
const native = require(path.join(installed, "node_modules/@vizejs/native"));
const token = native.getNativeBuildToken();
assert.match(token, /^[0-9a-f]{32}$/);
assert.notEqual(token, process.env.VIZE_NATIVE_BUILD_TOKEN);
assert.equal(native.getNativeBuildToken(), token);
const images = Object.keys(require.cache).filter(name => name.endsWith(".node"));
assert.equal(images.length, 1);
for (const image of images) assert.ok(fs.realpathSync(image).startsWith(fs.realpathSync(installed) + path.sep));
console.log(JSON.stringify({token,images:images.map(name => {const body=fs.readFileSync(name);return {sha256:crypto.createHash("sha256").update(body).digest("hex"),bytes:body.length};})}));
`,
            path.join(kept, "install"),
          ],
          {
            encoding: "utf8",
            env: { ...process.env, VIZE_NATIVE_BUILD_TOKEN: "11111111111111111111111111111111" },
          },
        );
        assert.ifError(loaded.error);
        assert.equal(loaded.status, 0, loaded.stderr);
        assert.equal(loaded.stderr, "");
        const image = object(JSON.parse(loaded.stdout));
        assert.deepEqual(
          image.images,
          sourceImages,
          "actually loaded image must equal public SRI-bound native bytes",
        );
        observations.push(image);
      }
      assert.deepEqual(observations[1], observations[0]);
      console.log(
        JSON.stringify({
          fixture: "public-native-smoke-parity",
          packages: evidence,
          images: observations,
        }),
      );
    } finally {
      for (const directory of retained) fs.rmSync(directory, { recursive: true, force: true });
      fs.rmSync(temporary, { recursive: true, force: true });
    }
  },
);
