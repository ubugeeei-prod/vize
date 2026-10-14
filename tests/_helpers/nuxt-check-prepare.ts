import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

import { execNuxtPrepareNpx, selectNuxtPrepareRuntime } from "./nuxt-check-runtime.ts";
import { prepareNuxtPrerequisites, retainNuxtPrerequisites } from "./nuxt-check-prerequisites.ts";
import {
  assertNoSourceMutation,
  blobOid,
  git,
  lockName,
  nearestPackage,
  readAuthored,
  restoreAuthored,
  walkFiles,
  type AuthoredFile,
} from "./nuxt-check-source.ts";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type CommandRunner = (
  args: string[],
  options: { cwd: string; env: NodeJS.ProcessEnv; timeout: number },
) => void;

interface PrepareOptions {
  name: string;
  sourceDir: string;
  cwd?: string;
  /** The existing project's preparation command, without any source patches. */
  prepareArgs?: string[];
  prepareTimeoutMs?: number;
  installTimeoutMs?: number;
  env?: NodeJS.ProcessEnv;
  authorityRoot?: string;
  captureDir?: string;
  run?: CommandRunner;
}

/** Prepare the program being checked, never the separately patched visual app. */
export function prepareNuxtCheckFixture(options: PrepareOptions): string {
  const cwd = path.resolve(options.cwd ?? options.sourceDir);
  const sourceDir = path.resolve(options.sourceDir);
  const authorityRoot = options.authorityRoot ?? repositoryRoot;
  assert.match(options.name, /^[a-z0-9][a-z0-9.-]*$/);
  const fixturePath = `tests/_fixtures/_git/${options.name}`;
  const pin = git(authorityRoot, ["ls-tree", "HEAD", "--", fixturePath]).trim();
  const match = /^160000 commit ([0-9a-f]{40})\t(.+)$/.exec(pin);
  assert.ok(match && match[2] === fixturePath, `${fixturePath} must be an authored Gitlink`);
  assert.equal(sourceDir, path.resolve(authorityRoot, fixturePath));
  assert.equal(git(sourceDir, ["rev-parse", "HEAD"]).trim(), match[1], "fixture pin changed");

  const authored = git(sourceDir, ["ls-tree", "-rz", "HEAD"])
    .split("\0")
    .filter(Boolean)
    .map((entry): AuthoredFile => {
      const parsed = /^(\d+) blob ([0-9a-f]{40})\t([\s\S]+)$/.exec(entry);
      assert.ok(parsed, `unsupported nested Gitlink in checked fixture: ${entry}`);
      const file = parsed[3];
      const bytes = readAuthored(cwd, file, parsed[1]);
      assert.equal(blobOid(bytes), parsed[2], `checked source differs from its pin: ${file}`);
      return { file, mode: parsed[1], oid: parsed[2], bytes };
    });
  const owner = authored.find((entry) => entry.file === "package.json");
  assert.ok(owner, "checked fixture must own a package manifest");
  const packageJson = JSON.parse(owner.bytes.toString("utf8"));
  assert.ok(
    packageJson.dependencies?.nuxt || packageJson.devDependencies?.nuxt,
    "checked fixture has no authored Nuxt provider",
  );
  const mutex = path.join(cwd, ".vize-check-prepare-lock");
  fs.mkdirSync(mutex);
  const destination = options.captureDir ?? process.env.VIZE_APP_CHECK_CAPTURE_DIR;
  const receiptDir = destination
    ? path.join(destination, options.name, "preparation")
    : fs.mkdtempSync(path.join(os.tmpdir(), `vize-${options.name}-check-prepare-`));
  fs.mkdirSync(path.join(receiptDir, "bytes"), { recursive: true });
  const files: Array<Record<string, unknown>> = [];
  const retained = new Set<string>();
  const declarationPackages = new Set<string>();
  const packageResolutions: Array<Record<string, unknown>> = [];
  const providerRoots = new Set<string>();
  const missing: Array<Record<string, unknown>> = [];
  const commands: Array<Record<string, unknown>> = [];
  const sourceMutations: string[] = [];
  const lockChanges: Array<Record<string, unknown>> = [];
  let env: NodeJS.ProcessEnv = {
    ...process.env,
    ...options.env,
    GIT_CEILING_DIRECTORIES: path.dirname(cwd),
    HUSKY: "0",
    SKIP_INSTALL_SIMPLE_GIT_HOOKS: "1",
  };
  const run = options.run ?? execNuxtPrepareNpx;
  function retain(file: string, role: string, bytes?: Buffer): void {
    try {
      const content = bytes ?? fs.readFileSync(file);
      const sha256 = createHash("sha256").update(content).digest("hex");
      const identity = `${role}\0${file}\0${sha256}`;
      if (retained.has(identity)) return;
      retained.add(identity);
      fs.writeFileSync(path.join(receiptDir, "bytes", sha256), content);
      files.push({
        role,
        path: file,
        physical: fs.existsSync(file) ? fs.realpathSync(file) : null,
        sha256,
        size: content.length,
        retainedBytes: `bytes/${sha256}`,
      });
    } catch (error) {
      missing.push({ role, path: file, error: String(error) });
    }
  }
  function command(args: string[], timeout: number): void {
    const observation: Record<string, unknown> = { command: ["npx", ...args], cwd, timeout };
    commands.push(observation);
    try {
      run(args, { cwd, env, timeout });
      observation.completed = true;
    } catch (error) {
      observation.completed = false;
      observation.error = String(error);
      throw error;
    }
  }
  for (const entry of authored) {
    if (
      /package\.json$|(?:ts|js)config[^/]*\.json$|(?:^|\/)modules\/|(?:^|\/)nuxt\.config\./.test(
        entry.file,
      ) ||
      lockName.test(entry.file) ||
      (options.name === "npmx.dev" && /^(?:vite\.config\.|scripts\/|lexicons\/)/.test(entry.file))
    ) {
      retain(path.join(cwd, entry.file), "authored-before", entry.bytes);
    }
  }
  let failure: unknown;
  let runtime: ReturnType<typeof selectNuxtPrepareRuntime>["observation"] | undefined;
  try {
    const selected = selectNuxtPrepareRuntime(packageJson.engines?.node, env);
    env = selected.env;
    runtime = selected.observation;
    // Retain the actual unlocked result; do not describe this as a frozen install.
    // Lifecycle scripts can rewrite authored app sources. The explicit Nuxt
    // preparation below invokes the real modules without those unrelated scripts.
    const install = [
      "-y",
      "pnpm@10",
      "install",
      "--no-frozen-lockfile",
      "--ignore-scripts",
      "--prefer-offline",
    ];
    if (!fs.existsSync(path.join(cwd, "pnpm-workspace.yaml"))) install.push("--ignore-workspace");
    command(install, options.installTimeoutMs ?? 600_000);
    for (const entry of authored.filter((entry) => lockName.test(entry.file))) {
      retain(path.join(cwd, entry.file), "resolved-install-lock");
    }
    assertNoSourceMutation(authored, cwd);
    assert.ok(
      fs.existsSync(path.join(cwd, "node_modules/nuxt/package.json")),
      "the checked fixture must install its own Nuxt provider",
    );
    for (const name of ["nuxt", "nuxt-module-build"]) {
      const file = path.join(cwd, "node_modules/.bin", name);
      if (fs.existsSync(file)) retain(file, "installed-cli-candidate");
    }
    prepareNuxtPrerequisites(options.name, packageJson.scripts?.postinstall, (args, timeout) => {
      command(args, timeout);
      assertNoSourceMutation(authored, cwd);
    });
    command(
      options.prepareArgs ?? ["-y", "pnpm@10", "exec", "nuxt", "prepare"],
      options.prepareTimeoutMs ?? 180_000,
    );
    assertNoSourceMutation(authored, cwd);
  } catch (error) {
    failure = error;
  } finally {
    try {
      retainNuxtPrerequisites(options.name, cwd, retain);
      // Capture full generated type/config/reference bytes and installed declaration
      // candidates. Native exec-time selection is owned by the checker descriptor.
      for (const entry of authored.filter((entry) => entry.file.endsWith("package.json"))) {
        const owner = path.dirname(path.join(cwd, entry.file));
        providerRoots.add(owner);
        const manifest = JSON.parse(entry.bytes.toString("utf8"));
        for (const name of [
          "pnpm-lock.yaml",
          "package-lock.json",
          "yarn.lock",
          "node_modules/.modules.yaml",
          "node_modules/.pnpm/lock.yaml",
        ]) {
          const file = path.join(owner, name);
          if (fs.existsSync(file)) retain(file, "resolved-install-contract");
        }
        const requests = {
          ...manifest.dependencies,
          ...manifest.devDependencies,
          ...manifest.optionalDependencies,
        };
        for (const name of Object.keys(requests)) {
          const installed = nearestPackage(owner, name);
          packageResolutions.push({
            owner,
            name,
            request: requests[name],
            observedDirectory: installed,
          });
          if (!installed) {
            missing.push({ role: "installed-package-candidate", owner, name });
            continue;
          }
          retain(path.join(installed, "package.json"), "installed-package-manifest");
          if (!declarationPackages.has(installed)) {
            declarationPackages.add(installed);
            walkFiles(
              installed,
              (file) => {
                if (/\.d\.(?:ts|mts|cts)$/.test(file))
                  retain(file, "installed-declaration-candidate");
              },
              new Set(["node_modules", ".git"]),
            );
          }
        }
      }
      for (const entry of authored.filter((entry) =>
        /(?:^|\/)nuxt\.config\.(?:[cm]?ts|[cm]?js)$/.test(entry.file),
      )) {
        providerRoots.add(path.dirname(path.join(cwd, entry.file)));
      }
      for (const root of providerRoots) {
        walkFiles(path.join(root, ".nuxt"), (file) => retain(file, "nuxt-generated-provider"));
      }
    } catch (error) {
      failure ??= error;
      missing.push({ role: "preparation-capture", error: String(error) });
    }
    for (const entry of authored) {
      const file = path.join(cwd, entry.file);
      let current: Buffer | null = null;
      let kindOrModeChanged = false;
      try {
        current = readAuthored(cwd, entry.file, entry.mode);
      } catch {
        kindOrModeChanged = true;
        try {
          const stat = fs.lstatSync(file);
          current = stat.isSymbolicLink()
            ? Buffer.from(fs.readlinkSync(file))
            : fs.readFileSync(file);
        } catch (error) {
          missing.push({ role: "authored-command-mutation", path: file, error: String(error) });
        }
      }
      if (!kindOrModeChanged && current?.equals(entry.bytes)) continue;
      retain(file, "authored-command-mutation", current ?? Buffer.alloc(0));
      if (lockName.test(entry.file)) {
        lockChanges.push({
          file: entry.file,
          beforeOid: entry.oid,
          resolvedOid: current ? blobOid(current) : null,
        });
      } else {
        sourceMutations.push(entry.file);
      }
      restoreAuthored(cwd, entry);
    }
    assertNoSourceMutation(authored, cwd, true);
    fs.writeFileSync(
      path.join(receiptDir, "receipt.json"),
      JSON.stringify(
        {
          schema: "vize.nuxtCheckPreparation.v1",
          name: options.name,
          revision: match[1],
          sourceDir,
          checkedRoot: cwd,
          authoredFiles: authored.map(({ file, mode, oid, bytes }) => ({
            file,
            mode,
            oid,
            size: bytes.length,
          })),
          authoredPreserved: true,
          lockedInstallation: false,
          runtime,
          commands,
          lockChanges,
          sourceMutations,
          packageResolutions,
          files,
          missing,
          failed: failure !== undefined || sourceMutations.length > 0,
          authorityUnproved: [
            "native selected config and dependency reads",
            "installed tarball bytes versus lockfile SRI",
          ],
        },
        null,
        2,
      ),
    );
    fs.rmdirSync(mutex);
    console.log(`[${options.name}:check:prepare] authority retained in ${receiptDir}`);
  }
  if (failure !== undefined) throw failure;
  assert.deepEqual(sourceMutations, [], "Nuxt preparation rewrote authored fixture source");
  assert.equal(git(sourceDir, ["rev-parse", "HEAD"]).trim(), match[1], "fixture pin changed");
  return cwd;
}
