import { execFileSync, spawnSync } from "node:child_process";
import type { Plugin } from "vite";
import { defineConfig } from "vite-plus";

const checkedPackages = [
  "./npm/vize",
  "./npm/vite-plugin-vize",
  "./npm/oxlint-plugin-vize",
  "./npm/vite-plugin-musea",
  "./npm/unplugin-vize",
  "./npm/rspack-vize-plugin",
  "./npm/nuxt",
  "./npm/musea-nuxt",
  "./npm/musea-mcp-server",
  "./npm/fresco",
  "./npm/vite-plugin-vize/example",
  "./npm/rspack-vize-plugin/example",
  "./examples/vite-musea",
  "./examples/oxlint-vize",
  "./playground",
];
const directCheckPackages = ["./examples/vite-musea", "./playground"];
const checkedPackagesViaVpRun = checkedPackages.filter((pkg) => !directCheckPackages.includes(pkg));
const ciCheckedPackages = checkedPackagesViaVpRun.filter((pkg) => pkg !== "./examples/oxlint-vize");

const packedPackages = [
  "./npm/vize",
  "./npm/vite-plugin-vize",
  "./npm/oxlint-plugin-vize",
  "./npm/vite-plugin-musea",
  "./npm/unplugin-vize",
  "./npm/rspack-vize-plugin",
  "./npm/nuxt",
  "./npm/musea-nuxt",
  "./npm/musea-mcp-server",
  "./npm/fresco",
];

const testedPackages = [
  "./npm/vite-plugin-vize",
  "./npm/oxlint-plugin-vize",
  "./npm/unplugin-vize",
  "./npm/rspack-vize-plugin",
];

const floatingPromiseTestPatterns = ["tests/**/*.ts"];

const cacheInputs = {
  workspace: [
    ".node-version",
    "package.json",
    "vite.config.ts",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
  ],
  jsChecks: [
    ".node-version",
    "package.json",
    "vite.config.ts",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "npm/**/package.json",
    "npm/**/vite.config.ts",
    "npm/**/rspack.config.ts",
    "npm/**/src/**",
    "examples/**/package.json",
    "examples/**/vite.config.ts",
    "examples/**/playwright.config.ts",
    "examples/**/src/**",
    "playground/package.json",
    "playground/vite*.ts",
    "playground/playwright.config.ts",
    "playground/src/**",
    "playground/e2e/**",
  ],
  rust: [
    ".node-version",
    "package.json",
    "vite.config.ts",
    "Cargo.toml",
    "Cargo.lock",
    "crates/**",
    "tests/**",
    "tools/**",
  ],
  e2e: [
    ".node-version",
    "package.json",
    "vite.config.ts",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "tests/package.json",
    "tests/app/**",
    "tests/_helpers/**",
    "npm/vize*/**",
    "npm/vite-plugin-vize/**",
    "npm/nuxt/**",
    "npm/vite-plugin-musea/**",
    "npm/musea-nuxt/**",
  ],
};

const localVp = "./node_modules/.bin/vp";
const shellQuote = (command: string) => `'${command.replaceAll("'", `'"'"'`)}'`;
const darwinLibiconvLibraryPath = process.env.VIZE_DARWIN_LIBICONV_LIB;
const rustTaskEnvironment =
  darwinLibiconvLibraryPath == null
    ? []
    : [
        `export LIBRARY_PATH=${shellQuote(darwinLibiconvLibraryPath)}\${LIBRARY_PATH:+:$LIBRARY_PATH}`,
        `export RUSTFLAGS=${shellQuote(`-L native=${darwinLibiconvLibraryPath}`)}\${RUSTFLAGS:+ $RUSTFLAGS}`,
      ];
const withRustTaskEnvironment = (command: string) =>
  rustTaskEnvironment.length === 0
    ? command
    : `sh -c ${shellQuote(`${rustTaskEnvironment.join("; ")}; ${command}`)}`;

const task = (
  command: string,
  options: {
    input?: string[];
  } = {},
) => ({
  command: withRustTaskEnvironment(command),
  ...options,
});

const noCacheTask = (command: string) => ({
  cache: false as const,
  command: withRustTaskEnvironment(command),
});

const runInDirectory = (cwd: string, command: string) =>
  `sh -c ${shellQuote(`cd ${cwd} && ${command}`)}`;
const runPackageScriptDirectly = (taskName: string, packages: string[]) =>
  packages.map((pkg) => runInDirectory(pkg, `pnpm run ${taskName}`)).join(" && ");
const installVscodeExtensionDependencies = runInDirectory(
  "npm/vscode-vize",
  "if [ -x node_modules/.bin/vp ]; then exit 0; fi && mkdir -p node_modules/.bin && pnpm install --ignore-workspace --no-lockfile --prefer-offline",
);
const runInVscodeExtension = (...commands: string[]) =>
  `${installVscodeExtensionDependencies} && ${runInDirectory("npm/vscode-vize", commands.join(" && "))}`;

