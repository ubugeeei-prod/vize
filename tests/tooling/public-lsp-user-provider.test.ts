import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";
import {
  installedCorsaProvider,
  assertInstalledCorsaProvider,
} from "../../tools/support/release/public_acceptance/lsp_assets.ts";
import { userProviderContract } from "../../tools/support/release/public_acceptance/lsp_source.ts";
import {
  installUserProvider,
  assertUserProviderInstall,
} from "../../tools/support/release/public_acceptance/lsp_user_provider.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import {
  policyFiles,
  sourceAuthority,
  assertCheckoutAuthority,
} from "../../tools/support/release/consumer_recovery/oracles.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const directory = path.join(root, "tests/_fixtures/release/legacy-0.571");
const legacy = fs.readFileSync(path.join(directory, "corsa-runtime.ts"));
const contractBytes = fs.readFileSync(
  path.join(root, "tests/_fixtures/release/user-typechecker-provider.json"),
);
const contract = JSON.parse(contractBytes.toString());
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const linux = process.platform === "linux" && process.arch === "x64";

function fixture(t: TestContext, historical = false) {
  const temp = fs.mkdtempSync(path.join(tmpdir(), "public-lsp-provider-"));
  t.after(() => fs.rmSync(temp, { recursive: true, force: true }));
  const source = path.join(temp, "source"),
    consumer = path.join(temp, "consumer");
  const write = (base: string, file: string, bytes: string | Buffer) => {
    const name = path.join(base, file);
    fs.mkdirSync(path.dirname(name), { recursive: true });
    fs.writeFileSync(name, bytes);
  };
  const cli = {
    name: "vize",
    version: "0.571.0",
    ...(historical ? { optionalDependencies: { [contract.name]: "7.0.2" } } : {}),
  };
  write(source, "npm/cli/package.json", JSON.stringify(cli));
  write(source, "tests/_fixtures/release/user-typechecker-provider.json", contractBytes);
  if (historical) write(source, "npm/cli/src/corsa-runtime.ts", legacy);
  execFileSync("git", ["init", "--quiet", source]);
  execFileSync("git", ["-C", source, "add", "."]);
  execFileSync("git", [
    "-C",
    source,
    "-c",
    "user.name=Provider fixture",
    "-c",
    "user.email=provider@example.test",
    "-c",
    "commit.gpgsign=false",
    "commit",
    "--quiet",
    "-m",
    "fixture",
  ]);
  const head = execFileSync("git", ["-C", source, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim();
  const request = {
    private: true,
    devDependencies: historical ? {} : { [contract.name]: "7.0.2" },
  };
  write(consumer, "package.json", JSON.stringify(request));
  const lock: {
    lockfileVersion: number;
    packages: Record<
      string,
      {
        private?: boolean;
        devDependencies?: Record<string, string>;
        version?: string;
        resolved?: string;
        integrity?: string;
      }
    >;
  } = {
    lockfileVersion: 3,
    packages: {
      "": request,
      "node_modules/vize": {
        version: "0.571.0",
        resolved: "https://registry.npmjs.org/vize/-/vize-0.571.0.tgz",
        integrity: "sha512-" + Buffer.alloc(64).toString("base64"),
      },
      [`node_modules/${contract.name}`]: {
        version: "7.0.2",
        resolved:
          "https://registry.npmjs.org/@typescript/typescript-linux-x64/-/typescript-linux-x64-7.0.2.tgz",
        integrity: contract.integrity,
      },
    },
  };
  write(consumer, "package-lock.json", JSON.stringify(lock));
  write(consumer, "node_modules/vize/package.json", JSON.stringify(cli));
  write(
    consumer,
    `node_modules/${contract.name}/package.json`,
    JSON.stringify({ name: contract.name, version: "7.0.2" }),
  );
  if (historical) write(consumer, "node_modules/vize/src/corsa-runtime.ts", legacy);
  // This is an observer fixture, not a language test: copy the independently
  // hash-pinned private dev image without running it or any producer build.
  const require = createRequire(import.meta.url);
  const provider = path.dirname(require.resolve(contract.name + "/package.json"));
  const image = fs.readFileSync(path.join(provider, "lib/tsc"));
  assert.equal(hash(image), contract.executableSha256);
  write(consumer, `node_modules/${contract.name}/lib/tsc`, image);
  const observe = () =>
    installedCorsaProvider(consumer, "0.571.0", head, contract.name, "7.0.2", source);
  return { consumer, source, head, observe, write, request, lock };
}

test("the historical observer retains exact public0.571 bytes outside production", () => {
  assert.equal(hash(legacy), "204311ddecf0dd9a687bfdf04ffaf3a4b5ec336e3ab71400fd7b660cf479935b");
  assert.equal(
    JSON.parse(fs.readFileSync(path.join(directory, "identity.json"), "utf8")).sha256,
    hash(legacy),
  );
  assert.equal(fs.existsSync(path.join(root, "npm/cli/src/corsa-runtime.ts")), false);
});

test("the historical inventory is readable as data but cannot authorize a new checkout", () => {
  const old = JSON.parse(
    fs.readFileSync(path.join(directory, "policy-files.json"), "utf8"),
  ) as string[];
  const declaration = (files: readonly string[]) =>
    `export const policyFiles = [\n${files.map((file) => `  ${JSON.stringify(file)},`).join("\n")}\n] as const;`;
  const read = (_sha: string, file: string) =>
    file.endsWith("oracles.ts") ? declaration(old) : "historical:" + file;
  const observed = sourceAuthority("a".repeat(40), read);
  assert.deepEqual(
    observed.map((row) => row.file),
    old,
  );
  assert.throws(
    () => assertCheckoutAuthority(observed, () => Buffer.from("")),
    /complete checkout policy required/,
  );
  assert.throws(
    () => sourceAuthority("a".repeat(40), () => declaration([...old, "foreign.ts"])),
    /unsupported historical/,
  );
  assert.ok(policyFiles.includes("tests/_fixtures/release/legacy-0.571/corsa-runtime.ts"));
});

test(
  "a current campaign requires a declared user provider and exact public image",
  { skip: !linux },
  async (t) => {
    const f = fixture(t),
      observed = await f.observe();
    assert.ok("resolutionMode" in observed);
    assert.equal(observed.resolutionMode, "explicit-user-provider");
    assert.equal(observed.executableSha256, contract.executableSha256);
    const selected = userProviderContract(f.source, f.head);
    assert.equal(selected.cohort, "explicit-user-provider-702-v1");
    await assertInstalledCorsaProvider(f.consumer, "0.571.0", f.head, observed, f.source);
    f.write(f.consumer, `node_modules/${contract.name}/lib/tsc`, "foreign image");
    await assert.rejects(f.observe, /reviewed public provider|strictly equal/);
  },
);

test(
  "physical or transitive provider presence cannot substitute for an explicit request",
  { skip: !linux },
  async (t) => {
    const f = fixture(t);
    f.write(f.consumer, "package.json", JSON.stringify({ private: true }));
    await assert.rejects(f.observe, /explicit user provider required/);
    f.write(f.consumer, "package.json", JSON.stringify(f.request));
    f.lock.packages[`node_modules/${contract.name}`].integrity =
      "sha512-" + Buffer.alloc(64).toString("base64");
    f.write(f.consumer, "package-lock.json", JSON.stringify(f.lock));
    await assert.rejects(f.observe, /reviewed public provider SRI required/);
  },
);

test(
  "the old bundled campaign keeps its original resolver receipt shape",
  { skip: !linux },
  async (t) => {
    const f = fixture(t, true),
      observed = await f.observe();
    assert.equal("resolutionMode" in observed, false);
    assert.deepEqual(Object.keys(observed.resolver), [
      "cli",
      "module",
      "moduleSha256",
      "javascriptSha256",
      "sourceFile",
      "head",
    ]);
    assert.ok("sourceFile" in observed.resolver && "moduleSha256" in observed.resolver);
    assert.equal(observed.resolver.sourceFile, "npm/cli/src/corsa-runtime.ts");
    assert.equal(observed.resolver.moduleSha256, hash(legacy));
  },
);

test(
  "the installer records an explicit request and retained-byte edits refuse finalization",
  { skip: !linux },
  (t) => {
    const f = fixture(t),
      output = path.join(f.consumer, "../output"),
      bin = path.join(f.consumer, "../bin");
    fs.mkdirSync(output);
    fs.mkdirSync(bin);
    f.write(f.consumer, "package.json", JSON.stringify({ private: true }));
    f.lock.packages[""] = { private: true };
    f.write(f.consumer, "package-lock.json", JSON.stringify(f.lock));
    writeFakeCommand(
      bin,
      "npm",
      [
        "const fs=require('node:fs');const args=process.argv.slice(2);",
        `if(args.at(-1)!==${JSON.stringify(contract.name + "@7.0.2")}||!args.includes('--ignore-scripts')||!args.includes('--save-dev'))process.exit(9);`,
        `const name=${JSON.stringify(contract.name)};`,
        "const request=JSON.parse(fs.readFileSync('package.json'));request.devDependencies={[name]:'7.0.2'};fs.writeFileSync('package.json',JSON.stringify(request));",
        "const lock=JSON.parse(fs.readFileSync('package-lock.json'));lock.packages[''].devDependencies=request.devDependencies;fs.writeFileSync('package-lock.json',JSON.stringify(lock));",
        "process.stdout.write('fixture package manager completed\\n');",
      ].join("\n"),
    );
    const previous = process.env.PATH;
    process.env.PATH = [bin, path.dirname(process.execPath), previous].join(path.delimiter);
    t.after(() => {
      if (previous === undefined) delete process.env.PATH;
      else process.env.PATH = previous;
    });
    const selected = userProviderContract(f.source, f.head);
    const receipt = installUserProvider(f.consumer, output, selected);
    assert.equal(receipt.resolutionMode, "explicit-user-provider");
    assertUserProviderInstall(f.consumer, output, selected, receipt);
    fs.appendFileSync(path.join(output, "lsp-user-provider.after.package.json"), " ");
    assert.throws(() => assertUserProviderInstall(f.consumer, output, selected, receipt));
  },
);
