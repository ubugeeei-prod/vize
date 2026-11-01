import { createHash } from "node:crypto";
import { posix as path } from "node:path";
import { writeTarGz } from "../../../tools/support/archive/tar.ts";
import { sourceInputBinding } from "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-inputs.ts";
import type {
  FileFacts,
  HumanBinFacts,
  PublicPackageFacts,
} from "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-types.ts";
import { pnpmSource } from "../../../tools/support/release/human_reports_11252_11253_public/pnpm-shim-render.ts";
import {
  globalBinConfigPath,
  workspaceInputPaths,
  type BinConfiguration,
} from "../../../tools/support/release/human_reports_11252_11253_public/bin-configuration.ts";
import {
  bytes,
  gitBlob,
  sha256,
} from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";

// Every consumer/package/path below is an inert synthetic fixture. Primary
// manager source bytes are pinned data; no manager or shell is executed.
const root = "/source-synthetic/consumer-original",
  parentDir = root + "/node_modules/vue",
  ownerDir = root + "/node_modules/typescript";
const image = (path: string, body: Buffer): FileFacts => ({
  path,
  realPath: path,
  kind: "file",
  body,
});
function pkg(
  name: string,
  version: string,
  directory: string,
  manifest: unknown,
  bodies: Record<string, Buffer>,
): PublicPackageFacts {
  const raw = Buffer.from(JSON.stringify(manifest) + "\n"),
    rows = { "package.json": raw, ...bodies };
  const archive = writeTarGz(
    Object.entries(rows).map(([name, data]) => ({
      name: "package/" + name,
      data,
    })),
  );
  return {
    name,
    version,
    directory,
    manifest: raw,
    archive,
    integrity: "sha512-" + createHash("sha512").update(archive).digest("base64"),
    resolved: `https://registry.npmjs.org/${name}/-/${name}-${version}.tgz`,
  };
}
/** Independent fixed expected source branch: c7b46a sh.rs:185. This is
 * specified before observed inventory; the production renderer is not called. */