const commandExists = (command: string) =>
  spawnSync("sh", ["-c", `command -v ${command}`], { stdio: "ignore" }).status === 0;

const rootBuildTaskPlugin = (): Plugin => ({
  name: "vize-root-build-task",
  apply: "build",
  closeBundle() {
    if (process.env.VIZE_SKIP_ROOT_BUILD_TASK === "1") {
      return;
    }

    const buildCommand = ["vp", "run", "--workspace-root", "build"];
    const command = commandExists("wasm-pack") || !commandExists("nix") ? "vp" : "nix";
    const args =
      command === "vp"
        ? buildCommand.slice(1)
        : ["--option", "warn-dirty", "false", "develop", "--command", ...buildCommand];

    execFileSync(command, args, {
      env: {
        ...process.env,
        VIZE_SKIP_ROOT_BUILD_TASK: "1",
      },
      stdio: "inherit",
    });
  },
});

const runInPackages = (
  taskName: string,
  packages: string[],
  options: {
    concurrencyLimit?: number;
  } = {},
) =>
  [
    ...(options.concurrencyLimit == null
      ? []
      : [`VP_RUN_CONCURRENCY_LIMIT=${options.concurrencyLimit}`]),
    "vp",
    "run",
    ...packages.map((pkg) => `--filter '${pkg}'`),
    taskName,
  ].join(" ");

const runTask = (taskName: string) => `vp run --workspace-root ${taskName}`;
const runTasks = (...taskNames: string[]) => taskNames.map(runTask).join(" && ");
const moonCommand = process.env.MOON_BIN ?? "env -u MOON_HOME moon";
const moonScript = (name: string, ...args: string[]) =>
  [
    moonCommand,
    "run",
    "-q",
    "--target",
    "native",
    "-",
    "--",
    ...args,
    "<",
    `tools/moon/scripts/${name}.mbtx`,
  ].join(" ");

const devApp = (target?: string) =>
  target == null ? moonScript("dev_app") : moonScript("dev_app", target);

const setupTasks = {
  setup: noCacheTask("vp install"),
};

const devTasks = {
  dev: noCacheTask(runTask("dev:app")),
  "dev:app": noCacheTask(devApp()),
  "dev:playground": noCacheTask(devApp("playground")),
  "dev:misskey": noCacheTask(devApp("misskey")),
  "dev:npmx": noCacheTask(devApp("npmx")),
  "dev:elk": noCacheTask(devApp("elk")),
  "dev:vuefes": noCacheTask(devApp("vuefes")),
  example: noCacheTask(runInPackages("dev", ["./npm/vite-plugin-vize/example"])),
};

const buildTasks = {
  build: noCacheTask(runTasks("build:rust", "build:all")),
  "build:all": noCacheTask(runTasks("build:runtime", "package:editor-extensions")),
  "build:rust": task("cargo build --workspace", { input: cacheInputs.rust }),
  "build:runtime": noCacheTask(runTasks("build:native", "build:wasm", "build:packages")),
  "build:packages": noCacheTask(runInPackages("build", packedPackages)),
  "build:native": noCacheTask(runPackageScriptDirectly("build", ["./npm/vize-native"])),
  "build:wasm": task(moonScript("build_vitrine_wasm", "nodejs", "npm/vite-plugin-vize/wasm")),
  "build:wasm-web": task(moonScript("build_vitrine_wasm", "web", "playground/src/wasm")),
  "build:vite-plugin": noCacheTask(
    `${runInPackages("build", ["./npm/vize"])} && ${runInPackages("build", ["./npm/vite-plugin-vize"])}`,
  ),
  "build:plugin": noCacheTask(runTask("build:vite-plugin")),
  "build:cli": task("cargo build --release -p vize"),
  "build:vscode-extension": noCacheTask(runInVscodeExtension("pnpm exec vp pack")),
  "build:editor-extensions": noCacheTask(runTasks("build:vscode-extension", "check:zed-extension")),
  "package:vscode-extension": noCacheTask(
    runInVscodeExtension("pnpm exec vsce package --no-dependencies --out dist/vize.vsix"),
  ),
  "check:zed-extension": task("cargo check --manifest-path npm/zed-vize/Cargo.toml", {
    input: ["npm/zed-vize/**"],
  }),
  "package:zed-extension": noCacheTask(
    "tar --exclude 'zed-vize/target' -czf zed-vize-extension.tar.gz -C npm zed-vize",
  ),
  "package:editor-extensions": noCacheTask(
    `${runInVscodeExtension(
      "pnpm exec tsgo --noEmit",
      "pnpm exec vp check src vite.config.ts",
      "pnpm exec vsce package --no-dependencies --out dist/vize.vsix",
    )} && ${runTask("check:zed-extension")} && ${runTask("package:zed-extension")}`,
  ),
  "install:plugin": noCacheTask("vp install --filter './npm/vite-plugin-vize'"),
};

