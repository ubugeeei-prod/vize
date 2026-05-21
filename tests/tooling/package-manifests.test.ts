import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const npmDir = path.join(root, "npm");

function collectStrings(value: unknown, out: string[]): void {
  if (typeof value === "string") {
    out.push(value);
    return;
  }

  if (Array.isArray(value)) {
    for (const item of value) collectStrings(item, out);
    return;
  }

  if (value != null && typeof value === "object") {
    for (const item of Object.values(value)) collectStrings(item, out);
  }
}

function isEsmPackPackage(packageDir: string): boolean {
  const configPath = path.join(packageDir, "vite.config.ts");
  if (!fs.existsSync(configPath)) return false;

  const config = fs.readFileSync(configPath, "utf-8");
  return config.includes('format: "esm"') && config.includes("pack:");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function collectNativeBinaryCatalogPins(workspaceYaml: string): Record<string, string> {
  const pins: Record<string, string> = {};
  let inNativeCatalog = false;

  for (const line of workspaceYaml.split("\n")) {
    if (line === "  native-binaries:") {
      inNativeCatalog = true;
      continue;
    }

    if (inNativeCatalog && /^\S|^  [^\s]/.test(line)) {
      break;
    }

    if (!inNativeCatalog) {
      continue;
    }

    const match = line.match(/^    "(@vizejs\/native-[^"]+)": "([^"]+)"$/);
    if (match) {
      pins[match[1]] = match[2];
    }
  }

  return pins;
}

function readRepoFile(filePath: string): string {
  return fs.readFileSync(path.join(root, filePath), "utf-8");
}

function releaseWorkflowJobBlock(jobName: string): string {
  const workflow = readRepoFile(".github/workflows/release.yml");
  const startMarker = `  ${jobName}:\n`;
  const start = workflow.indexOf(startMarker);
  assert.notEqual(start, -1, `missing release workflow job ${jobName}`);

  const rest = workflow.slice(start + startMarker.length);
  const nextJob = rest.search(/\n  [a-zA-Z0-9_-]+:\n/);
  return nextJob === -1 ? rest : rest.slice(0, nextJob);
}

function parseNeedsList(value: string): string[] {
  return value.match(/[a-zA-Z0-9][a-zA-Z0-9_-]*/g) ?? [];
}

function releaseWorkflowNeeds(jobName: string): string[] {
  const block = releaseWorkflowJobBlock(jobName);
  const inline = block.match(/^    needs:\s*\[(?<needs>[^\]]+)\]/m)?.groups?.needs;
  if (inline != null) {
    return parseNeedsList(inline);
  }

  const lines = block.split("\n");
  const needsLine = lines.findIndex((line) => /^    needs:\s*$/.test(line));
  assert.notEqual(needsLine, -1, `missing needs for release workflow job ${jobName}`);

  const needsBlock: string[] = [];
  for (const line of lines.slice(needsLine + 1)) {
    if (/^    \S/.test(line)) break;
    needsBlock.push(line);
  }
  return parseNeedsList(needsBlock.join("\n"));
}

test("esm packed npm manifests point at mjs and d.mts outputs", () => {
  const failures: string[] = [];

  for (const entry of fs.readdirSync(npmDir, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;

    const packageDir = path.join(npmDir, entry.name);
    if (!isEsmPackPackage(packageDir)) continue;

    const packageJsonPath = path.join(packageDir, "package.json");
    if (!fs.existsSync(packageJsonPath)) continue;

    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8")) as {
      bin?: unknown;
      exports?: unknown;
      main?: unknown;
      name?: string;
      types?: unknown;
    };

    const publishedPaths: string[] = [];
    collectStrings(packageJson.main, publishedPaths);
    collectStrings(packageJson.types, publishedPaths);
    collectStrings(packageJson.bin, publishedPaths);
    collectStrings(packageJson.exports, publishedPaths);

    for (const publishedPath of publishedPaths) {
      if (!publishedPath.includes("dist/")) continue;

      if (publishedPath.endsWith(".js") || publishedPath.endsWith(".d.ts")) {
        failures.push(`${packageJson.name ?? entry.name}: ${publishedPath}`);
      }
    }
  }

  assert.deepEqual(failures, []);
});

