import { defineConfig } from "vite-plus";

const checkedPackages = [
  "./npm/vize",
  "./npm/vite-plugin-vize",
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
  "./playground",
];

const packedPackages = [
  "./npm/vize",
  "./npm/vite-plugin-vize",
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
  "./npm/unplugin-vize",
  "./npm/rspack-vize-plugin",
];

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
    "scripts/**",
  ],
};

const task = (
  command: string,
  options: {
    input?: string[];
  } = {},
) => ({
  command,
  ...options,
});

const noCacheTask = (command: string) => ({
  cache: false as const,
  command,
});

const runInPackages = (taskName: string, packages: string[]) =>
  ["vp", "run", ...packages.map((pkg) => `--filter '${pkg}'`), taskName].join(" ");

const runTask = (taskName: string) => `vp run --workspace-root ${taskName}`;
const runTasks = (...taskNames: string[]) => taskNames.map(runTask).join(" && ");

const devApp = (target?: string) =>
  target == null
    ? "node --experimental-strip-types scripts/dev-app.ts"
    : `usage_target=${target} node --experimental-strip-types scripts/dev-app.ts`;

const publishWithVersionTag = (cwd: string, publishCommand: string) =>
  `sh -c 'cd ${cwd} && VERSION=$(node -p "require(\\\"./package.json\\\").version") && case "$VERSION" in *-alpha*) ${publishCommand} --tag alpha ;; *-beta*) ${publishCommand} --tag beta ;; *-rc*) ${publishCommand} --tag rc ;; *) ${publishCommand} ;; esac'`;

const setupTasks = {
  setup: noCacheTask("vp install"),
};

const devTasks = {
  dev: noCacheTask(runInPackages("dev", ["./playground"])),
  "dev:app": noCacheTask(devApp()),
  "dev:playground": noCacheTask(devApp("playground")),
  "dev:misskey": noCacheTask(devApp("misskey")),
  "dev:npmx": noCacheTask(devApp("npmx")),
  "dev:elk": noCacheTask(devApp("elk")),
  "dev:vuefes": noCacheTask(devApp("vuefes")),
  example: noCacheTask(runInPackages("dev", ["./npm/vite-plugin-vize/example"])),
};

const buildTasks = {
  build: noCacheTask(runTasks("build:native", "build:wasm", "build:packages")),
  "build:packages": noCacheTask(runInPackages("build", packedPackages)),
  "build:native": noCacheTask(runInPackages("build", ["./npm/vize-native"])),
  "build:wasm": task(
    "wasm-pack build crates/vize_vitrine --target nodejs --out-dir ../../npm/vite-plugin-vize/wasm --features wasm --no-default-features",
  ),
  "build:wasm-web": task(
    "wasm-pack build crates/vize_vitrine --target web --out-dir ../../playground/src/wasm --features wasm --no-default-features",
  ),
  "build:vite-plugin": noCacheTask(runInPackages("build", ["./npm/vite-plugin-vize"])),
  "build:plugin": noCacheTask(runTask("build:vite-plugin")),
  "build:cli": task("cargo build --release -p vize"),
  "install:plugin": noCacheTask("pnpm -C npm/vite-plugin-vize install"),
};