function expectedNode(bin: string, canonical = false, self = false): Buffer {
  const relative = (self ? "../../bin/" : "../../../typescript/bin/") + bin,
    p = `"$basedir_abs/${relative}"`,
    w = `"$basedir_win/${relative}"`;
  const exec = `if [ -n "$exe" ] && [ -x "$basedir/node.exe" ]; then\n  exec "$basedir/node.exe"  ${w} "$@"\nelif [ -x "$basedir/node" ]; then\n  exec "$basedir/node"  ${p} "$@"\nelif [ -n "$msys" ] && command -v node >/dev/null 2>&1; then\n  exec node  ${w} "$@"\nelif command -v node >/dev/null 2>&1; then\n  exec node  ${p} "$@"\nelif [ -n "$exe" ] && command -v node.exe >/dev/null 2>&1; then\n  exec node.exe  ${w} "$@"\nelse\n  exec node  ${p} "$@"\nfi\n`;
  return Buffer.from(
    pnpmSource.header +
      pnpmSource.physicalPrelude +
      pnpmSource.platform +
      (canonical
        ? self
          ? 'if [ -z "$NODE_PATH" ]; then\n  export NODE_PATH="$basedir_abs/..:$basedir_abs/../../..:$basedir_abs/../../../.pnpm/node_modules"\nelse\n  export NODE_PATH="$basedir_abs/..:$basedir_abs/../../..:$basedir_abs/../../../.pnpm/node_modules:$NODE_PATH"\nfi\n'
          : 'if [ -z "$NODE_PATH" ]; then\n  export NODE_PATH="$basedir_abs/../../../typescript/node_modules:$basedir_abs/../../..:$basedir_abs/../../../.pnpm/node_modules"\nelse\n  export NODE_PATH="$basedir_abs/../../../typescript/node_modules:$basedir_abs/../../..:$basedir_abs/../../../.pnpm/node_modules:$NODE_PATH"\nfi\n'
        : "") +
      exec +
      `# cmd-shim-target=${canonical ? relative : ownerDir + "/bin/" + bin}\n`,
  );
}
function configurationFixture(
  inputRoot: string,
  platform: "linux" | "darwin",
  manifest: Buffer,
): BinConfiguration {
  const home = "/source-synthetic-home",
    paths = { HOME: home, XDG_CONFIG_HOME: null, LOCALAPPDATA: null, PNPM_HOME: null };
  const absent = (path: string) => ({
    path,
    state: "ABSENT" as const,
    kind: null,
    bytes: null,
    sha256: null,
    rawBase64: null,
  });
  const empty = (path: string) => ({
    path,
    state: "PRESENT" as const,
    kind: "file" as const,
    bytes: 0,
    sha256: sha256(""),
    rawBase64: "",
  });
  const explicitEnvironment = {
    npm_config_registry: "https://registry.npmjs.org",
    npm_config_userconfig: path.join(path.dirname(inputRoot), "original-user.npmrc"),
    npm_config_globalconfig: path.join(path.dirname(inputRoot), "original-global.npmrc"),
  };
  return {
    schema: "vize.humanPnpm12161Configuration.v1",
    sourceCommit: "c7b46aedef7b81c07816cb551187449a3dd81792",
    root: inputRoot,
    platform,
    home,
    pathEnvironment: paths,
    explicitEnvironment,
    configEnvironment: Object.entries(explicitEnvironment)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([name, value]) => ({
        name,
        value,
        present: true,
        empty: value === "",
        bytes: Buffer.byteLength(value),
        sha256: sha256(value),
      })),
    argv: ["install"],
    configDirPreseeded: false,
    global: absent(globalBinConfigPath(platform, home, paths)),
    userNpmrc: empty(explicitEnvironment.npm_config_userconfig),
    globalNpmrc: empty(explicitEnvironment.npm_config_globalconfig),
    rootNpmrc: absent(inputRoot + "/.npmrc"),
    rootPackage: {
      path: inputRoot + "/package.json",
      state: "PRESENT",
      kind: "file",
      bytes: manifest.length,
      sha256: sha256(manifest),
      rawBase64: manifest.toString("base64"),
    },
    workspace: workspaceInputPaths(inputRoot).map(absent),
    writerRuntimeAttribution: false,
  };
}
function fixture(role: HumanBinFacts["role"] = "imports-vue-ts-11253"): HumanBinFacts {
  const version = role === "imports-vue-ts-11253" ? "5.9.3" : "7.1.0-beta",
    names = role === "imports-vue-ts-11253" ? ["tsc", "tsserver"] : ["tsc"];
  const files = Object.fromEntries(
    Array.from({ length: 36 }, (_, i) => [
      `dist/inert-${i}.txt`,
      Buffer.from(`syntheticpayload${i}\n`),
    ]),
  );
  const parent = pkg(
    "vue",
    "3.5.44",
    parentDir,
    {
      name: "vue",
      version: "3.5.44",
      peerDependencies: { typescript: "*" },
      dependencies: Object.fromEntries(Array.from({ length: 5 }, (_, i) => [`dep-${i}`, "1.0.0"])),
    },
    files,
  );
  const targets = Object.fromEntries(
    names.map((name) => [
      "bin/" + name,
      Buffer.from("#!/usr/bin/env node\n// synthetic target " + name + "\n"),
    ]),
  );
  const owner = pkg(
    "typescript",
    version,
    ownerDir,
    {
      name: "typescript",
      version,
      bin: Object.fromEntries(names.map((name) => [name, "./bin/" + name])),
    },
    targets,
  );
  const managerDir = "/source-synthetic-manager/pnpm",
    entry = pnpmSource.managerSources.find((row) => row.path === "bin/pnpm.mjs")!,
    noBinOwners = Array.from({ length: 5 }, (_, i) =>
      pkg(
        `dep-${i}`,
        "1.0.0",
        root + `/node_modules/dep-${i}`,
        { name: `dep-${i}`, version: "1.0.0" },
        {},
      ),
    );
  const managerRaw = bytes(entry.base64),
    managerPath = managerDir + "/bin/pnpm.mjs";
  const facts: HumanBinFacts = {
    role,
    host:
      role === "imports-vue-ts-11253"
        ? { platform: "linux", arch: "x64" }
        : { platform: "darwin", arch: "arm64" },
    root,
    configuration: configurationFixture(
      root,
      role === "imports-vue-ts-11253" ? "linux" : "darwin",
      Buffer.from('{"name":"syntheticconsumer"}\n'),
    ),
    parent,
    owner,
    dependencyOwners: [owner, ...noBinOwners],
    graph: [
      {
        parentDirectory: parentDir,
        dependencyName: "typescript",
        version,
        directory: ownerDir,
      },
      ...noBinOwners.map((row) => ({
        parentDirectory: parentDir,
        dependencyName: row.name,
        version: row.version,
        directory: row.directory,
      })),
    ],
    lock: Buffer.from(
      `lockfileVersion: '9.0'\npackages:\n  'typescript@${version}':\n    resolution: {integrity: ${owner.integrity}}\n  'vue@3.5.44':\n    resolution: {integrity: ${parent.integrity}}\n` +
        noBinOwners
          .map(
            (row) =>
              `  '${row.name}@${row.version}':\n    resolution: {integrity: ${row.integrity}}\n`,
          )
          .join(""),
    ),
    manager: {
      reportedVersion: "12.16.1",
      versionStdout: Buffer.from("12.16.1\n"),
      versionStatus: 0,
      versionSignal: null,
      versionError: null,
      versionStderr: Buffer.alloc(0),
      originalArgv: ["install"],
      originalImage: {
        path: managerPath,
        bytes: managerRaw.length,
        sha256: sha256(managerRaw),
      },
      currentImage: image(managerPath, managerRaw),
    },
    expectedInputPolicyGitBlob: null,
    inputPolicyRaw: null,
    settings: {
      userConfig: Buffer.alloc(0),
      globalConfig: Buffer.alloc(0),
      rootNpmrc: null,
      rootWorkspace: Buffer.alloc(0),
      rootPackage: Buffer.from('{"name":"syntheticconsumer"}\n'),
    },
    targets: names.map((name) => image(ownerDir + "/bin/" + name, targets["bin/" + name]!)),
    inventory: [],
  };
  const plan = {
    schema: "vize.humanPnpm12161InputPolicy.v1",
    sourceCommit: pnpmSource.sourceCommit,
    bindings: { [role]: sourceInputBinding(facts) },
    selfBindings: {
      [role]: sourceInputBinding({
        ...facts,
        parent: facts.owner,
        dependencyOwners: [],
        graph: [],
      }),
    },
    options: {
      extraNodePaths: ["CONSUMER_VIRTUAL_NODE_MODULES"],
      preferSymlinkedExecutables: false,
      projectModulesDirName: null,
      relocatableRoot: "CONSUMER_ROOT",
    },
  };
  facts.inputPolicyRaw = Buffer.from(JSON.stringify(plan) + "\n");
  facts.expectedInputPolicyGitBlob = gitBlob(facts.inputPolicyRaw);
  const inventory = new Map<string, FileFacts>();
  for (const [relative, body] of Object.entries({
    "package.json": parent.manifest,
    ...files,
  }))
    inventory.set(parentDir + "/" + relative, image(parentDir + "/" + relative, body));
  for (const name of names)
    inventory.set(
      parentDir + "/node_modules/.bin/" + name,
      image(parentDir + "/node_modules/.bin/" + name, expectedNode(name, true)),
    );
  // Preserve the authored file population while structural directories are
  // inserted into the same map below.
  const authoredFiles = [...inventory.keys()];
  for (const file of authoredFiles) {
    let directory = path.dirname(file);
    while (directory !== parentDir) {
      inventory.set(directory, {
        path: directory,
        realPath: directory,
        kind: "directory",
        body: null,
      });
      directory = path.dirname(directory);
    }
  }
  facts.inventory = [...inventory.values()];
  return facts;
}
function mutatePlan(
  facts: HumanBinFacts,
  mutate: (plan: Record<string, unknown>) => void,
  bind = false,
): void {
  const plan = JSON.parse(facts.inputPolicyRaw!.toString("utf8")) as Record<string, unknown>;
  mutate(plan);
  facts.inputPolicyRaw = Buffer.from(JSON.stringify(plan) + "\n");
  if (bind) facts.expectedInputPolicyGitBlob = gitBlob(facts.inputPolicyRaw);
}

export {
  root,
  parentDir,
  ownerDir,
  image,
  pkg,
  fixture,
  mutatePlan,
  expectedNode,
  configurationFixture,
};
