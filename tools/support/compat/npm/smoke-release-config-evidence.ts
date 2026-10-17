/** Whole process/image evidence for the bounded installed config campaign. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

interface Image {
  path: string;
  sha256: string;
}
interface InstalledIdentity {
  project: string;
  workspace: string;
  native: { binaries: Image[] };
  corsa: { name: string; manifest: Image & { contents: string } };
}

/** Every installed image must be the actual source-built image, not a fallback. */
export function verifyInstalledImages(identities: InstalledIdentity[], expected: string): void {
  assert.match(expected, /^[0-9a-f]{64}$/u);
  assert.deepEqual(
    identities
      .map(({ project }) => path.basename(project))
      .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0)),
    ["app", "control"],
  );
  for (const identity of identities) {
    assert.equal(identity.native.binaries.length, 1, "one loaded native provider per sibling");
    const image = identity.native.binaries[0];
    const relative = path.relative(identity.workspace, image.path);
    assert.ok(
      relative !== "" && !path.isAbsolute(relative) && !relative.startsWith(".."),
      image.path,
    );
    assert.equal(image.sha256, expected, `installed native image for ${identity.project}`);
  }
}

const sha = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");

/** Keep the config session's complete, multi-megabyte packet off sync pipe buffers. */
export function runConfigHarness(
  executable: string,
  args: string[],
  options: { cwd: string; env: NodeJS.ProcessEnv; directory: string },
) {
  fs.mkdirSync(options.directory, { recursive: true });
  const directory = fs.mkdtempSync(path.join(path.resolve(options.directory), "config-harness-"));
  const stdout = path.join(directory, "stdout.jsonl");
  const stderr = path.join(directory, "stderr.log");
  const descriptors: number[] = [];
  let result;
  try {
    descriptors.push(fs.openSync(stdout, "wx"));
    descriptors.push(fs.openSync(stderr, "wx"));
    // Preserve the previous pipe stdin, direct executable, and absence of a
    // timeout. Only this harness's output transport changes; native wire/session
    // custody and every consumer assertion remain owned by the child.
    result = spawnSync(executable, args, {
      cwd: options.cwd,
      env: options.env,
      stdio: ["pipe", descriptors[0], descriptors[1]],
    });
  } finally {
    for (const descriptor of descriptors) fs.closeSync(descriptor);
  }
  const describe = (file: string) => {
    const bytes = fs.readFileSync(file);
    return { path: file, bytes: bytes.length, sha256: sha(bytes) };
  };
  const receipt = {
    scope: "config harness process transport; native/session assertions remain in the child",
    executable,
    args,
    cwd: options.cwd,
    // Explicit relevant values/absences, without publishing inherited credentials.
    environment: Object.fromEntries(
      [
        "PATH",
        "Path",
        "NODE_OPTIONS",
        "NODE_PATH",
        "CI",
        "TMPDIR",
        "TEMP",
        "TMP",
        "VIZE_SMOKE_LSP_LOG_DIR",
        "VIZE_TEST_DISABLE_TSGO",
        "VIZE_TRACE_EDITOR_PREPARATION",
        "CORSA_PATH",
        "CORSA_EXECUTABLE",
        "TSGO_PATH",
        "TSGO_EXECUTABLE",
      ].map((name) => [name, options.env[name] ?? null]),
    ),
    stdin: "pipe",
    timeout: null,
    pid: result.pid,
    status: result.status,
    signal: result.signal,
    error: result.error
      ? {
          ...Object.fromEntries(Object.entries(result.error)),
          name: result.error.name,
          message: result.error.message,
          stack: result.error.stack,
        }
      : null,
    stdout: describe(stdout),
    stderr: describe(stderr),
  };
  const file = path.join(directory, "process.json");
  fs.writeFileSync(file, JSON.stringify(receipt, null, 2) + "\n");
  console.log(
    JSON.stringify({
      configHarness: file,
      status: receipt.status,
      signal: receipt.signal,
      stdout: receipt.stdout,
      stderr: receipt.stderr,
    }),
  );
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `config harness failed; complete process evidence: ${file}`);
  assert.equal(result.signal, null, `config harness signaled; complete process evidence: ${file}`);
  return receipt;
}

function collect(): void {
  const directory = process.env.PACKED_CONFIG_OUTPUT;
  assert.ok(directory, "PACKED_CONFIG_OUTPUT is required");
  fs.mkdirSync(directory, { recursive: true });
  const receipt: Record<string, unknown> = {
    scope: "Linux Node24 source-built installed config observation; no release/full-matrix credit",
    passed: false,
  };
  try {
    const source = JSON.parse(fs.readFileSync(path.join(directory, "source.json"), "utf8"));
    const producer = JSON.parse(
      fs.readFileSync("npm/native/.artifacts/native/build-token-receipt.json", "utf8"),
    );
    receipt.source = source;
    receipt.nativeProducer = producer;
    assert.equal(producer.source, source.source);
    const log = fs.readFileSync(path.join(directory, "runtime.log"), "utf8");
    receipt.runtimeSha256 = sha(log);
    const identities: InstalledIdentity[] = [];
    for (const line of log.split("\n")) {
      let record;
      try {
        record = JSON.parse(line);
      } catch {
        continue;
      }
      if (typeof record.stdout !== "string") continue;
      let identity;
      try {
        identity = JSON.parse(record.stdout);
      } catch {
        continue;
      }
      if (!identity?.native?.binaries || !identity?.vize || !identity?.corsa) continue;
      identities.push(identity);
    }
    receipt.installed = identities;
    verifyInstalledImages(identities, producer.binary.sha256);
    receipt.corsaPayloads = identities.map((identity) => {
      for (const image of identity.native.binaries) {
        assert.equal(fs.realpathSync(image.path), image.path);
        assert.equal(sha(fs.readFileSync(image.path)), image.sha256);
      }
      const manifest = identity.corsa.manifest;
      assert.equal(sha(fs.readFileSync(manifest.path)), manifest.sha256);
      assert.equal(sha(manifest.contents), manifest.sha256);
      const metadata = JSON.parse(manifest.contents);
      assert.equal(metadata.name, identity.corsa.name);
      const payload = fs.realpathSync(path.join(path.dirname(manifest.path), "lib/tsc"));
      const relative = path.relative(identity.workspace, payload);
      assert.ok(
        relative !== "" && !path.isAbsolute(relative) && !relative.startsWith(".."),
        payload,
      );
      return {
        project: identity.project,
        package: metadata.name,
        version: metadata.version,
        path: payload,
        sha256: sha(fs.readFileSync(payload)),
        scope:
          "installed platform payload identity; process selection follows source-authenticated installed discovery, not an observed child PID",
      };
    });
    const kept = [...log.matchAll(/^kept (.+)$/gmu)];
    assert.equal(kept.length, 1, "one retained fresh-install root");
    const packs = path.join(kept[0][1], "packs");
    receipt.tarballs = fs
      .readdirSync(packs)
      .filter((name) => name.endsWith(".tgz"))
      .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0))
      .map((name) => {
        const file = path.join(packs, name);
        return { name, bytes: fs.statSync(file).size, sha256: sha(fs.readFileSync(file)) };
      });
    assert.ok((receipt.tarballs as unknown[]).length > 0);
    receipt.passed = true;
  } catch (error) {
    receipt.failure = error instanceof Error ? `${error.name}: ${error.message}` : String(error);
    throw error;
  } finally {
    fs.writeFileSync(
      path.join(directory, "installed-images.json"),
      JSON.stringify(receipt, null, 2) + "\n",
    );
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  assert.equal(process.argv[2], "collect");
  collect();
}