const cliTasks = {
  cli: noCacheTask(
    'sh -c \'if [ "${usage_debug:-$1}" = "true" ] || [ "$1" = "--debug" ]; then cargo install --path crates/vize --force --debug && echo "Installed vize CLI (debug build)"; else cargo install --path crates/vize --force && echo "Installed vize CLI (release build)"; fi\' --',
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
  "test:scripts": task("node --experimental-strip-types --test scripts/*.test.ts", {
    input: cacheInputs.rust,
  }),
  "test:playground": task(runInPackages("test:browser", ["./playground"]), {
    input: cacheInputs.jsChecks,
  }),
  "test:vue": task("cargo test -p vize_test_runner", { input: cacheInputs.rust }),
  coverage: task("cargo run -p vize_test_runner --bin coverage", { input: cacheInputs.rust }),
  "coverage:verbose": task("cargo run -p vize_test_runner --bin coverage -- -v", {
    input: cacheInputs.rust,
  }),
  "coverage:diff": task("cargo run -p vize_test_runner --bin coverage -- -vv", {
    input: cacheInputs.rust,
  }),
  "expected:generate": task("node --experimental-strip-types scripts/generate-expected.ts"),
  "expected:generate:sfc": task(
    "node --experimental-strip-types scripts/generate-expected.ts --mode sfc",
  ),
  "expected:generate:vdom": task(
    "node --experimental-strip-types scripts/generate-expected.ts --mode vdom",
  ),
  "expected:generate:vapor": task(
    "node --experimental-strip-types scripts/generate-expected.ts --mode vapor",
  ),
  snapshot: noCacheTask(runTasks("snapshot:test", "snapshot:review")),
  "snapshot:test": task("cargo insta test -p vize_atelier_sfc -- snapshot_tests"),
  "snapshot:review": noCacheTask("cargo insta review"),
  "snapshot:accept": noCacheTask("cargo insta accept"),
};

const benchmarkTasks = {
  bench: noCacheTask("node --experimental-strip-types bench/run.ts"),
  "bench:quick": noCacheTask("node --experimental-strip-types bench/run.ts 1000"),
  "bench:generate": noCacheTask("node bench/generate.mjs 15000"),
  "bench:lint": noCacheTask("node --experimental-strip-types bench/lint.ts"),
  "bench:fmt": noCacheTask("node --experimental-strip-types bench/fmt.ts"),
  "bench:check": noCacheTask("node --experimental-strip-types bench/check.ts"),
  "bench:vite": noCacheTask("node --experimental-strip-types bench/vite.ts"),
  "bench:all": noCacheTask(
    runTasks("bench", "bench:lint", "bench:fmt", "bench:check", "bench:vite"),
  ),
  "bench:rust": noCacheTask("cargo bench -p vize_atelier_sfc"),
};

const checkTasks = {
  check: task(runInPackages("check", checkedPackages), { input: cacheInputs.jsChecks }),
  "check:fix": noCacheTask(runInPackages("check:fix", checkedPackages)),
  "check:rust": task("cargo check --workspace", { input: cacheInputs.rust }),
  clippy: task("cargo clippy --workspace -- -D warnings", { input: cacheInputs.rust }),
  fmt: noCacheTask(runInPackages("fmt", checkedPackages)),
  "fmt:rust": task("cargo fmt --all", { input: cacheInputs.rust }),
  "fmt:all": noCacheTask(runTasks("fmt:rust", "fmt")),
  lint: noCacheTask(runTask("check")),
  "lint:fix": noCacheTask(runTask("check:fix")),
  "lint:rust": task("cargo clippy --workspace -- -D warnings", { input: cacheInputs.rust }),
  "lint:all": noCacheTask(runTasks("lint:rust", "check")),
  "fmt:check": noCacheTask(runTask("check")),
  ci: noCacheTask(runTasks("fmt:all", "clippy", "test")),
};

const releaseTasks = {
  release: noCacheTask(
    "sh -c 'if [ -n \"${usage_type:-}\" ] && { [ $# -eq 0 ] || [ \"$1\" != \"$usage_type\" ]; }; then set -- \"$usage_type\" \"$@\"; fi; ./scripts/release.sh \"$@\"' --",
  ),
  "publish:wasm": noCacheTask(
    'sh -c \'cd npm/vize-wasm && cargo build --release -p vize_vitrine --no-default-features --features wasm --target wasm32-unknown-unknown && wasm-bindgen ../../target/wasm32-unknown-unknown/release/vize_vitrine.wasm --out-dir . --target web && VERSION=$(node -p "require(\\"./package.json\\").version") && case "$VERSION" in *-alpha*) npm publish --access public --tag alpha ;; *-beta*) npm publish --access public --tag beta ;; *-rc*) npm publish --access public --tag rc ;; *) npm publish --access public ;; esac\'',
  ),
  "publish:native": noCacheTask(
    `${runTask("build:native")} && ${publishWithVersionTag("npm/vize-native", "npm publish --access public")}`,
  ),
  "publish:vite-plugin": noCacheTask(
    `${runTask("build:vite-plugin")} && ${publishWithVersionTag("npm/vite-plugin-vize", "pnpm publish --access public --no-git-checks")}`,
  ),
  "publish:npm": noCacheTask(runTasks("publish:wasm", "publish:native", "publish:vite-plugin")),
  "publish:crates": noCacheTask("bash ./scripts/publish-crates.sh"),
  publish: noCacheTask(runTasks("publish:npm", "publish:crates")),
};

export default defineConfig({
  lint: {
    options: {
      typeAware: true,
    },
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
