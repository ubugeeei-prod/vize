import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

import { execBoundPrepareNpx, inspectPrepareNode } from "./nuxt-check-runtime.ts";
import {
  assertNoSourceMutation,
  blobOid,
  git,
  readAuthored,
  restoreAuthored,
  type AuthoredFile,
} from "./nuxt-check-source.ts";
import {
  inspectRekaNpx,
  inspectRekaProviders,
  rekaProviderScopes,
} from "./reka-check-providers.ts";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixturePath = "tests/_fixtures/_git/reka-ui";
export const rekaFixturePin = "c23e0ed8a38c66cd4f9350e7a46c6b502a8491c2";

interface PrepareOptions {
  sourceDir?: string;
  authorityRoot?: string;
  captureDir?: string;
  env?: NodeJS.ProcessEnv;
  node?: string;
  /** An explicit synthetic Gitlink for guard tests, never a production fallback. */
  expectedPin?: string;
  run?: typeof execBoundPrepareNpx;
}

/** Populate original providers before both the full App row and native replay. */
export function prepareRekaCheckFixture(options: PrepareOptions = {}): string {
  const authorityRoot = path.resolve(options.authorityRoot ?? repositoryRoot);
  const cwd = path.resolve(options.sourceDir ?? path.join(authorityRoot, fixturePath));
  assert.equal(
    cwd,
    path.join(authorityRoot, fixturePath),
    "ReKa must use its original authored root",
  );
  const pin = git(authorityRoot, ["ls-tree", "HEAD", "--", fixturePath]).trim();
  assert.equal(pin, `160000 commit ${options.expectedPin ?? rekaFixturePin}\t${fixturePath}`);
  assert.equal(git(cwd, ["rev-parse", "HEAD"]).trim(), options.expectedPin ?? rekaFixturePin);
  const authored = git(cwd, ["ls-tree", "-rz", "HEAD"])
    .split("\0")
    .filter(Boolean)
    .map((entry): AuthoredFile => {
      const parsed = /^(\d+) blob ([0-9a-f]{40})\t([\s\S]+)$/.exec(entry);
      assert.ok(parsed, `unsupported authored ReKa entry: ${entry}`);
      const bytes = readAuthored(cwd, parsed[3], parsed[1]);
      assert.equal(blobOid(bytes), parsed[2], `ReKa source differs from its pin: ${parsed[3]}`);
      return { file: parsed[3], mode: parsed[1], oid: parsed[2], bytes };
    });
  const manifest = JSON.parse(fs.readFileSync(path.join(cwd, "package.json"), "utf8"));
  assert.match(manifest.packageManager, /^pnpm@\d+\.\d+\.\d+$/);
  assert.equal(
    manifest.scripts.prepare,
    "pnpm simple-git-hooks",
    "unexpected authored install hook",
  );
  const capture =
    options.captureDir ??
    process.env.VIZE_APP_CHECK_CAPTURE_DIR ??
    path.join(authorityRoot, "target/app-e2e-logs");
  const receiptDir = path.join(capture, "reka-ui/preparation");
  fs.mkdirSync(path.join(receiptDir, "bytes"), { recursive: true });
  const files: Array<Record<string, unknown>> = [];
  function retain(file: string, role: string) {
    const bytes = fs.readFileSync(file);
    const sha256 = createHash("sha256").update(bytes).digest("hex");
    fs.writeFileSync(path.join(receiptDir, "bytes", sha256), bytes);
    files.push({
      file,
      role,
      physical: fs.realpathSync(file),
      sha256,
      size: bytes.length,
      retainedBytes: `bytes/${sha256}`,
    });
  }
  for (const entry of authored) {
    if (
      /package\.json$|(?:ts|js)config[^/]*\.json$|pnpm-(?:lock|workspace)\.yaml$/.test(entry.file)
    )
      retain(path.join(cwd, entry.file), "authored-before");
  }
  const mutex = path.join(cwd, ".vize-check-prepare-lock");
  fs.mkdirSync(mutex);
  const sourceMutations: string[] = [];
  let runtime: ReturnType<typeof inspectPrepareNode>["observation"] | undefined;
  let npm: ReturnType<typeof inspectRekaNpx> | undefined;
  let providers: ReturnType<typeof inspectRekaProviders> | undefined;
  const command = {
    args: [
      "-y",
      manifest.packageManager,
      "install",
      "--frozen-lockfile",
      "--ignore-scripts",
      "--prefer-offline",
    ],
    cwd,
    timeout: 600_000,
    completed: false,
  };
  let failure: unknown;
  try {
    const selected = inspectPrepareNode(options.node ?? process.execPath, {
      ...process.env,
      ...options.env,
    });
    runtime = selected.observation;
    npm = inspectRekaNpx(selected.env);
    retain(npm.cli, "executed-npm-cli");
    retain(npm.manifest, "executed-npm-manifest");
    (options.run ?? execBoundPrepareNpx)(runtime.executable, npm.cli, command.args, {
      cwd,
      env: selected.env,
      timeout: command.timeout,
    });
    command.completed = true;
    assertNoSourceMutation(authored, cwd, true);
    assert.deepEqual(
      inspectPrepareNode(runtime.executable, selected.env).observation,
      runtime,
      "ReKa preparation runtime changed during install",
    );
    assert.deepEqual(inspectRekaNpx(selected.env), npm, "ReKa preparation npm CLI changed");
    for (const file of [
      "pnpm-lock.yaml",
      "node_modules/.modules.yaml",
      "node_modules/.pnpm/lock.yaml",
    ])
      retain(path.join(cwd, file), "resolved-install-contract");
    providers = inspectRekaProviders(cwd, retain);
  } catch (error) {
    failure = error;
  } finally {
    for (const entry of authored) {
      let unchanged = false;
      try {
        unchanged = blobOid(readAuthored(cwd, entry.file, entry.mode)) === entry.oid;
      } catch {
        /* retain absence below */
      }
      if (!unchanged) {
        sourceMutations.push(entry.file);
        const file = path.join(cwd, entry.file);
        if (fs.existsSync(file) && fs.statSync(file).isFile())
          retain(file, "rejected-source-mutation");
        restoreAuthored(cwd, entry);
      }
    }
    if (sourceMutations.length && !failure) failure = new Error("authored ReKa source changed");
    fs.writeFileSync(
      path.join(receiptDir, "receipt.json"),
      JSON.stringify(
        {
          fixture: "reka-ui",
          pin: options.expectedPin ?? rekaFixturePin,
          cwd,
          runtime,
          npm,
          command,
          ignoredLifecycle: {
            prepare: manifest.scripts.prepare,
            reason: "Git hooks only; no generated source provider",
          },
          providers,
          providerScopes: rekaProviderScopes(cwd, authorityRoot),
          files,
          sourceMutations,
          authoredPreserved: sourceMutations.length === 0,
          failed: Boolean(failure),
          error:
            failure instanceof Error ? failure.stack : failure ? JSON.stringify(failure) : null,
          qualification:
            "provider preparation only; native same-client diagnostics remain a separate gate",
        },
        null,
        2,
      ),
    );
    fs.rmdirSync(mutex);
  }
  if (failure) throw failure;
  return receiptDir;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  console.log(prepareRekaCheckFixture());
}
