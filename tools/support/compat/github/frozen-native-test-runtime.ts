// Explicit dependency for historical CI fixtures. Production resolvers never
// import this file and new discovery observers must remove this test pin.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const name = "@typescript/typescript-linux-x64";
const version = "7.0.2";
const image = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
const variables = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
];

export function verifyFrozenNativeTestRuntime(root: string) {
  const workspace = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8")) as {
    private?: boolean;
    optionalDependencies?: Record<string, string>;
  };
  assert.equal(
    workspace.private,
    true,
    "immutable fixture provider belongs to the private workspace",
  );
  assert.equal(
    workspace.optionalDependencies?.[name],
    "catalog:corsa-runtime",
    "private fixture provider must remain declared",
  );
  const packageRoot = fs.realpathSync(path.join(root, "node_modules", name));
  const manifest = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8")) as {
    name?: string;
    version?: string;
  };
  assert.equal(manifest.name, name);
  assert.equal(manifest.version, version);
  const executable = fs.realpathSync(path.join(packageRoot, "lib", "tsc"));
  assert.equal(
    path.dirname(path.dirname(executable)),
    packageRoot,
    "fixture executable must belong to the declared package",
  );
  assert(fs.statSync(executable).isFile());
  assert(!/[\r\n]/.test(executable));
  const sha256 = createHash("sha256").update(fs.readFileSync(executable)).digest("hex");
  assert.equal(sha256, image, "immutable historical provider image changed");
  return {
    schema: "vize.frozen-native-test-runtime",
    version: 1,
    scope: "explicit historical fixture dependency; zero production-default or new SDK credit",
    name,
    packageVersion: version,
    executable,
    sha256,
  };
}

export function pinFrozenNativeTestRuntime(
  root: string,
  environment: NodeJS.ProcessEnv,
  receiptPath: string,
) {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  for (const key of variables)
    assert(!environment[key], `refusing to replace an existing explicit ${key}`);
  assert(environment.GITHUB_ENV, "GitHub Actions environment file is required");
  const provider = verifyFrozenNativeTestRuntime(root);
  const source = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
  assert.match(source, /^[a-f0-9]{40}$/);
  const receipt = {
    ...provider,
    source,
    run: environment.GITHUB_RUN_ID,
    attempt: environment.GITHUB_RUN_ATTEMPT,
  };
  fs.writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  fs.appendFileSync(environment.GITHUB_ENV, `CORSA_PATH=${receipt.executable}\n`);
  return receipt;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [receipt] = process.argv.slice(2);
  assert(receipt, "explicit fixture receipt path is required");
  process.stdout.write(
    `${JSON.stringify(pinFrozenNativeTestRuntime(process.cwd(), process.env, receipt))}\n`,
  );
}
