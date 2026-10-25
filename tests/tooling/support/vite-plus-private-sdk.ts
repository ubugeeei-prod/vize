// The original packed consumer keeps its immutable comparison SDK explicitly.
// This observer grants no production-default or new SDK discovery credit.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const overrides = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
];
const contexts = new Map<
  string,
  { evidence: string; executable: string; image: string; sequence: number }
>();
const hash = (value: Uint8Array) => createHash("sha256").update(value).digest("hex");

type Image = {
  name: string;
  version: string;
  binary: string;
  executableSha256: string;
  executableBytes: number;
  integrity: string;
  archiveSha256: string;
};

export function privateSdkChildEnvironment(original: NodeJS.ProcessEnv): NodeJS.ProcessEnv {
  const environment: NodeJS.ProcessEnv = { ...original, NO_COLOR: "1", FORCE_COLOR: "0" };
  // Isolate this child, never the caller or unrelated runtime-discovery tests.
  for (const key of overrides) delete environment[key];
  return environment;
}

function retain(file: string, result: ReturnType<typeof spawnSync>) {
  const bytes = (value: string | Buffer | null) => Buffer.from(value ?? "");
  fs.writeFileSync(
    file,
    JSON.stringify(
      {
        status: result.status,
        signal: result.signal,
        error: result.error
          ? Object.fromEntries(
              Object.getOwnPropertyNames(result.error).map((key) => [
                key,
                Reflect.get(result.error!, key),
              ]),
            )
          : null,
        stdout: bytes(result.stdout).toString("utf8"),
        stderr: bytes(result.stderr).toString("utf8"),
        stdoutBase64: bytes(result.stdout).toString("base64"),
        stderrBase64: bytes(result.stderr).toString("base64"),
      },
      null,
      2,
    ) + "\n",
  );
}

export function preparePrivateSdkProvider(
  root: string,
  directory: string,
): { executable: string; evidence: string } {
  const pinsPath = path.join(root, "tests/_fixtures/tooling/vite-plus-private-sdk-702.json");
  const pins = JSON.parse(fs.readFileSync(pinsPath, "utf8")) as {
    schema: string;
    version: string;
    platforms: Image[];
  };
  assert.equal(pins.schema, "vize.vite-plus.private-sdk.v1");
  assert.equal(pins.version, "7.0.2");
  const name = `@typescript/typescript-${process.platform}-${process.arch}`;
  const pin = pins.platforms.find((entry) => entry.name === name);
  assert(pin, `No reviewed private comparison SDK for ${process.platform}/${process.arch}`);
  assert.equal(pin.version, "7.0.2");
  assert.equal(pin.binary, process.platform === "win32" ? "lib/tsc.exe" : "lib/tsc");
  const workspace = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
  assert.equal(workspace.private, true);
  assert.equal(workspace.optionalDependencies[name], "catalog:corsa-runtime");
  const installed = fs.realpathSync(path.join(root, "node_modules", name));
  const manifest = JSON.parse(fs.readFileSync(path.join(installed, "package.json"), "utf8"));
  assert.equal(manifest.name, name);
  assert.equal(manifest.version, pin.version);
  assert(fs.lstatSync(path.join(installed, pin.binary)).isFile());
  const originalImage = fs.readFileSync(path.join(installed, pin.binary));
  assert.equal(originalImage.length, pin.executableBytes);
  assert.equal(hash(originalImage), pin.executableSha256, "immutable historical SDK image changed");
  const destination = path.join(directory, ".vize-private-sdk/node_modules", name);
  assert.equal(
    fs.existsSync(path.join(directory, ".vize-private-sdk")),
    false,
    "fresh fixture-owned SDK directory required",
  );
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.cpSync(installed, destination, { recursive: true, dereference: true });
  const executable = fs.realpathSync(path.join(destination, pin.binary));
  assert.equal(path.dirname(path.dirname(executable)), fs.realpathSync(destination));
  assert.equal(hash(fs.readFileSync(executable)), pin.executableSha256);
  const packagePath = path.join(directory, "package.json");
  const consumer = fs.existsSync(packagePath)
    ? JSON.parse(fs.readFileSync(packagePath, "utf8"))
    : { private: true };
  assert.equal(consumer.private, true);
  assert.equal(consumer.devDependencies?.[name], undefined);
  consumer.devDependencies = { ...consumer.devDependencies, [name]: pin.version };
  fs.writeFileSync(packagePath, JSON.stringify(consumer));
  const lock = fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8");
  assert(
    lock.includes(`  '${name}@${pin.version}':\n    resolution: {integrity: ${pin.integrity}}\n`),
    "source lock must retain reviewed provider SRI",
  );
  const artifactRoot = path.join(root, "target/differential/vite-plus-private-sdk");
  fs.mkdirSync(artifactRoot, { recursive: true });
  const evidence = fs.mkdtempSync(path.join(artifactRoot, "consumer-"));
  fs.writeFileSync(
    path.join(evidence, "provider.json"),
    JSON.stringify(
      {
        schema: "vize.vite-plus.explicit-private-sdk.v1",
        scope: "Explicit historical provider only; no default/alias/new SDK credit",
        cohort: "explicit-private-sdk-702-v1",
        pin,
        executable,
        consumerPackage: consumer,
        fixturePinsSha256: hash(fs.readFileSync(pinsPath)),
        sourceLockSha256: hash(fs.readFileSync(path.join(root, "pnpm-lock.yaml"))),
        physicalImageSha256: hash(fs.readFileSync(executable)),
        node: { executable: process.execPath, version: process.version },
      },
      null,
      2,
    ) + "\n",
  );
  const version = spawnSync(executable, ["--version"], {
    encoding: "utf8",
    env: privateSdkChildEnvironment(process.env),
    timeout: 30_000,
  });
  retain(path.join(evidence, "sdk-version.json"), version);
  assert.equal(version.error, undefined);
  assert.equal(version.signal, null);
  assert.equal(version.status, 0, version.stderr);
  assert.equal(version.stdout, "Version 7.0.2\n");
  assert.equal(version.stderr, "");
  contexts.set(path.resolve(directory), {
    evidence,
    executable,
    image: pin.executableSha256,
    sequence: 0,
  });
  return { executable, evidence };
}