test("native package catalog pins and generated loader version checks stay aligned", () => {
  const nativePackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/vize-native/package.json"), "utf-8"),
  ) as {
    optionalDependencies?: Record<string, string>;
    version?: string;
  };
  assert.ok(nativePackage.version);

  const workspacePins = collectNativeBinaryCatalogPins(
    fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf-8"),
  );
  const nativeOptionalDependencies = Object.entries(
    nativePackage.optionalDependencies ?? {},
  ).filter(([name]) => name.startsWith("@vizejs/native-"));

  assert.ok(nativeOptionalDependencies.length > 0);
  assert.deepEqual(
    Object.fromEntries(nativeOptionalDependencies),
    Object.fromEntries(
      nativeOptionalDependencies.map(([name]) => [name, "catalog:native-binaries"]),
    ),
  );

  const lockfile = fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf-8");
  for (const [name] of nativeOptionalDependencies) {
    const escapedName = escapeRegExp(name);
    const catalogVersion = workspacePins[name];
    assert.ok(catalogVersion, `${name} catalog pin`);
    assert.equal(
      catalogVersion,
      nativePackage.version,
      `${name} catalog pin should match @vizejs/native version`,
    );
    const escapedVersion = escapeRegExp(catalogVersion);
    assert.match(
      lockfile,
      new RegExp(`['"]?${escapedName}['"]?:\\n\\s+specifier: ${escapedVersion}\\n`),
      `${name} lockfile catalog specifier should match @vizejs/native version`,
    );
  }

  const nativeTargetsLoader = fs.readFileSync(
    path.join(root, "npm/vize-native/native-targets.js"),
    "utf-8",
  );
  assert.match(
    nativeTargetsLoader,
    /const packageVersion = require\("\.\/package\.json"\)\.version;/,
  );
  assert.match(nativeTargetsLoader, /bindingPackageVersion !== packageVersion/);
  assert.match(nativeTargetsLoader, /expected \$\{packageVersion\} but got/);
  assert.doesNotMatch(nativeTargetsLoader, /bindingPackageVersion !== "[^"]+"/);
  assert.match(nativeTargetsLoader, /execFileSync\("ldd", \["--version"\]/);
  assert.doesNotMatch(nativeTargetsLoader, /execSync\(/);

  const oxlintNativeLoader = fs.readFileSync(
    path.join(root, "npm/oxlint-plugin-vize/src/native.ts"),
    "utf-8",
  );
  assert.match(oxlintNativeLoader, /spawnSync\("ldd", \["--version"\]/);
  assert.doesNotMatch(oxlintNativeLoader, /execSync\(/);
});

test("pkl runtime stays optional for consumers of the vize package", () => {
  const packageJson = JSON.parse(readRepoFile("npm/vize/package.json")) as {
    dependencies?: Record<string, string>;
    devDependencies?: Record<string, string>;
    optionalDependencies?: Record<string, string>;
    peerDependencies?: Record<string, string>;
    peerDependenciesMeta?: Record<string, { optional?: boolean }>;
  };

  assert.equal(packageJson.dependencies?.["@pkl-community/pkl"], undefined);
  assert.equal(packageJson.optionalDependencies?.["@pkl-community/pkl"], undefined);
  assert.equal(packageJson.devDependencies?.["@pkl-community/pkl"], "catalog:repo-tooling");
  assert.equal(packageJson.peerDependencies?.["@pkl-community/pkl"], "catalog:repo-tooling");
  assert.deepEqual(packageJson.peerDependenciesMeta?.["@pkl-community/pkl"], {
    optional: true,
  });
});

test("musea Nuxt tests the same vue-router major that it declares as a peer", () => {
  const workspaceYaml = readRepoFile("pnpm-workspace.yaml");
  const catalogVersion = workspaceYaml.match(/^\s+vue-router: "([^"]+)"$/m)?.[1];
  assert.ok(catalogVersion, "vue-router catalog pin");

  const packageJson = JSON.parse(readRepoFile("npm/musea-nuxt/package.json")) as {
    peerDependencies?: Record<string, string>;
  };
  const peerRange = packageJson.peerDependencies?.["vue-router"];
  assert.ok(peerRange, "vue-router peer range");

  assert.match(catalogVersion, /^4\./);
  assert.match(peerRange, /\^4\./);
});

test("published package manifests declare support metadata", () => {
  const failures: string[] = [];

  for (const entry of fs.readdirSync(npmDir, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;

    const packageDir = path.join("npm", entry.name);
    const packageJsonPath = path.join(root, packageDir, "package.json");
    if (!fs.existsSync(packageJsonPath)) continue;

    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8")) as {
      bugs?: { url?: string };
      homepage?: string;
      name?: string;
      private?: boolean;
    };
    if (packageJson.private === true) continue;

    if (packageJson.bugs?.url !== "https://github.com/ubugeeei/vize/issues") {
      failures.push(`${packageJson.name ?? packageDir}: missing canonical bugs.url`);
    }
    if (packageJson.homepage !== "https://github.com/ubugeeei/vize") {
      failures.push(`${packageJson.name ?? packageDir}: missing canonical homepage`);
    }
  }

  assert.deepEqual(failures, []);
});

test("documented install commands point at supported release artifacts", () => {
  const vizeNpmPackage = JSON.parse(readRepoFile("npm/vize/package.json")) as {
    bin?: Record<string, string>;
    publishConfig?: Record<string, string>;
  };
  const vizeCrateToml = readRepoFile("crates/vize/Cargo.toml");
  const releaseWorkflow = readRepoFile(".github/workflows/release.yml");
  const flake = readRepoFile("flake.nix");
  const publicDocs = [
    "README.md",
    "docs/content/getting-started.md",
    "docs/content/guide/cli.md",
    "docs/content/guide/static-analysis.md",
    "npm/vize/README.md",
    "crates/vize/README.md",
  ];
  const unsupportedCargoInstallDocs = publicDocs.filter((filePath) =>
    readRepoFile(filePath).includes("cargo install vize"),
  );

  assert.equal(vizeNpmPackage.publishConfig?.access, "public");
  assert.equal(vizeNpmPackage.bin?.vize, "bin/vize");
  assert.match(releaseWorkflow, /cli-artifacts\/\*\.tar\.gz/);
  assert.match(flake, /apps = \{/);
  assert.match(flake, /vize = flake-utils\.lib\.mkApp \{ drv = vize; \};/);

  if (/^publish = false$/m.test(vizeCrateToml)) {
    assert.deepEqual(unsupportedCargoInstallDocs, []);
  }
});

test("release workflow publishes npm packages after their npm dependencies", () => {
  const vitePluginNeeds = releaseWorkflowNeeds("release-npm-vite-plugin");
  const nuxtNeeds = releaseWorkflowNeeds("release-npm-nuxt");

  assert.ok(vitePluginNeeds.includes("release-npm-cli"));
  assert.ok(nuxtNeeds.includes("release-npm-cli"));
  assert.ok(nuxtNeeds.includes("release-npm-vite-plugin"));
  assert.ok(nuxtNeeds.includes("release-npm-vite-plugin-musea"));
  assert.ok(nuxtNeeds.includes("release-npm-musea-nuxt"));
});

test("editor extension manifests stay opt-in and version aligned", () => {
  const workspaceVersion = fs
    .readFileSync(path.join(root, "Cargo.toml"), "utf-8")
    .match(/^version = "(.+)"$/m)?.[1];
  assert.ok(workspaceVersion);

  const vscodePackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/vscode-vize/package.json"), "utf-8"),
  ) as {
    contributes?: {
      configuration?: {
        properties?: Record<string, { default?: unknown }>;
      };
    };
    devDependencies?: Record<string, string>;
    scripts?: Record<string, string>;
    version?: string;
  };

  assert.equal(vscodePackage.version, workspaceVersion);
  assert.equal(
    vscodePackage.contributes?.configuration?.properties?.["vize.enable"]?.default,
    false,
  );
  assert.equal(
    vscodePackage.contributes?.configuration?.properties?.["vize.lint.enable"]?.default,
    false,
  );
  assert.equal(
    vscodePackage.contributes?.configuration?.properties?.["vize.typecheck.enable"]?.default,
    false,
  );
  assert.equal(
    vscodePackage.contributes?.configuration?.properties?.["vize.editor.enable"]?.default,
    false,
  );
  assert.equal(vscodePackage.scripts?.["vscode:prepublish"], "vp pack");
  assert.equal(vscodePackage.scripts?.build, "vp pack");
  assert.equal(vscodePackage.scripts?.watch, "vp pack --watch");
  assert.equal(vscodePackage.scripts?.check, "tsgo --noEmit && vp check src vite.config.ts");
  assert.equal(
    vscodePackage.scripts?.["check:fix"],
    "vp check --fix src vite.config.ts && tsgo --noEmit",
  );
  assert.equal(
    vscodePackage.devDependencies?.["@typescript/native-preview"],
    "7.0.0-dev.20260421.1",
  );

  const zedManifest = fs.readFileSync(path.join(root, "npm/zed-vize/extension.toml"), "utf-8");
  const zedVersion = zedManifest.match(/^version = "(.+)"$/m)?.[1];

  assert.equal(zedVersion, workspaceVersion);
  assert.match(zedManifest, /^\[language_servers\.vize\]$/m);
  assert.match(zedManifest, /^languages = \["Vue", "Art Vue"\]$/m);
});

test("workspace package builds do not nest pnpm run commands", () => {
  const museaPackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/vite-plugin-musea/package.json"), "utf-8"),
  ) as {
    scripts?: Record<string, string>;
  };

  assert.equal(museaPackage.scripts?.build, "vp pack && vp build --config gallery-vite.config.ts");
  assert.equal(museaPackage.scripts?.dev, "vp pack --watch");
  assert.doesNotMatch(museaPackage.scripts?.build ?? "", /\bpnpm run\b/);
});

test("vize package delegates rule type generation to the workspace MoonBit task", () => {
  const vizePackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/vize/package.json"), "utf-8"),
  ) as {
    engines?: Record<string, string>;
    scripts?: Record<string, string>;
  };

  assert.equal(vizePackage.engines?.node, ">=22");
  assert.equal(
    vizePackage.scripts?.["generate:rule-types"],
    "vp run --workspace-root generate:rule-types",
  );
  assert.equal(
    vizePackage.scripts?.build,
    "vp run --workspace-root generate:rule-types && vp pack",
  );
});