const cliTasks = {
  cli: noCacheTask(
    'sh -c \'if [ "${usage_debug:-$1}" = "true" ] || [ "$1" = "--debug" ]; then cargo install --path crates/vize --force --locked --debug && echo "Installed vize CLI (debug build)"; else cargo install --path crates/vize --force --locked && echo "Installed vize CLI (release build)"; fi\' --',
  ),
  "cli:help": noCacheTask("vize --help"),
  "cli:example": noCacheTask("vize './**/*.vue' -o . -v"),
  "cli:example-json": noCacheTask("vize './**/*.vue' -o . -f json -v"),
  "cli:example-ssr": noCacheTask("vize './**/*.vue' -o . -f json --ssr -v"),
  "cli:example-stats": noCacheTask("vize './**/*.vue' -f stats -v"),
};

const testTasks = {
  test: noCacheTask(runTasks("test:rust", "test:js", "test:scripts")),
  "test:rust": task("cargo test --workspace", { input: cacheInputs.rust }),
  "test:js": noCacheTask(`${runTask("build:native")} && ${runInPackages("test", testedPackages)}`),
  "test:scripts": task("node --test --test-concurrency=1 tests/tooling/*.test.ts", {
    input: cacheInputs.rust,
  }),
  "test:playground": task(runInPackages("test:browser", ["./playground"]), {
    input: cacheInputs.jsChecks,
  }),
  "test:e2e": noCacheTask(runTasks("test:e2e:dev", "test:e2e:preview")),
  "test:e2e:dev": task(runInPackages("test:dev", ["./tests"]), { input: cacheInputs.e2e }),
  "test:e2e:preview": task(runInPackages("test:preview", ["./tests"]), {
    input: cacheInputs.e2e,
  }),
  "test:e2e:vrt": task(runInPackages("test:vrt", ["./tests"]), { input: cacheInputs.e2e }),
  "test:vue": task("cargo test -p vize_test_runner", { input: cacheInputs.rust }),
  coverage: task("cargo run -p vize_test_runner --bin coverage", { input: cacheInputs.rust }),
  "coverage:verbose": task("cargo run -p vize_test_runner --bin coverage -- -v", {
    input: cacheInputs.rust,
  }),
  "coverage:diff": task("cargo run -p vize_test_runner --bin coverage -- -vv", {
    input: cacheInputs.rust,
  }),
  "generate:rule-types": task(moonScript("generate_rule_types"), {
    input: cacheInputs.rust,
  }),
  "expected:generate": task(moonScript("generate_expected")),
  "expected:generate:sfc": task(moonScript("generate_expected", "--mode", "sfc")),
  "expected:generate:vdom": task(moonScript("generate_expected", "--mode", "vdom")),
  "expected:generate:vapor": task(moonScript("generate_expected", "--mode", "vapor")),
  snapshot: noCacheTask(runTasks("snapshot:test", "snapshot:review")),
  "snapshot:test": task("cargo insta test -p vize_atelier_sfc -- snapshot_tests"),
  "snapshot:review": noCacheTask("cargo insta review"),
  "snapshot:accept": noCacheTask("cargo insta accept"),
};

const benchmarkTasks = {
  bench: noCacheTask(moonScript("bench", "run")),
  "bench:quick": noCacheTask(moonScript("bench", "run", "1000")),
  "bench:generate": noCacheTask(moonScript("bench", "generate", "15000")),
  "bench:lint": noCacheTask(moonScript("bench", "lint")),
  "bench:fmt": noCacheTask(moonScript("bench", "fmt")),
  "bench:check": noCacheTask(moonScript("bench", "check")),
  "bench:vite": noCacheTask(moonScript("bench", "vite")),
  "bench:all": noCacheTask(
    runTasks("bench", "bench:lint", "bench:fmt", "bench:check", "bench:vite"),
  ),
  "bench:rust": noCacheTask("cargo bench -p vize_atelier_sfc"),
};

const ciPackageCheckCommand = runInPackages("check", ciCheckedPackages, {
  concurrencyLimit: 1,
});
const directPackageCheckCommand = runPackageScriptDirectly("check", directCheckPackages);

