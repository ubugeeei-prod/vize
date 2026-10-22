import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { extractNpmPackage, resolveCorsaServerEnv, serverProcessEnv } from "../src/corsa-runtime.ts";

const packageName = `@typescript/typescript-${process.platform}-${process.arch}`;
const executableName = process.platform === "win32" ? "tsc.exe" : "tsc";
const platformArch = `${process.platform}-${process.arch}`;

void test("createServerOptions env carries CORSA_PATH when lib/tsc exists", async () => {
  const fixture = createFixture();
  const executable = writeRuntime(path.join(fixture.extensionPath, "node_modules", packageName));
  try {
    const env = await resolveCorsaServerEnv({
      extensionPath: fixture.extensionPath,
      globalStoragePath: fixture.storage,
      env: { PATH: "/usr/bin" },
      install: blockedInstall,
    });
    assert.equal(serverProcessEnv(env, false).CORSA_PATH, executable);
    assert.equal(serverProcessEnv(env, true).CORSA_PATH, executable);
    assert.equal(serverProcessEnv(env, true).RUST_BACKTRACE, "1");
    assert.equal(env.PATH, "/usr/bin");
  } finally {
    fixture.cleanup();
  }
});

void test("a cached lib/tsc is used before downloading", async () => {
  const fixture = createFixture();
  const executable = writeRuntime(
    path.join(fixture.storage, "corsa", "7.0.2", platformArch),
  );
  try {
    const env = await resolveCorsaServerEnv({
      extensionPath: fixture.extensionPath,
      globalStoragePath: fixture.storage,
      env: {},
      install: blockedInstall,
    });
    assert.equal(env.CORSA_PATH, executable);
  } finally {
    fixture.cleanup();
  }
});

void test("does not override a pre-set CORSA_PATH", async () => {
  const fixture = createFixture();
  writeRuntime(path.join(fixture.extensionPath, "node_modules", packageName));
  try {
    for (const key of ["CORSA_PATH", "TSGO_PATH", "CORSA_EXECUTABLE", "TSGO_EXECUTABLE"] as const) {
      const env = await resolveCorsaServerEnv({
        extensionPath: fixture.extensionPath,
        globalStoragePath: fixture.storage,
        env: { [key]: "/already/set", PATH: "/usr/bin" },
        install: blockedInstall,
      });
      assert.equal(serverProcessEnv(env, false)[key], "/already/set");
      assert.equal(serverProcessEnv(env, true)[key], "/already/set");
      if (key !== "CORSA_PATH") assert.equal(env.CORSA_PATH, undefined);
    }
  } finally {
    fixture.cleanup();
  }
});

void test("names the missing lib/tsc and the required package", async () => {
  const fixture = createFixture();
  const lines: string[] = [];
  const expected = path.join(fixture.storage, "corsa", "7.0.2", platformArch, "lib", executableName);
  try {
    const env = await resolveCorsaServerEnv({
      extensionPath: fixture.extensionPath,
      globalStoragePath: fixture.storage,
      env: { CORSA_PATH: "" },
      output: { appendLine: (line) => lines.push(line) },
      install: async () => {
        throw new Error("offline");
      },
    });
    assert.equal(env.CORSA_PATH, "");
    assert.ok(
      lines.some(
        (line) => line.includes(expected) && line.includes(`${packageName} 7.0.2 is required`),
      ),
      lines.join("\n"),
    );
  } finally {
    fixture.cleanup();
  }
});

function createFixture(): { cleanup: () => void; extensionPath: string; storage: string } {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vscode-corsa-"));
  const extensionPath = path.join(root, "extension");
  fs.mkdirSync(extensionPath, { recursive: true });
  fs.writeFileSync(
    path.join(extensionPath, "package.json"),
    JSON.stringify({ optionalDependencies: { [packageName]: "7.0.2" } }),
  );
  return {
    extensionPath,
    storage: path.join(root, "storage"),
    cleanup: () => fs.rmSync(root, { recursive: true, force: true }),
  };
}

function writeRuntime(packageRoot: string): string {
  const executable = path.join(packageRoot, "lib", executableName);
  fs.mkdirSync(path.dirname(executable), { recursive: true });
  fs.writeFileSync(executable, "fake-tsc");
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    JSON.stringify({ name: packageName, version: "7.0.2" }),
  );
  return executable;
}

function blockedInstall(): Promise<void> {
  throw new Error("download should not run");
}

void test("extracts package/lib/tsc and ignores paths outside the package", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vscode-corsa-tar-"));
  try {
    const tar = Buffer.concat([
      tarEntry("package/package.json", Buffer.from('{"name":"runtime"}')),
      tarEntry(`package/lib/${executableName}`, Buffer.from("fake-tsc")),
      tarEntry("package/../outside.txt", Buffer.from("nope")),
      Buffer.alloc(512),
    ]);
    extractNpmPackage(tar, root);
    assert.equal(fs.readFileSync(path.join(root, "lib", executableName), "utf8"), "fake-tsc");
    assert.equal(fs.readFileSync(path.join(root, "package.json"), "utf8"), '{"name":"runtime"}');
    assert.equal(fs.existsSync(path.join(path.dirname(root), "outside.txt")), false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

function tarEntry(name: string, contents: Buffer): Buffer {
  const header = Buffer.alloc(512);
  header.write(name, 0, "utf8");
  header.write(contents.length.toString(8).padStart(11, "0"), 124, "utf8");
  header[156] = 48;
  const padding = (512 - (contents.length % 512)) % 512;
  return Buffer.concat([header, contents, Buffer.alloc(padding)]);
}