test("wasm package publishes the wrapper entrypoint and raw wasm assets", () => {
  const wasmPackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/vize-wasm/package.json"), "utf-8"),
  ) as {
    exports?: Record<string, string | Record<string, string>>;
    files?: string[];
    main?: string;
    publishConfig?: Record<string, string>;
    types?: string;
  };

  assert.equal(wasmPackage.publishConfig?.access, "public");
  assert.equal(wasmPackage.main, "./index.js");
  assert.equal(wasmPackage.types, "./index.d.ts");
  assert.deepEqual(wasmPackage.exports?.["."], {
    types: "./index.d.ts",
    import: "./index.js",
    default: "./index.js",
  });
  assert.deepEqual(wasmPackage.exports?.["./vize_vitrine.js"], {
    types: "./vize_vitrine.d.ts",
    import: "./vize_vitrine.js",
    default: "./vize_vitrine.js",
  });
  assert.equal(wasmPackage.exports?.["./vize_vitrine_bg.wasm"], "./vize_vitrine_bg.wasm");

  for (const file of [
    "index.js",
    "index.d.ts",
    "vize_vitrine.js",
    "vize_vitrine.d.ts",
    "vize_vitrine_bg.wasm",
  ]) {
    assert.ok(wasmPackage.files?.includes(file), `@vizejs/wasm files include ${file}`);
  }
});

