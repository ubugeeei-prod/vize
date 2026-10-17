import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  installedCorsaProvider,
  packageProvider,
} from "../../tools/support/release/public_acceptance/lsp_assets.ts";
import { sha256 } from "../../tools/support/release/public_acceptance/lsp_source.ts";

const released = JSON.parse(
  fs.readFileSync(
    new URL("../_fixtures/release/public-corsa-resolver.json", import.meta.url),
    "utf8",
  ),
);
const name = `@typescript/typescript-${process.platform}-${process.arch}`;
const version = "7.0.2";
const write = (file: string, value: unknown) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, typeof value === "string" ? value : JSON.stringify(value));
};
function fixture() {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-corsa-")));
  const consumer = path.join(root, "fresh public consumer Ω"),
    repo = path.join(root, "source");
  const cli = path.join(consumer, "node_modules/vize"),
    provider = path.join(consumer, "node_modules", name);
  write(path.join(cli, "package.json"), {
    name: "vize",
    version: released.version,
    type: "module",
    optionalDependencies: { [name]: version },
  });
  write(path.join(provider, "package.json"), {
    name,
    version,
    exports: { "./package.json": "./package.json" },
  });
  const module = path.join(cli, released.file);
  write(module, released.content);
  const binary = path.join(provider, "lib", process.platform === "win32" ? "tsc.exe" : "tsc");
  write(binary, "controlled provider bytes; no product execution in this discovery test\n");
  write(path.join(consumer, "package-lock.json"), {
    lockfileVersion: 3,
    packages: Object.fromEntries(
      [
        ["vize", released.version],
        [name, version],
      ].map(([pkg, ver]) => [
        `node_modules/${pkg}`,
        {
          version: ver,
          resolved: `https://registry.npmjs.org/${pkg}/-/${pkg.split("/").at(-1)}-${ver}.tgz`,
          integrity: `sha512-${Buffer.alloc(64, 1).toString("base64")}`,
        },
      ]),
    ),
  });
  write(path.join(repo, "npm/cli/src/corsa-runtime.ts"), released.content);
  const git = (args: string[]) =>
    execFileSync("git", args, { cwd: repo, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
  git(["init", "--quiet"]);
  git(["add", "npm/cli/src/corsa-runtime.ts"]);
  git([
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.test",
    "-c",
    "commit.gpgSign=false",
    "-c",
    "core.hooksPath=/dev/null",
    "commit",
    "--quiet",
    "-m",
    "controlled frozen launcher source",
  ]);
  const head = git(["rev-parse", "HEAD"]).trim();
  return {
    root,
    consumer,
    repo,
    cli,
    provider,
    module,
    binary,
    head,
    resolve: () => installedCorsaProvider(consumer, released.version, head, name, version, repo),
    cleanup: () => fs.rmSync(root, { recursive: true, force: true }),
  };
}

test("authentic public launcher resolves its installed Corsa contract rather than guessed lib/tsgo", async () => {
  assert.equal(sha256(released.content), released.sha256);
  const f = fixture();
  try {
    assert.equal(fs.existsSync(path.join(f.provider, "lib/tsgo")), false);
    const result = await f.resolve();
    assert.equal(result.executable, fs.realpathSync(f.binary));
    assert.equal(result.executableSha256, sha256(fs.readFileSync(f.binary)));
    assert.equal(result.resolver.moduleSha256, released.sha256);
    assert.match(result.resolver.javascriptSha256, /^[a-f0-9]{64}$/u);
    assert.equal(fs.readFileSync(f.module, "utf8"), released.content);
    assert.equal(result.resolver.head, f.head);
    assert.equal(result.resolver.cli.version, "0.457.0");
    assert.deepEqual(
      JSON.parse(JSON.stringify(result)),
      result,
      "retained identity round trips exactly",
    );
  } finally {
    f.cleanup();
  }
});

const mutations: Record<string, (f: ReturnType<typeof fixture>) => void> = {
  "missing shipped executable cannot fall back to a guessed tsgo or host executable": (f) => {
    fs.renameSync(f.binary, path.join(f.provider, "lib/tsgo"));
  },
  "changed installed resolver refuses before module execution": (f) => {
    fs.appendFileSync(f.module, "\nthrow new Error('untrusted resolver executed');\n");
  },
  "CLI provider declaration must match the original exact version": (f) => {
    const p = path.join(f.cli, "package.json");
    const manifest = JSON.parse(fs.readFileSync(p, "utf8"));
    manifest.optionalDependencies[name] = "7.0.1";
    write(p, manifest);
  },
  "provider manifest version must match its public lock": (f) => {
    write(path.join(f.provider, "package.json"), { name, version: "7.0.1" });
  },
  "provider manifest identity must match the platform declaration": (f) => {
    write(path.join(f.provider, "package.json"), { name: "foreign/provider", version });
  },
  "foreign registry entries cannot authorize a provider": (f) => {
    const file = path.join(f.consumer, "package-lock.json");
    const lock = JSON.parse(fs.readFileSync(file, "utf8"));
    lock.packages[`node_modules/${name}`].resolved = "https://example.invalid/provider.tgz";
    write(file, lock);
  },
  "linked provider entries cannot borrow a workspace runtime": (f) => {
    const file = path.join(f.consumer, "package-lock.json");
    const lock = JSON.parse(fs.readFileSync(file, "utf8"));
    lock.packages[`node_modules/${name}`].link = true;
    write(file, lock);
  },
  "provider binary symlink cannot escape its public package": (f) => {
    const outside = path.join(f.root, "borrowed-runtime");
    write(outside, fs.readFileSync(f.binary, "utf8"));
    fs.unlinkSync(f.binary);
    fs.symlinkSync(outside, f.binary);
  },
  "resolver symlink cannot borrow code outside the installed CLI": (f) => {
    const outside = path.join(f.root, "borrowed-resolver.ts");
    write(outside, released.content);
    fs.unlinkSync(f.module);
    fs.symlinkSync(outside, f.module);
  },
};
for (const [label, mutate] of Object.entries(mutations))
  test(label, async () => {
    const f = fixture();
    try {
      mutate(f);
      await assert.rejects(f.resolve());
    } finally {
      f.cleanup();
    }
  });

test("package identities without an executable retain an exact JSON representation", () => {
  const f = fixture();
  try {
    const result = packageProvider(f.consumer, "vize", released.version);
    assert.deepEqual(JSON.parse(JSON.stringify(result)), result);
  } finally {
    f.cleanup();
  }
});

test("changed provider bytes cannot reuse an earlier installed observation", async () => {
  const f = fixture();
  try {
    const before = await f.resolve();
    fs.appendFileSync(f.binary, "changed bytes\n");
    assert.notDeepEqual(await f.resolve(), before);
  } finally {
    f.cleanup();
  }
});