export function preparePrivateSdkFixture(root: string, directory: string): void {
  const { executable, evidence } = preparePrivateSdkProvider(root, directory);
  const config = path.join(directory, "vite.config.ts");
  const original = fs.readFileSync(config, "utf8");
  const anchor = "export default defineConfig({\n";
  assert.equal(original.split(anchor).length, 2, "one authored config entry required");
  fs.writeFileSync(
    config,
    original.replace(
      anchor,
      `${anchor}  vue: { typeChecker: { corsaPath: ${JSON.stringify(executable)} } },\n`,
    ),
  );
  fs.writeFileSync(
    path.join(evidence, "config.json"),
    JSON.stringify(
      {
        originalConfigBase64: Buffer.from(original).toString("base64"),
        configuredSource: fs.readFileSync(config, "utf8"),
      },
      null,
      2,
    ) + "\n",
  );
}

export function retainPrivateSdkProcess(
  cwd: string,
  argv: string[],
  result: ReturnType<typeof spawnSync>,
): void {
  const context = contexts.get(path.resolve(cwd));
  if (!context) return; // Initial package builds are the unchanged source producer.
  const stem = `${String(context.sequence++).padStart(2, "0")}-${argv.join("-").replaceAll(/[^a-zA-Z0-9_-]/g, "_")}`;
  retain(path.join(context.evidence, `${stem}.process.json`), result);
  fs.writeFileSync(
    path.join(context.evidence, `${stem}.command.json`),
    JSON.stringify(
      {
        cwd,
        argv,
        executable: process.execPath,
        nativeProvider: context.executable,
        runtimeOverrides: Object.fromEntries(
          overrides.map((key) => [key, privateSdkChildEnvironment(process.env)[key] ?? null]),
        ),
      },
      null,
      2,
    ) + "\n",
  );
  assert.equal(
    hash(fs.readFileSync(context.executable)),
    context.image,
    "consumer command changed private SDK",
  );
}
