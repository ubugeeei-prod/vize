import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const npmDir = path.join(root, "npm");
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

/** First-party native binaries are age-exempt by one name pattern, never per version. */
function assertNativeAgeExclusion(workspaceYaml: string): void {
  const exclude = (parse(workspaceYaml) as { minimumReleaseAgeExclude?: unknown })
    .minimumReleaseAgeExclude;
  assert.ok(Array.isArray(exclude), "minimumReleaseAgeExclude must be an array");
  const native = (exclude as string[]).filter((entry) => entry.startsWith("@vizejs/native"));
  assert.deepEqual(native, ["@vizejs/native-*"]);
}

function readRepoFile(filePath: string): string {
  return fs.readFileSync(path.join(root, filePath), "utf-8");
}

test("native package catalog pins and generated loader version checks stay aligned", () => {
  const nativePackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/native/package.json"), "utf-8"),
  ) as {
    optionalDependencies?: Record<string, string>;
    version?: string;
  };
  assert.ok(nativePackage.version);

  const workspaceYaml = fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf-8");
  const workspacePins = collectNativeBinaryCatalogPins(workspaceYaml);
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
    assert.match(
      lockfile,
      new RegExp(`['"]?${escapedName}['"]?:\\n\\s+specifier: ${escapeRegExp(catalogVersion)}\\n`),
      `${name} lockfile catalog specifier should match @vizejs/native version`,
    );
  }
  assertNativeAgeExclusion(workspaceYaml);

  const nativeTargetsLoader = fs.readFileSync(
    path.join(root, "npm/native/native-targets.js"),
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

  const oxlintNativeLoader = fs.readFileSync(path.join(root, "npm/oxlint/src/native.ts"), "utf-8");
  assert.match(oxlintNativeLoader, /spawnSync\("ldd", \["--version"\]/);
  assert.doesNotMatch(oxlintNativeLoader, /execSync\(/);
});

test("pkl runtime stays optional for consumers of the vize package", () => {
  const packageJson = JSON.parse(readRepoFile("npm/cli/package.json")) as {
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

test("vize package leaves Vue type versions to the consuming project", () => {
  const packageJson = JSON.parse(readRepoFile("npm/cli/package.json")) as {
    dependencies?: Record<string, string>;
    devDependencies?: Record<string, string>;
    optionalDependencies?: Record<string, string>;
    peerDependencies?: Record<string, string>;
    peerDependenciesMeta?: Record<string, { optional?: boolean }>;
  };
  const cliEntry = readRepoFile("npm/cli/src/cli.ts");

  assert.equal(packageJson.dependencies?.vue, undefined);
  assert.equal(packageJson.optionalDependencies?.vue, undefined);
  assert.equal(packageJson.devDependencies?.vue, "catalog:vue-stable");
  assert.equal(packageJson.peerDependencies?.vue, ">=2.6.0 || >=3.6.0-0");
  assert.deepEqual(packageJson.peerDependenciesMeta?.vue, {
    optional: true,
  });
  assert.doesNotMatch(cliEntry, /VIZE_VUE_PACKAGE\s*\?\?=/);
  assert.doesNotMatch(cliEntry, /require\.resolve\("vue\/package\.json"\)/);
});

test("musea Nuxt tests the same vue-router major that it declares as a peer", () => {
  const workspaceYaml = readRepoFile("pnpm-workspace.yaml");
  const catalogVersion = workspaceYaml.match(/^\s+vue-router: "([^"]+)"$/m)?.[1];
  assert.ok(catalogVersion, "vue-router catalog pin");

  const packageJson = JSON.parse(readRepoFile("npm/framework/musea-nuxt/package.json")) as {
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

    if (packageJson.bugs?.url !== "https://github.com/ubugeeei-prod/vize/issues") {
      failures.push(`${packageJson.name ?? packageDir}: missing canonical bugs.url`);
    }
    if (packageJson.homepage !== "https://github.com/ubugeeei-prod/vize") {
      failures.push(`${packageJson.name ?? packageDir}: missing canonical homepage`);
    }
  }

  assert.deepEqual(failures, []);
});

test("documented install commands point at supported release artifacts", () => {
  const vizeNpmPackage = JSON.parse(readRepoFile("npm/cli/package.json")) as {
    bin?: Record<string, string>;
    publishConfig?: Record<string, string>;
  };
  const vizeCrateToml = readRepoFile("crates/vize/Cargo.toml");
  const releaseWorkflow = readRepoFile(".github/workflows/release.yml");
  const nixPackage = readRepoFile("tools/nix/package.nix");
  const publicDocs = [
    "README.md",
    "docs/content/getting-started.md",
    "docs/content/guide/cli.md",
    "docs/content/guide/static-analysis.md",
    "npm/cli/README.md",
    "crates/vize/README.md",
  ];
  const unsupportedCargoInstallDocs = publicDocs.filter((filePath) =>
    readRepoFile(filePath).includes("cargo install vize"),
  );

  assert.equal(vizeNpmPackage.publishConfig?.access, "public");
  assert.equal(vizeNpmPackage.bin?.vize, "bin/vize");
  assert.match(releaseWorkflow, /cli-artifacts\/\*\.tar\.gz/);
  assert.match(nixPackage, /binaries = \{\n\s*"vize" = "vize";\n\s*\};/);
  assert.match(
    readRepoFile("tools/nix/packages.nix"),
    /apps =\s+lib\.genAttrs[\s\S]*?getExe' vize name/,
  );

  if (/^publish = false$/m.test(vizeCrateToml)) {
    assert.deepEqual(unsupportedCargoInstallDocs, []);
  }
});

test("release contracts run the manifest tests", () => {
  const contracts = parse(readRepoFile(".github/workflows/release-contracts.yml"));
  const steps = contracts.jobs.contracts.steps as Array<{ run?: string }>;
  for (const filename of [
    "tests/tooling/package-manifests.test.ts",
    "tests/tooling/bundler-plugin-manifests.test.ts",
  ]) {
    assert.ok(contracts.on.pull_request.paths.includes(filename));
    assert.ok(
      steps.some((step) => {
        const [command, flag, ...files] = step.run?.trim().split(/\s+/) ?? [];
        return command === "node" && flag === "--test" && files.includes(filename);
      }),
    );
  }
});

test("editor extension manifests keep expected defaults and version alignment", () => {
  const workspaceVersion = fs
    .readFileSync(path.join(root, "Cargo.toml"), "utf-8")
    .match(/^version = "(.+)"$/m)?.[1];
  assert.ok(workspaceVersion);

  const vscodePackage = JSON.parse(
    fs.readFileSync(path.join(root, "editors/vscode/package.json"), "utf-8"),
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
    true,
  );
  assert.equal(
    vscodePackage.contributes?.configuration?.properties?.["vize.lint.enable"]?.default,
    true,
  );
  assert.equal(
    vscodePackage.contributes?.configuration?.properties?.["vize.typecheck.enable"]?.default,
    true,
  );
  assert.equal(
    vscodePackage.contributes?.configuration?.properties?.["vize.editor.enable"]?.default,
    true,
  );
  assert.equal(
    vscodePackage.contributes?.configuration?.properties?.["vize.formatting.enable"]?.default,
    false,
  );
  assert.equal(vscodePackage.scripts?.check, "tsc --noEmit && vp check src vite.config.ts");
  assert.equal(
    vscodePackage.scripts?.["check:fix"],
    "vp check --fix src vite.config.ts && tsc --noEmit",
  );

  const zedManifest = fs.readFileSync(path.join(root, "editors/zed/extension.toml"), "utf-8");
  const zedVersion = zedManifest.match(/^version = "(.+)"$/m)?.[1];

  assert.equal(zedVersion, workspaceVersion);
  assert.match(zedManifest, /^\[language_servers\.vize\]$/m);
  assert.match(zedManifest, /^languages = \["Vue.js", "Art Vue"\]$/m);
});

test("workspace package builds do not nest pnpm run commands", () => {
  const museaPackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/builder/vite-musea/package.json"), "utf-8"),
  ) as {
    scripts?: Record<string, string>;
  };

  assert.equal(museaPackage.scripts?.build, "vp pack && vp build --config gallery-vite.config.ts");
  assert.equal(museaPackage.scripts?.dev, "vp pack --watch");
  assert.doesNotMatch(museaPackage.scripts?.build ?? "", /\bpnpm run\b/);
});

test("vize package delegates rule type generation to the workspace task", () => {
  const vizePackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/package.json"), "utf-8"),
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
    fs.readFileSync(path.join(root, "npm/wasm/package.json"), "utf-8"),
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
    "lint-format.d.ts",
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
    ["npm/mcp-musea", "vp pack", "vp pack --watch"],
    ["npm/framework/musea-nuxt", "vp pack", "vp pack --watch"],
    ["npm/framework/nuxt", "vp pack", "vp pack --watch"],
    ["npm/oxlint", "vp pack", undefined],
    ["npm/builder/rspack", "vp pack", "vp pack --watch"],
    ["npm/builder/unplugin", "vp pack", "vp pack --watch"],
    ["npm/builder/vite", "vp pack", "vp pack --watch"],
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

  const oxlintPackage = JSON.parse(readRepoFile("npm/oxlint/package.json")) as {
    engines?: Record<string, string>;
    scripts?: Record<string, string>;
  };
  assert.equal(oxlintPackage.engines?.node, "^22 || >= 24");
  assert.equal(
    oxlintPackage.scripts?.test,
    "vp pack && vp test run src/file-state.test.ts src/batched-rules.test.ts src/script-safe-source.test.ts src/active-rule-collection.test.ts && node src/test.ts && node src/script-location.test.ts && node src/casing-options.test.ts && node src/router-globals.test.ts && node src/template-style-fixes.test.mjs && node src/html-binding.test.mjs && node --test src/eslint-source.test.ts && node src/eslint-acceptance.test.mjs",
  );
  const rootTasks = readRepoFile("tools/config/vite-plus/tasks/build.ts");
  assert.match(rootTasks, /vscodeExtensionPackageBin\("vite-plus", "vp"\)[\s\S]*pack/);
});