test("workspace TypeScript package builds use vp pack", () => {
  const packages = [
    ["npm/fresco", "vp pack", "vp pack --watch"],
    ["npm/musea-mcp-server", "vp pack", "vp pack --watch"],
    ["npm/musea-nuxt", "vp pack", "vp pack --watch"],
    ["npm/nuxt", "vp pack", "vp pack --watch"],
    ["npm/oxlint-plugin-vize", "vp pack", undefined],
    ["npm/rspack-vize-plugin", "vp pack", "vp pack --watch"],
    ["npm/unplugin-vize", "vp pack", "vp pack --watch"],
    ["npm/vite-plugin-vize", "vp pack", "vp pack --watch"],
  ] as const;

  for (const [packageDir, buildScript, devScript] of packages) {
    const packageJson = JSON.parse(
      fs.readFileSync(path.join(root, packageDir, "package.json"), "utf-8"),
    ) as {
      scripts?: Record<string, string>;
    };

    assert.equal(packageJson.scripts?.build, buildScript, `${packageDir} build script`);

    if (devScript != null) {
      assert.equal(packageJson.scripts?.dev, devScript, `${packageDir} dev script`);
    }
  }

  const oxlintPackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/oxlint-plugin-vize/package.json"), "utf-8"),
  ) as {
    scripts?: Record<string, string>;
  };
  assert.equal(oxlintPackage.scripts?.test, "vp pack && node src/test.ts");

  const rootTasks = fs.readFileSync(path.join(root, "tools/vite-plus/tasks/build.ts"), "utf-8");
  assert.match(rootTasks, /pnpm exec vp pack/);
});

test("fresco-native publishes bundled binaries directly from the root package", () => {
  const frescoNativePackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/fresco-native/package.json"), "utf-8"),
  ) as {
    files?: string[];
    scripts?: Record<string, string>;
  };
  const vizeNativePackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/vize-native/package.json"), "utf-8"),
  ) as {
    scripts?: Record<string, string>;
  };

  assert.deepEqual(frescoNativePackage.files, ["index.js", "index.d.ts", "*.node"]);
  assert.equal(frescoNativePackage.scripts?.prepublishOnly, undefined);
  assert.equal(
    frescoNativePackage.scripts?.["build:ci"],
    "napi build --platform --profile ci --manifest-path ../../crates/vize_fresco/Cargo.toml -p vize_fresco --features napi --output-dir .",
  );
  assert.equal(
    vizeNativePackage.scripts?.["build:ci"],
    "napi build --platform --profile ci --manifest-path ../../crates/vize_vitrine/Cargo.toml -p vize_vitrine --features napi --output-dir .",
  );
});