const checkTasks = {
  check: noCacheTask(runTasks("check:repo", "check:rust", "check:js", "check:editor-extensions")),
  "check:js": noCacheTask(runTasks("check:js:packages", "check:js:direct-packages")),
  "check:js:packages": task(
    runInPackages("check", checkedPackagesViaVpRun, { concurrencyLimit: 1 }),
    {
      input: cacheInputs.jsChecks,
    },
  ),
  "check:js:direct-packages": noCacheTask(directPackageCheckCommand),
  "check:repo": noCacheTask(`${localVp} check`),
  // The oxlint example intentionally exits non-zero for its default lint script,
  // so CI checks every package except that runnable failure-case fixture.
  "check:ci": noCacheTask(
    `${runTask("check:repo")} && ${ciPackageCheckCommand} && ${directPackageCheckCommand}`,
  ),
  "check:fix": noCacheTask(runInPackages("check:fix", checkedPackages)),
  "check:rust": noCacheTask("cargo check --workspace"),
  "check:vscode-extension": noCacheTask(
    runInVscodeExtension("pnpm exec tsgo --noEmit", "pnpm exec vp check src vite.config.ts"),
  ),
  "check:editor-extensions": noCacheTask(runTasks("check:vscode-extension", "check:zed-extension")),
  clippy: task("cargo clippy --workspace -- -D warnings", { input: cacheInputs.rust }),
  fmt: noCacheTask(runTasks("fmt:repo", "fmt:rust", "fmt:js")),
  "fmt:repo": noCacheTask(`${localVp} fmt --write`),
  "fmt:js": noCacheTask(runInPackages("fmt", checkedPackages)),
  "fmt:rust": task("cargo fmt --all", { input: cacheInputs.rust }),
  "fmt:all": noCacheTask(runTask("fmt")),
  lint: noCacheTask(runTask("check")),
  "lint:fix": noCacheTask(runTask("check:fix")),
  "lint:rust": task("cargo clippy --workspace -- -D warnings", { input: cacheInputs.rust }),
  "lint:all": noCacheTask(runTasks("lint:rust", "check")),
  "fmt:check": noCacheTask(runTask("check")),
  ci: noCacheTask(runTasks("fmt:all", "clippy", "test", "check:ci")),
};

const releaseTasks = {
  release: noCacheTask(moonScript("release")),
  "publish:wasm": noCacheTask(
    `${moonScript("build_vize_wasm_package")} && ${moonScript("publish_npm_package", "npm/vize-wasm")}`,
  ),
  "publish:native": noCacheTask(
    `${runTask("build:native")} && ${moonScript("publish_npm_package", "npm/vize-native")}`,
  ),
  "publish:vite-plugin": noCacheTask(
    `${runTask("build:vite-plugin")} && ${moonScript("publish_npm_package", "npm/vite-plugin-vize")}`,
  ),
  "publish:oxlint-plugin": noCacheTask(
    `${runInPackages("build", ["./npm/oxlint-plugin-vize"])} && ${moonScript("inject_native_optional_deps", "npm/oxlint-plugin-vize/package.json", "npm/vize-native/package.json")} && ${moonScript("publish_npm_package", "npm/oxlint-plugin-vize")}`,
  ),
  "publish:npm": noCacheTask(
    runTasks("publish:wasm", "publish:native", "publish:vite-plugin", "publish:oxlint-plugin"),
  ),
  "publish:crates": noCacheTask(moonScript("publish_crates")),
  "publish:vscode-extension": noCacheTask(
    `${installVscodeExtensionDependencies} && ${moonScript("publish_vscode_extension", "npm/vscode-vize/dist/vize.vsix")}`,
  ),
  publish: noCacheTask(runTasks("publish:npm", "publish:crates")),
};

export default defineConfig({
  plugins: [rootBuildTaskPlugin()],
  build: {
    emptyOutDir: true,
    lib: {
      entry: "tests/tooling/support/vp-build-entry.ts",
      fileName: "vp-build",
      formats: ["es"],
    },
    outDir: "target/vp-build",
  },
  fmt: {
    ignorePatterns: [
      "**/__snapshots__/**",
      "**/__snapshot__/**",
      "**/__agent_only/**",
      "**/__ubugeeei__/**",
      "tests/_fixtures/**",
    ],
  },
  lint: {
    ignorePatterns: [
      "**/__snapshots__/**",
      "**/__snapshot__/**",
      "**/__agent_only/**",
      "**/__ubugeeei__/**",
      "npm/vscode-vize/**",
      "tests/_fixtures/**",
    ],
    options: {
      typeAware: true,
    },
    overrides: [
      {
        files: floatingPromiseTestPatterns,
        rules: {
          "typescript/no-floating-promises": "off",
        },
      },
    ],
  },
  run: {
    cache: {
      scripts: true,
      tasks: true,
    },
    tasks: {
      ...setupTasks,
      ...devTasks,
      ...buildTasks,
      ...cliTasks,
      ...testTasks,
      ...benchmarkTasks,
      ...checkTasks,
      ...releaseTasks,
    },
  },
});
