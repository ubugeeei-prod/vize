import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import {
  snapshot,
  plainHostEnvironment,
  readEvents,
} from "../src/test-support/html-cli-oracles.mjs";
import { presentationInputs } from "../src/cli/presentation-context.ts";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const errorPacket = (error) =>
  error == null
    ? null
    : {
        name: error.name,
        ...Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]])),
      };
const sanitized = () => {
  const env = plainHostEnvironment(process.env);
  for (const key of [
    "NODE_OPTIONS",
    "VIZE_OXLINT_NATIVE_CUSTODY",
    "VIZE_PREFER_WORKSPACE_BINDING",
    "VIZE_OXLINT_TEST_ENTRYPOINT",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
  ])
    delete env[key];
  return env;
};
const authorities = (cwd) => {
  const files = [];
  for (let directory = cwd; ; directory = path.dirname(directory)) {
    for (const name of [".gitignore", ".eslintignore"]) {
      const file = path.join(directory, name);
      try {
        files.push([file, snapshot(file)]);
      } catch (error) {
        if (error.code !== "ENOENT") throw error;
        files.push([file, null]);
      }
    }
    if (path.dirname(directory) === directory) return files;
  }
};

export function installMixedHost(directory, output, version) {
  fs.mkdirSync(output, { recursive: true });
  const args = [
    "install",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    "--legacy-peer-deps",
    "oxlint@1.81.0",
    ...(version ? [`oxlint-plugin-vize@${version}`] : []),
  ];
  const result = spawnSync("npm", args, {
    cwd: directory,
    env: sanitized(),
    timeout: 180_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const record = {
    command: "npm",
    args,
    cwd: directory,
    status: result.status,
    signal: result.signal,
    error: errorPacket(result.error),
    stdoutBytes: Array.from(result.stdout ?? []),
    stderrBytes: Array.from(result.stderr ?? []),
  };
  fs.writeFileSync(path.join(output, "install.json"), JSON.stringify(record, null, 2) + "\n");
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  assert.equal(record.error, null);
  assert.equal(record.signal, null);
  assert.equal(record.status, 0);
  const lock = fs.readFileSync(path.join(directory, "package-lock.json"));
  fs.writeFileSync(path.join(output, "package-lock.json"), lock);
  const packages = JSON.parse(lock).packages;
  for (const [name, expected] of [
    ["oxlint", "1.81.0"],
    ...(version ? [["oxlint-plugin-vize", version]] : []),
  ]) {
    const item = packages[`node_modules/${name}`];
    assert.equal(item.version, expected);
    assert.match(item.integrity, /^sha512-/u);
    assert.ok(item.resolved.startsWith("https://registry.npmjs.org/"));
    assert.equal(
      JSON.parse(fs.readFileSync(path.join(directory, "node_modules", name, "package.json")))
        .version,
      expected,
    );
  }
  return {
    lockSha256: hash(lock),
    engine: fs.realpathSync(path.join(directory, "node_modules/oxlint/bin/oxlint")),
  };
}

export function qualifyHistoricalMixedDirectory({ root, output }) {
  const corpus = path.join(root, "tests/_fixtures/differential/lint/oxlint-mixed-directory-8507");
  const plan = JSON.parse(fs.readFileSync(path.join(corpus, "controls.json")));
  const capture = {
    complete: false,
    inputs: snapshot(corpus),
    attempts: [],
    providers: [],
    observations: [],
    limits: [
      "Linux Actions historical registry assembly, not the reporter's Darwin installation",
      "only the explicit plugin version is substituted in the original package.json",
      "no current-source, installed-release or full HTML-corpus credit",
    ],
  };
  fs.mkdirSync(output, { recursive: true });
  const save = () =>
    fs.writeFileSync(
      path.join(output, "historical-before.json"),
      JSON.stringify(capture, null, 2) + "\n",
    );
  const preload = fileURLToPath(new URL("./mixed-directory-history-load.cjs", import.meta.url));
  const failure = "Script-safe Vue transport unavailable:";
  save();
  try {
    for (const version of plan.historicalBefore.pluginVersions) {
      const installation = fs.realpathSync(
        fs.mkdtempSync(path.join(os.tmpdir(), "vize-8507-before-host-")),
      );
      const directory = path.join(output, version),
        calls = path.join(directory, "loaded-addons.jsonl");
      const attempt = { version, installation, directory, phase: "install" };
      capture.attempts.push(attempt);
      save();
      try {
        const provider = installMixedHost(installation, directory, version);
        attempt.phase = "cases";
        save();
        const plugin = path.join(installation, "node_modules/oxlint-plugin-vize");
        const metadataBytes = fs.readFileSync(path.join(plugin, "package.json"));
        const metadata = JSON.parse(metadataBytes);
        const wrapper = fs.realpathSync(
          path.join(
            plugin,
            typeof metadata.bin === "string" ? metadata.bin : metadata.bin["oxlint-vize"],
          ),
        );
        const providerBefore = snapshot(installation).map(([file, kind, bytes]) =>
          kind === "file" ? [file, kind, hash(Buffer.from(bytes, "base64"))] : [file, kind, bytes],
        );
        capture.providers.push({
          version,
          installation,
          ...provider,
          metadataBytes: Array.from(metadataBytes),
          files: providerBefore,
        });
        save();
        for (const name of plan.historicalBefore.cases) {
          const fixture = plan.cases.find((item) => item.name === name);
          const temporary = fs.realpathSync(
            fs.mkdtempSync(path.join(os.tmpdir(), "vize-8507-before-case-")),
          );
          const cwd = path.join(temporary, "repro");
          try {
            for (let ancestor = temporary; ; ancestor = path.dirname(ancestor)) {
              for (const marker of [".git", ".jj"])
                assert.ok(!fs.existsSync(path.join(ancestor, marker)));
              if (path.dirname(ancestor) === ancestor) break;
            }
            for (const file of plan.baseFiles) {
              const target = path.join(cwd, file);
              fs.mkdirSync(path.dirname(target), { recursive: true });
              fs.copyFileSync(path.join(corpus, file), target);
            }
            const originalPackageBytes = fs.readFileSync(path.join(cwd, "package.json"));
            const packageBytes = Buffer.from(
              originalPackageBytes.toString().replace('"0.441.0"', `"${version}"`),
            );
            fs.writeFileSync(path.join(cwd, "package.json"), packageBytes);
            for (const [file, source] of Object.entries(fixture.overlay ?? {}))
              fs.copyFileSync(path.join(corpus, source), path.join(cwd, file));
            fs.mkdirSync(path.join(cwd, "node_modules"));
            for (const name of ["oxlint", "oxlint-plugin-vize"])
              fs.symlinkSync(
                path.join(installation, "node_modules", name),
                path.join(cwd, "node_modules", name),
                "dir",
              );
            const before = snapshot(temporary),
              sourceAuthorities = authorities(cwd),
              index = readEvents(calls).length;
            const env = {
              ...sanitized(),
              NODE_OPTIONS: `--require=${JSON.stringify(preload)}`,
              VIZE_OXLINT_HISTORY_INSTALL: installation,
              VIZE_OXLINT_HISTORY_CALLS: calls,
            };
            const result = spawnSync(process.execPath, [wrapper, ...fixture.argv], {
              cwd,
              env,
              timeout: 60_000,
              maxBuffer: 64 * 1024 * 1024,
            });
            const record = {
              version,
              environment: Object.fromEntries(
                [
                  ...presentationInputs,
                  "NODE_OPTIONS",
                  "VIZE_OXLINT_HISTORY_INSTALL",
                  "VIZE_OXLINT_HISTORY_CALLS",
                  "VIZE_PREFER_WORKSPACE_BINDING",
                  "VIZE_OXLINT_NATIVE_CUSTODY",
                  "NAPI_RS_NATIVE_LIBRARY_PATH",
                  "NAPI_RS_FORCE_WASI",
                ].map((key) => [key, env[key] ?? null]),
              ),
              fixture: name,
              cwd,
              provider,
              wrapper,
              wrapperSha256: hash(fs.readFileSync(wrapper)),
              metadataBytes: Array.from(metadataBytes),
              originalPackageBytes: Array.from(originalPackageBytes),
              packageBytes: Array.from(packageBytes),
              command: [process.execPath, wrapper, ...fixture.argv],
              before,
              after: snapshot(temporary),
              sourceAuthorities,
              sourceAuthoritiesAfter: authorities(cwd),
              status: result.status,
              signal: result.signal,
              error: errorPacket(result.error),
              stdoutBytes: Array.from(result.stdout ?? []),
              stderrBytes: Array.from(result.stderr ?? []),
              loadedAddons: readEvents(calls).slice(index),
            };
            capture.observations.push(record);
            save();
            assert.equal(record.signal, null);
            assert.equal(record.error, null);
            assert.deepEqual(record.after, before);
            assert.deepEqual(record.sourceAuthoritiesAfter, sourceAuthorities);
            for (const event of record.loadedAddons) {
              if (event.kind === "load") {
                assert.equal(event.outcome, "return");
                assert.equal(event.error, null);
                const expected = providerBefore.find(
                  ([file, kind]) =>
                    kind === "file" && file === path.relative(installation, event.file),
                );
                assert.ok(expected, "historical load has no original installed file identity");
                assert.equal(event.sha256, expected[2]);
              } else {
                assert.ok(["host-child", "host-handshake"].includes(event.kind));
                assert.equal(event.signal, null);
                assert.equal(event.error, null);
                assert.equal(fs.realpathSync(event.provider), provider.engine);
                assert.equal(event.providerSha256, hash(fs.readFileSync(provider.engine)));
                if (event.kind === "host-handshake") {
                  assert.equal(event.status, 0);
                  assert.equal(Buffer.from(event.stdoutBytes).toString().trim(), "Version: 1.81.0");
                }
              }
            }
            const explicit = fixture.argv[0] === "app/App.vue";
            const text = Buffer.concat([
              result.stdout ?? Buffer.alloc(0),
              result.stderr ?? Buffer.alloc(0),
            ]).toString();
            const expected = version === "0.435.0" || explicit ? fixture.expectedExit : 1;
            assert.equal(record.status, expected, `historical ${version} ${name}: ${text}`);
            if (version !== "0.435.0" && !explicit) assert.ok(text.includes(failure));
            else {
              assert.ok(!text.includes(failure));
              if (fixture.expectedDiagnostics.length)
                assert.ok(text.includes("Duplicate attribute 'id'"));
            }
          } finally {
            fs.rmSync(temporary, { recursive: true, force: true });
          }
        }
        assert.deepEqual(
          snapshot(installation).map(([file, kind, bytes]) =>
            kind === "file"
              ? [file, kind, hash(Buffer.from(bytes, "base64"))]
              : [file, kind, bytes],
          ),
          providerBefore,
        );
        attempt.phase = "complete";
        save();
      } finally {
        fs.rmSync(installation, { recursive: true, force: true });
      }
    }
    assert.equal(capture.observations.length, 24);
    assert.deepEqual(snapshot(corpus), capture.inputs);
    capture.complete = true;
    save();
  } catch (error) {
    capture.failure = errorPacket(error);
    save();
    throw error;
  }
  return { cases: capture.observations.length, complete: capture.complete };
}
