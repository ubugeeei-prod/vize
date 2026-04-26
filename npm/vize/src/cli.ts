import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import * as path from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { loadConfig } from "./config.js";

const require = createRequire(import.meta.url);
const WORKSPACE_BINDING_PATH = "../../vize-native";

// ============================================================================
// Native binding loader (oxlint pattern)
// ============================================================================

function isMusl(): boolean {
  const report = process.report?.getReport();
  if (typeof report === "object" && report !== null && "header" in report) {
    const header = (report as { header: { glibcVersionRuntime?: string } }).header;
    return !header.glibcVersionRuntime;
  }
  try {
    const lddPath = require("child_process").execSync("which ldd").toString().trim();
    return readFileSync(lddPath, "utf8").includes("musl");
  } catch {
    return true;
  }
}

function getBindingPackageName(): string {
  const { platform, arch } = process;

  switch (platform) {
    case "darwin":
      switch (arch) {
        case "x64":
          return "@vizejs/native-darwin-x64";
        case "arm64":
          return "@vizejs/native-darwin-arm64";
        default:
          throw new Error(`Unsupported architecture on macOS: ${arch}`);
      }
    case "win32":
      switch (arch) {
        case "x64":
          return "@vizejs/native-win32-x64-msvc";
        case "arm64":
          return "@vizejs/native-win32-arm64-msvc";
        default:
          throw new Error(`Unsupported architecture on Windows: ${arch}`);
      }
    case "linux":
      switch (arch) {
        case "x64":
          return isMusl() ? "@vizejs/native-linux-x64-musl" : "@vizejs/native-linux-x64-gnu";
        case "arm64":
          return isMusl() ? "@vizejs/native-linux-arm64-musl" : "@vizejs/native-linux-arm64-gnu";
        default:
          throw new Error(`Unsupported architecture on Linux: ${arch}`);
      }
    default:
      throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
  }
}

interface NativeBinding {
  compileSfcBatchWithResults: (
    files: BatchFileInput[],
    options?: NativeBuildOptions,
  ) => BatchCompileResult;
  formatSfc: (source: string, options?: NativeFormatOptions) => FormatResult;
  typeCheck: (source: string, options?: NativeTypeCheckOptions) => TypeCheckResult;
  generateDeclaration?: (source: string, options?: NativeDeclarationOptions) => DeclarationResult;
  lint: (
    patterns: string[],
    options?: {
      format?: string;
      max_warnings?: number;
      quiet?: boolean;
      fix?: boolean;
      help_level?: string;
      preset?: string;
    },
  ) => LintResult;
}

type NativeCommand = "build" | "check" | "fmt" | "lint";

const REQUIRED_BINDINGS: Record<NativeCommand, keyof NativeBinding> = {
  build: "compileSfcBatchWithResults",
  check: "typeCheck",
  fmt: "formatSfc",
  lint: "lint",
};

function loadNative(command: NativeCommand): NativeBinding {
  const attemptedPackages = getAttemptedPackages();
  let lastError: unknown = null;
  const requiredBinding = REQUIRED_BINDINGS[command];

  for (const packageName of attemptedPackages) {
    try {
      const binding = require(packageName) as Partial<NativeBinding>;
      if (typeof binding[requiredBinding] !== "function") {
        throw new Error(`${packageName} does not expose the ${command} binding.`);
      }
      return binding as NativeBinding;
    } catch (error) {
      lastError = error;
    }
  }

  console.error(`Failed to load native binding. Tried: ${attemptedPackages.join(", ")}`);
  console.error("Try reinstalling: npm install vize");
  throw lastError instanceof Error ? lastError : new Error("Failed to load native binding");
}

function getAttemptedPackages(): readonly string[] {
  const platformBindingPackage = getBindingPackageName();
  return shouldPreferWorkspaceBinding(resolveWorkspaceBindingPath())
    ? [WORKSPACE_BINDING_PATH, platformBindingPackage]
    : [platformBindingPackage, WORKSPACE_BINDING_PATH];
}

function resolveWorkspaceBindingPath(): string | null {
  try {
    return require.resolve(WORKSPACE_BINDING_PATH);
  } catch {
    return null;
  }
}

function shouldPreferWorkspaceBinding(resolvedPath: string | null): boolean {
  const override = process.env.VIZE_PREFER_WORKSPACE_BINDING;
  if (override === "1" || override === "true") {
    return true;
  }
  if (override === "0" || override === "false") {
    return false;
  }
  if (resolvedPath == null) {
    return false;
  }

  return resolvedPath.includes(`${path.sep}npm${path.sep}vize-native${path.sep}`);
}

// ============================================================================
// Lint command
// ============================================================================

interface LintOptions {
  format?: string;
  maxWarnings?: number;
  quiet?: boolean;
  fix?: boolean;
  helpLevel?: string;
  preset?: string;
}

interface LintResult {
  output: string;
  errorCount: number;
  warningCount: number;
  fileCount: number;
  timeMs: number;
}

interface SharedConfigOptions {
  configFile?: string;
  configMode: "root" | "none";
}

interface ParsedLintCommand {
  patterns: string[];
  options: LintOptions;
  sharedConfig: SharedConfigOptions;
}

function printUsage(): void {
  console.error("Usage: vize <command> [options]");
  console.error("Commands: build, fmt, check, lint, upgrade, ready, musea");
}

function printBuildUsage(): void {
  console.error("Usage: vize build [options] [files-or-directories]");
  console.error("Options:");
  console.error("  -o, --output <dir>             Output directory");
  console.error("  -f, --format <js|json|stats>   Output format");
  console.error("      --ssr                      Enable SSR compilation");
  console.error("      --script-ext <mode>        preserve or downcompile");
  console.error("  -j, --threads <number>         Worker thread count");
}

function printFmtUsage(): void {
  console.error("Usage: vize fmt [options] [files-or-directories]");
  console.error("Options:");
  console.error("      --check                    Exit with an error if files need formatting");
  console.error("  -w, --write                    Write formatted output");
  console.error("      --single-quote             Use single quotes");
  console.error("      --print-width <number>     Maximum line width");
  console.error("      --tab-width <number>       Indentation width");
  console.error("      --use-tabs                 Indent with tabs");
  console.error("      --no-semi                  Omit semicolons");
}

function printCheckUsage(): void {
  console.error("Usage: vize check [options] [files-or-directories]");
  console.error("Options:");
  console.error("  -f, --format <text|json>       Output format");
  console.error("  -q, --quiet                    Show summary only");
  console.error("      --strict                   Enable strict checks");
  console.error("      --show-virtual-ts          Print generated Virtual TS");
  console.error("      --declaration              Emit Vue component .d.ts files");
  console.error("      --declaration-dir <dir>    Output directory for declarations");
  console.error("      --max-warnings <number>    Fail when warnings exceed the limit");
  console.error("  -c, --config <path>            Use a specific vize config file");
  console.error("      --no-config                Disable config discovery");
  console.error("");
  console.error(
    "Note: npm `vize check` uses the packaged NAPI checker. Install the Rust CLI for project-backed Corsa diagnostics.",
  );
}

function printUpgradeUsage(): void {
  console.error("Usage: vize upgrade [options]");
  console.error("Options:");
  console.error("      --package-manager <name>   npm, pnpm, yarn, bun, or vp");
  console.error("  -g, --global                   Upgrade the global installation");
  console.error("      --dry-run                  Print the command without running it");
}

function printReadyUsage(): void {
  console.error("Usage: vize ready [options] [files-or-directories]");
  console.error("Runs: fmt --write -> lint -> check -> build");
  console.error("Options:");
  console.error("  -o, --output <dir>             Output directory for build");
  console.error("      --ssr                      Enable SSR compilation for build");
  console.error("      --script-ext <mode>        preserve or downcompile");
}

function resolvePackageBinaryFromCwd(packageName: string, binName: string = packageName): string {
  const cwdRequire = createRequire(pathToFileURL(path.join(process.cwd(), "package.json")).href);
  const packageJsonPath = cwdRequire.resolve(`${packageName}/package.json`);
  const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8")) as {
    bin?: string | Record<string, string>;
  };

  const bin = typeof packageJson.bin === "string" ? packageJson.bin : packageJson.bin?.[binName];

  if (!bin) {
    throw new Error(`Could not resolve binary '${binName}' from package '${packageName}'`);
  }

  return path.resolve(path.dirname(packageJsonPath), bin);
}

function runMusea(args: string[]): void {
  const isHelp = args.includes("--help") || args.includes("-h");
  if (isHelp) {
    console.error("Usage: vize musea [--build] [...vite options]");
    console.error("  --build    Run `vite build` instead of `vite dev`");
    return;
  }

  const isBuild = args.includes("--build");
  const viteArgs = args.filter((arg) => arg !== "--build");
  const viteCommand = isBuild ? "build" : "dev";
  const viteBin = resolvePackageBinaryFromCwd("vite");
  const result = spawnSync(process.execPath, [viteBin, viteCommand, ...viteArgs], {
    stdio: "inherit",
    cwd: process.cwd(),
    env: process.env,
  });

  if (result.error) {
    throw result.error;
  }

  process.exit(result.status ?? 1);
}

function parseLintCommand(args: string[]): ParsedLintCommand {
  const patterns: string[] = [];
  const options: LintOptions = {};
  const sharedConfig: SharedConfigOptions = {
    configMode: "root",
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--format" || arg === "-f") {
      options.format = args[++i];
    } else if (arg === "--max-warnings") {
      options.maxWarnings = Number.parseInt(args[++i], 10);
    } else if (arg === "--quiet" || arg === "-q") {
      options.quiet = true;
    } else if (arg === "--fix") {
      options.fix = true;
    } else if (arg === "--help-level") {
      options.helpLevel = args[++i];
    } else if (arg === "--preset") {
      options.preset = args[++i];
    } else if (arg === "--config" || arg === "-c") {
      const configFile = args[++i];
      if (!configFile) {
        throw new Error("Missing path after --config");
      }
      sharedConfig.configFile = configFile;
    } else if (arg === "--no-config") {
      sharedConfig.configMode = "none";
    } else if (!arg.startsWith("-")) {
      patterns.push(arg);
    }
  }

  return { patterns, options, sharedConfig };
}

// ============================================================================
// Build command
// ============================================================================

interface NativeBuildOptions {
  ssr?: boolean;
  vapor?: boolean;
  customRenderer?: boolean;
  custom_renderer?: boolean;
  isTs?: boolean;
  is_ts?: boolean;
  threads?: number;
}

interface BatchFileInput {
  path: string;
  source: string;
}

interface BatchFileResult {
  path: string;
  code: string;
  css?: string;
  errors: string[];
  warnings: string[];
  scopeId?: string;
  scope_id?: string;
  hasScoped?: boolean;
  has_scoped?: boolean;
}

interface BatchCompileResult {
  results: BatchFileResult[];
  successCount?: number;
  success_count?: number;
  failedCount?: number;
  failed_count?: number;
  timeMs?: number;
  time_ms?: number;
}

interface BuildOptions {
  output: string;
  format: "js" | "json" | "stats";
  ssr?: boolean;
  vapor?: boolean;
  customRenderer?: boolean;
  scriptExt: "preserve" | "downcompile";
  threads?: number;
  help?: boolean;
}

interface ParsedBuildCommand {
  patterns: string[];
  options: BuildOptions;
  sharedConfig: SharedConfigOptions;
}

function parseBuildCommand(args: string[]): ParsedBuildCommand {
  const patterns: string[] = [];
  const options: BuildOptions = {
    output: "./dist",
    format: "js",
    scriptExt: "downcompile",
  };
  const sharedConfig: SharedConfigOptions = {
    configMode: "root",
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--output" || arg === "-o") {
      options.output = args[++i] ?? options.output;
    } else if (arg === "--format" || arg === "-f") {
      const format = args[++i];
      if (format === "js" || format === "json" || format === "stats") {
        options.format = format;
      }
    } else if (arg === "--ssr") {
      options.ssr = true;
    } else if (arg === "--vapor") {
      options.vapor = true;
    } else if (arg === "--custom-renderer") {
      options.customRenderer = true;
    } else if (arg === "--script-ext") {
      const scriptExt = args[++i];
      if (scriptExt === "preserve" || scriptExt === "downcompile") {
        options.scriptExt = scriptExt;
      }
    } else if (arg === "--threads" || arg === "-j") {
      options.threads = Number.parseInt(args[++i], 10);
    } else if (arg === "--config" || arg === "-c") {
      const configFile = args[++i];
      if (!configFile) {
        throw new Error("Missing path after --config");
      }
      sharedConfig.configFile = configFile;
    } else if (arg === "--no-config") {
      sharedConfig.configMode = "none";
    } else if (arg === "--profile" || arg === "--continue-on-error") {
      // Accepted for command compatibility. The npm build path prints a compact summary.
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else if (!arg.startsWith("-")) {
      patterns.push(arg);
    }
  }

  return { patterns, options, sharedConfig };
}

function getScriptLang(source: string): string {
  const match = source.match(/<script\b[^>]*\blang=["']([^"']+)["']/i);
  return match?.[1] ?? "js";
}

function getOutputExtension(source: string, scriptExt: BuildOptions["scriptExt"]): string {
  if (scriptExt === "downcompile") {
    return "js";
  }
  const lang = getScriptLang(source);
  return lang === "ts" || lang === "tsx" || lang === "jsx" ? lang : "js";
}

function outputFileName(file: string, extension: string): string {
  return path.basename(file).replace(/\.vue$/i, `.${extension}`);
}

function toNativeBuildOptions(options: BuildOptions): NativeBuildOptions {
  const isTs = options.scriptExt === "preserve";
  return {
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer,
    custom_renderer: options.customRenderer,
    isTs,
    is_ts: isTs,
    threads: options.threads,
  };
}

async function runBuild(args: string[]): Promise<void> {
  const { patterns, options, sharedConfig } = parseBuildCommand(args);
  if (options.help) {
    printBuildUsage();
    return;
  }

  const config = await loadConfig(process.cwd(), {
    mode: sharedConfig.configMode,
    configFile: sharedConfig.configFile,
    env: {
      mode: process.env.NODE_ENV ?? "development",
      command: "build",
    },
  });

  if (sharedConfig.configFile && !config) {
    throw new Error(`Could not find config file: ${sharedConfig.configFile}`);
  }

  options.ssr ??= config?.compiler?.ssr;
  options.vapor ??= config?.compiler?.vapor;
  options.customRenderer ??= config?.compiler?.customRenderer;
  if (config?.compiler?.scriptExt === "ts") {
    options.scriptExt = "preserve";
  } else if (config?.compiler?.scriptExt === "js") {
    options.scriptExt = "downcompile";
  }

  const files = collectVueFiles(patterns);
  if (files.length === 0) {
    process.stderr.write(`No Vue files found matching inputs: ${JSON.stringify(patterns)}\n`);
    process.exit(1);
  }

  const inputs = files.map((file) => ({
    path: file,
    source: readFileSync(file, "utf8"),
  }));
  const native = loadNative("build");
  const startedAt = performance.now();
  const result = native.compileSfcBatchWithResults(inputs, toNativeBuildOptions(options));
  const timeMs = result.timeMs ?? result.time_ms ?? performance.now() - startedAt;
  const results = [...result.results].sort((left, right) => left.path.localeCompare(right.path));

  if (options.format !== "stats") {
    mkdirSync(options.output, { recursive: true });
  }

  for (const fileResult of results) {
    const source = inputs.find((input) => input.path === fileResult.path)?.source ?? "";
    for (const warning of fileResult.warnings) {
      process.stderr.write(`warning: ${displayPath(fileResult.path)} ${warning}\n`);
    }
    for (const error of fileResult.errors) {
      process.stderr.write(`error: ${displayPath(fileResult.path)} ${error}\n`);
    }

    if (fileResult.errors.length > 0 || options.format === "stats") {
      continue;
    }

    const extension =
      options.format === "json" ? "json" : getOutputExtension(source, options.scriptExt);
    const outputPath = path.join(options.output, outputFileName(fileResult.path, extension));
    const content =
      options.format === "json" ? JSON.stringify(fileResult, null, 2) : fileResult.code;
    writeFileSync(outputPath, content);
  }

  const failed =
    result.failedCount ?? result.failed_count ?? results.filter((r) => r.errors.length).length;
  const success = result.successCount ?? result.success_count ?? results.length - failed;
  process.stderr.write(
    `\x1b[32mOK\x1b[0m Built ${success} Vue file(s) in ${timeMs.toFixed(2)}ms\n`,
  );

  if (failed > 0) {
    process.stderr.write(`\x1b[31mERR\x1b[0m ${failed} file(s) failed\n`);
    process.exit(1);
  }
}

// ============================================================================
// Format command
// ============================================================================

interface NativeFormatOptions {
  printWidth?: number;
  print_width?: number;
  tabWidth?: number;
  tab_width?: number;
  useTabs?: boolean;
  use_tabs?: boolean;
  semi?: boolean;
  singleQuote?: boolean;
  single_quote?: boolean;
  sortAttributes?: boolean;
  sort_attributes?: boolean;
  singleAttributePerLine?: boolean;
  single_attribute_per_line?: boolean;
  maxAttributesPerLine?: number;
  max_attributes_per_line?: number;
  normalizeDirectiveShorthands?: boolean;
  normalize_directive_shorthands?: boolean;
}

interface FormatResult {
  code: string;
  changed: boolean;
}

interface FmtOptions extends NativeFormatOptions {
  check?: boolean;
  write?: boolean;
  help?: boolean;
}

interface ParsedFmtCommand {
  patterns: string[];
  options: FmtOptions;
  sharedConfig: SharedConfigOptions;
}

function parseFmtCommand(args: string[]): ParsedFmtCommand {
  const patterns: string[] = [];
  const options: FmtOptions = {};
  const sharedConfig: SharedConfigOptions = {
    configMode: "root",
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--check") {
      options.check = true;
    } else if (arg === "--write" || arg === "-w") {
      options.write = true;
    } else if (arg === "--single-quote") {
      options.singleQuote = true;
    } else if (arg === "--print-width") {
      options.printWidth = Number.parseInt(args[++i], 10);
    } else if (arg === "--tab-width") {
      options.tabWidth = Number.parseInt(args[++i], 10);
    } else if (arg === "--use-tabs") {
      options.useTabs = true;
    } else if (arg === "--no-semi") {
      options.semi = false;
    } else if (arg === "--sort-attributes") {
      options.sortAttributes = true;
    } else if (arg === "--single-attribute-per-line") {
      options.singleAttributePerLine = true;
    } else if (arg === "--max-attributes-per-line") {
      options.maxAttributesPerLine = Number.parseInt(args[++i], 10);
    } else if (arg === "--normalize-directive-shorthands") {
      options.normalizeDirectiveShorthands = true;
    } else if (arg === "--config" || arg === "-c") {
      const configFile = args[++i];
      if (!configFile) {
        throw new Error("Missing path after --config");
      }
      sharedConfig.configFile = configFile;
    } else if (arg === "--no-config") {
      sharedConfig.configMode = "none";
    } else if (arg === "--profile") {
      // Accepted for command compatibility. The npm fmt path prints a compact summary.
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else if (!arg.startsWith("-")) {
      patterns.push(arg);
    }
  }

  return { patterns, options, sharedConfig };
}

function toNativeFormatOptions(options: FmtOptions): NativeFormatOptions {
  return {
    printWidth: options.printWidth,
    print_width: options.printWidth,
    tabWidth: options.tabWidth,
    tab_width: options.tabWidth,
    useTabs: options.useTabs,
    use_tabs: options.useTabs,
    semi: options.semi,
    singleQuote: options.singleQuote,
    single_quote: options.singleQuote,
    sortAttributes: options.sortAttributes,
    sort_attributes: options.sortAttributes,
    singleAttributePerLine: options.singleAttributePerLine,
    single_attribute_per_line: options.singleAttributePerLine,
    maxAttributesPerLine: options.maxAttributesPerLine,
    max_attributes_per_line: options.maxAttributesPerLine,
    normalizeDirectiveShorthands: options.normalizeDirectiveShorthands,
    normalize_directive_shorthands: options.normalizeDirectiveShorthands,
  };
}

async function runFmt(args: string[]): Promise<void> {
  const { patterns, options, sharedConfig } = parseFmtCommand(args);
  if (options.help) {
    printFmtUsage();
    return;
  }

  const config = await loadConfig(process.cwd(), {
    mode: sharedConfig.configMode,
    configFile: sharedConfig.configFile,
    env: {
      mode: process.env.NODE_ENV ?? "development",
      command: "fmt",
    },
  });

  if (sharedConfig.configFile && !config) {
    throw new Error(`Could not find config file: ${sharedConfig.configFile}`);
  }

  options.printWidth ??= config?.formatter?.printWidth;
  options.tabWidth ??= config?.formatter?.tabWidth;
  options.useTabs ??= config?.formatter?.useTabs;
  options.semi ??= config?.formatter?.semi;
  options.singleQuote ??= config?.formatter?.singleQuote;

  const files = collectVueFiles(patterns);
  if (files.length === 0) {
    process.stderr.write(`No Vue files found matching inputs: ${JSON.stringify(patterns)}\n`);
    return;
  }

  const native = loadNative("fmt");
  let changed = 0;
  let errored = 0;

  for (const file of files) {
    const source = readFileSync(file, "utf8");
    try {
      const result = native.formatSfc(source, toNativeFormatOptions(options));
      if (!result.changed) {
        continue;
      }
      changed++;
      if (options.check) {
        process.stderr.write(`Would reformat: ${displayPath(file)}\n`);
      } else if (options.write) {
        writeFileSync(file, result.code);
        process.stderr.write(`Reformatted: ${displayPath(file)}\n`);
      } else {
        process.stderr.write(`Would reformat: ${displayPath(file)}\n`);
      }
    } catch (error) {
      errored++;
      process.stderr.write(
        `Error formatting ${displayPath(file)}: ${error instanceof Error ? error.message : String(error)}\n`,
      );
    }
  }

  process.stderr.write(
    `\x1b[32mOK\x1b[0m Formatted ${files.length} Vue file(s), ${changed} changed\n`,
  );

  if (errored > 0 || (options.check && changed > 0)) {
    process.exit(1);
  }
}

// ============================================================================
// Check command
// ============================================================================

interface NativeTypeCheckOptions {
  filename?: string;
  strict?: boolean;
  includeVirtualTs?: boolean;
  include_virtual_ts?: boolean;
  checkProps?: boolean;
  check_props?: boolean;
  checkEmits?: boolean;
  check_emits?: boolean;
  checkTemplateBindings?: boolean;
  check_template_bindings?: boolean;
  checkReactivity?: boolean;
  check_reactivity?: boolean;
  checkSetupContext?: boolean;
  check_setup_context?: boolean;
  checkInvalidExports?: boolean;
  check_invalid_exports?: boolean;
  checkFallthroughAttrs?: boolean;
  check_fallthrough_attrs?: boolean;
}

interface NativeDeclarationOptions {
  filename?: string;
}

interface DeclarationResult {
  code: string;
}

interface TypeDiagnostic {
  severity: string;
  message: string;
  start: number;
  end: number;
  code?: string;
  help?: string;
  related?: Array<{
    message: string;
    start: number;
    end: number;
    filename?: string;
  }>;
}

interface TypeCheckResult {
  diagnostics: TypeDiagnostic[];
  virtualTs?: string;
  errorCount: number;
  warningCount: number;
  analysisTimeMs?: number;
}

interface CheckOptions {
  format?: string;
  quiet?: boolean;
  strict?: boolean;
  includeVirtualTs?: boolean;
  maxWarnings?: number;
  checkProps?: boolean;
  checkEmits?: boolean;
  checkTemplateBindings?: boolean;
  checkReactivity?: boolean;
  checkSetupContext?: boolean;
  checkInvalidExports?: boolean;
  checkFallthroughAttrs?: boolean;
  declaration?: boolean;
  declarationDir?: string;
  help?: boolean;
}

interface ParsedCheckCommand {
  patterns: string[];
  options: CheckOptions;
  sharedConfig: SharedConfigOptions;
}

interface CheckedFileResult {
  file: string;
  source: string;
  result: TypeCheckResult;
}

interface EmittedDeclaration {
  file: string;
  path: string;
}

function parseCheckCommand(args: string[]): ParsedCheckCommand {
  const patterns: string[] = [];
  const options: CheckOptions = {};
  const sharedConfig: SharedConfigOptions = {
    configMode: "root",
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--format" || arg === "-f") {
      options.format = args[++i];
    } else if (arg === "--quiet" || arg === "-q") {
      options.quiet = true;
    } else if (arg === "--strict") {
      options.strict = true;
    } else if (arg === "--no-strict") {
      options.strict = false;
    } else if (arg === "--show-virtual-ts" || arg === "--include-virtual-ts") {
      options.includeVirtualTs = true;
    } else if (arg === "--max-warnings") {
      options.maxWarnings = Number.parseInt(args[++i], 10);
    } else if (arg === "--no-check-props") {
      options.checkProps = false;
    } else if (arg === "--no-check-emits") {
      options.checkEmits = false;
    } else if (arg === "--no-check-template-bindings") {
      options.checkTemplateBindings = false;
    } else if (arg === "--no-check-reactivity") {
      options.checkReactivity = false;
    } else if (arg === "--no-check-setup-context") {
      options.checkSetupContext = false;
    } else if (arg === "--no-check-invalid-exports") {
      options.checkInvalidExports = false;
    } else if (arg === "--no-check-fallthrough-attrs") {
      options.checkFallthroughAttrs = false;
    } else if (arg === "--declaration") {
      options.declaration = true;
    } else if (arg === "--declaration-dir") {
      const declarationDir = args[++i];
      if (!declarationDir) {
        throw new Error("Missing path after --declaration-dir");
      }
      options.declarationDir = declarationDir;
    } else if (arg === "--config" || arg === "-c") {
      const configFile = args[++i];
      if (!configFile) {
        throw new Error("Missing path after --config");
      }
      sharedConfig.configFile = configFile;
    } else if (arg === "--no-config") {
      sharedConfig.configMode = "none";
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else if (arg === "--tsconfig" || arg === "--corsa-path" || arg === "--servers") {
      i++;
    } else if (arg === "--socket" || arg === "-s") {
      i++;
    } else if (arg === "--profile") {
      // Accepted for package-script compatibility with the Rust CLI.
    } else if (!arg.startsWith("-")) {
      patterns.push(arg);
    }
  }

  return { patterns, options, sharedConfig };
}

function hasGlobSyntax(pattern: string): boolean {
  return pattern.includes("*") || pattern.includes("?") || pattern.includes("[");
}

function normalizePath(filePath: string): string {
  return filePath.split(path.sep).join("/");
}

function displayPath(filePath: string): string {
  const relative = path.relative(process.cwd(), filePath);
  if (relative && !relative.startsWith("..") && !path.isAbsolute(relative)) {
    return normalizePath(relative);
  }
  return normalizePath(filePath);
}

function isVueFile(filePath: string): boolean {
  return path.extname(filePath) === ".vue";
}

function collectVueFilesFromDirectory(directory: string, recursive: boolean): string[] {
  const files: string[] = [];
  const entries = readdirSync(directory, { withFileTypes: true });

  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "node_modules" || entry.name === ".git") {
        continue;
      }
      if (recursive) {
        files.push(...collectVueFilesFromDirectory(entryPath, true));
      }
    } else if (entry.isFile() && isVueFile(entryPath)) {
      files.push(entryPath);
    }
  }

  return files;
}

function globBase(pattern: string): string {
  const normalized = normalizePath(pattern);
  const globIndex = normalized.search(/[*?[]/);
  if (globIndex === -1) {
    return normalized;
  }

  const beforeGlob = normalized.slice(0, globIndex);
  const slashIndex = beforeGlob.lastIndexOf("/");
  if (slashIndex === -1) {
    return ".";
  }
  return beforeGlob.slice(0, slashIndex) || "/";
}

function globToRegExp(pattern: string): RegExp {
  const normalized = normalizePath(pattern);
  let source = "";

  for (let i = 0; i < normalized.length; i++) {
    const char = normalized[i];
    const next = normalized[i + 1];
    const afterNext = normalized[i + 2];

    if (char === "*" && next === "*" && afterNext === "/") {
      source += "(?:.*/)?";
      i += 2;
    } else if (char === "*" && next === "*") {
      source += ".*";
      i++;
    } else if (char === "*") {
      source += "[^/]*";
    } else if (char === "?") {
      source += "[^/]";
    } else if ("\\^$+?.()|{}[]".includes(char)) {
      source += `\\${char}`;
    } else {
      source += char;
    }
  }

  return new RegExp(`^${source}$`);
}

function shouldRecurseGlob(pattern: string, base: string): boolean {
  const normalizedPattern = normalizePath(pattern);
  const normalizedBase = normalizePath(base);
  const rest =
    normalizedBase === "."
      ? normalizedPattern
      : normalizedPattern.slice(normalizedBase.length).replace(/^\/+/, "");
  return rest.includes("/");
}

function collectVueFilesFromGlob(pattern: string): string[] {
  const basePattern = globBase(pattern);
  const base = path.resolve(process.cwd(), basePattern);
  if (!existsSync(base)) {
    return [];
  }

  const isAbsolutePattern = path.isAbsolute(pattern);
  const normalizedPattern = normalizePath(isAbsolutePattern ? path.resolve(pattern) : pattern);
  const regex = globToRegExp(normalizedPattern);
  const candidates = collectVueFilesFromDirectory(base, shouldRecurseGlob(pattern, basePattern));

  return candidates.filter((file) => {
    const comparable = isAbsolutePattern
      ? normalizePath(file)
      : normalizePath(path.relative(process.cwd(), file));
    return regex.test(comparable);
  });
}

function collectVueFiles(patterns: string[]): string[] {
  const files = new Set<string>();
  const inputs = patterns.length === 0 ? ["."] : patterns;

  for (const input of inputs) {
    if (hasGlobSyntax(input)) {
      for (const file of collectVueFilesFromGlob(input)) {
        files.add(path.resolve(file));
      }
      continue;
    }

    const resolved = path.resolve(process.cwd(), input);
    if (!existsSync(resolved)) {
      continue;
    }

    const stats = statSync(resolved);
    if (stats.isDirectory()) {
      for (const file of collectVueFilesFromDirectory(resolved, true)) {
        files.add(path.resolve(file));
      }
    } else if (stats.isFile() && isVueFile(resolved)) {
      files.add(resolved);
    }
  }

  return Array.from(files).sort();
}

function commonSourceDirectory(results: CheckedFileResult[]): string {
  let common = path.dirname(results[0]?.file ?? process.cwd());

  for (let i = 1; i < results.length; i++) {
    const directory = path.dirname(results[i].file);
    while (common !== path.dirname(common)) {
      const relative = path.relative(common, directory);
      if (relative !== ".." && !relative.startsWith(`..${path.sep}`)) {
        break;
      }
      common = path.dirname(common);
    }
  }

  return common;
}

function emitCheckDeclarations(
  results: CheckedFileResult[],
  native: NativeBinding,
  options: CheckOptions,
): EmittedDeclaration[] {
  if (!options.declaration) {
    return [];
  }

  if (typeof native.generateDeclaration !== "function") {
    throw new Error("The loaded native binding does not support declaration generation.");
  }

  const outDir = path.resolve(process.cwd(), options.declarationDir ?? "dist/types");
  const sourceRoot = commonSourceDirectory(results);
  const declarations: EmittedDeclaration[] = [];

  for (const { file, source } of results) {
    const relative = normalizePath(path.relative(sourceRoot, file));
    const outputPath = path.join(outDir, `${relative}.d.ts`);
    mkdirSync(path.dirname(outputPath), { recursive: true });

    const declaration = native.generateDeclaration(source, { filename: file });
    writeFileSync(outputPath, declaration.code);
    declarations.push({
      file: displayPath(outputPath),
      path: outputPath,
    });
  }

  return declarations;
}

function lineStarts(source: string): number[] {
  const starts = [0];
  for (let i = 0; i < source.length; i++) {
    if (source.charCodeAt(i) === 10) {
      starts.push(i + 1);
    }
  }
  return starts;
}

function offsetToLineColumn(starts: number[], offset: number): { line: number; column: number } {
  let low = 0;
  let high = starts.length - 1;
  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if (starts[mid] <= offset) {
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }

  const lineIndex = Math.max(0, high);
  return {
    line: lineIndex + 1,
    column: offset - starts[lineIndex] + 1,
  };
}

function toNativeTypeCheckOptions(file: string, options: CheckOptions): NativeTypeCheckOptions {
  return {
    filename: file,
    strict: options.strict,
    includeVirtualTs: options.includeVirtualTs,
    include_virtual_ts: options.includeVirtualTs,
    checkProps: options.checkProps,
    check_props: options.checkProps,
    checkEmits: options.checkEmits,
    check_emits: options.checkEmits,
    checkTemplateBindings: options.checkTemplateBindings,
    check_template_bindings: options.checkTemplateBindings,
    checkReactivity: options.checkReactivity,
    check_reactivity: options.checkReactivity,
    checkSetupContext: options.checkSetupContext,
    check_setup_context: options.checkSetupContext,
    checkInvalidExports: options.checkInvalidExports,
    check_invalid_exports: options.checkInvalidExports,
    checkFallthroughAttrs: options.checkFallthroughAttrs,
    check_fallthrough_attrs: options.checkFallthroughAttrs,
  };
}

function renderCheckText(
  results: CheckedFileResult[],
  options: CheckOptions,
  timeMs: number,
  declarations: EmittedDeclaration[] = [],
): void {
  let totalErrors = 0;
  let totalWarnings = 0;

  for (const { file, source, result } of results) {
    totalErrors += result.errorCount;
    totalWarnings += result.warningCount;

    if (options.includeVirtualTs && result.virtualTs) {
      process.stderr.write(`\n=== ${displayPath(file)} ===\n${result.virtualTs}\n`);
    }

    if (options.quiet || result.diagnostics.length === 0) {
      continue;
    }

    const starts = lineStarts(source);
    process.stdout.write(`\n\x1b[4m${displayPath(file)}\x1b[0m\n`);
    for (const diagnostic of result.diagnostics) {
      const color = diagnostic.severity === "error" ? "\x1b[31m" : "\x1b[33m";
      const location = offsetToLineColumn(starts, diagnostic.start);
      const code = diagnostic.code ? ` [${diagnostic.code}]` : "";
      process.stdout.write(
        `  ${color}${diagnostic.severity}:${location.line}:${location.column}\x1b[0m${code} ${diagnostic.message}\n`,
      );
      if (diagnostic.help) {
        process.stdout.write(`    help: ${diagnostic.help}\n`);
      }
    }
  }

  const status = totalErrors > 0 ? "\x1b[31mERR\x1b[0m" : "\x1b[32mOK\x1b[0m";
  process.stdout.write(
    `\n${status} Type checked ${results.length} Vue files in ${timeMs.toFixed(2)}ms\n`,
  );
  if (totalErrors > 0) {
    process.stdout.write(`  \x1b[31m${totalErrors} error(s)\x1b[0m\n`);
  } else {
    process.stdout.write("  \x1b[32mNo type errors found!\x1b[0m\n");
  }
  if (totalWarnings > 0) {
    process.stdout.write(`  \x1b[33m${totalWarnings} warning(s)\x1b[0m\n`);
  }
  if (declarations.length > 0) {
    process.stdout.write(`  \x1b[32mEmitted ${declarations.length} declaration file(s)\x1b[0m\n`);
  }
}

async function runCheck(args: string[]): Promise<void> {
  const { patterns, options, sharedConfig } = parseCheckCommand(args);
  if (options.help) {
    printCheckUsage();
    return;
  }

  const config = await loadConfig(process.cwd(), {
    mode: sharedConfig.configMode,
    configFile: sharedConfig.configFile,
    env: {
      mode: process.env.NODE_ENV ?? "development",
      command: "check",
    },
  });

  if (sharedConfig.configFile && !config) {
    throw new Error(`Could not find config file: ${sharedConfig.configFile}`);
  }

  if (config?.typeChecker?.enabled === false) {
    process.stderr.write(
      "[vize] Skipping check because typeChecker.enabled is false in vize.config.\n",
    );
    return;
  }

  options.strict ??= config?.typeChecker?.strict;
  options.checkProps ??= config?.typeChecker?.checkProps;
  options.checkEmits ??= config?.typeChecker?.checkEmits;
  options.checkTemplateBindings ??= config?.typeChecker?.checkTemplateBindings;

  const files = collectVueFiles(patterns);
  if (files.length === 0) {
    process.stderr.write(`No Vue files found matching inputs: ${JSON.stringify(patterns)}\n`);
    return;
  }

  const native = loadNative("check");
  const start = performance.now();
  const results = files.map((file) => {
    const source = readFileSync(file, "utf8");
    return {
      file,
      source,
      result: native.typeCheck(source, toNativeTypeCheckOptions(file, options)),
    };
  });
  const timeMs = performance.now() - start;
  const declarations = emitCheckDeclarations(results, native, options);
  const totalErrors = results.reduce((sum, { result }) => sum + result.errorCount, 0);
  const totalWarnings = results.reduce((sum, { result }) => sum + result.warningCount, 0);

  if (options.format === "json") {
    process.stdout.write(
      `${JSON.stringify(
        {
          files: results.map(({ file, result }) => ({
            file: displayPath(file),
            diagnostics: result.diagnostics,
            virtualTs: result.virtualTs,
          })),
          errorCount: totalErrors,
          warningCount: totalWarnings,
          fileCount: results.length,
          declarations: declarations.map(({ file }) => file),
        },
        null,
        2,
      )}\n`,
    );
  } else {
    renderCheckText(results, options, timeMs, declarations);
  }

  if (totalErrors > 0) {
    process.exit(1);
  }

  if (options.maxWarnings !== undefined && totalWarnings > options.maxWarnings) {
    process.stderr.write(`\nToo many warnings (${totalWarnings} > max ${options.maxWarnings})\n`);
    process.exit(1);
  }
}

async function runLint(args: string[]): Promise<void> {
  const { patterns, options, sharedConfig } = parseLintCommand(args);
  const config = await loadConfig(process.cwd(), {
    mode: sharedConfig.configMode,
    configFile: sharedConfig.configFile,
    env: {
      mode: process.env.NODE_ENV ?? "development",
      command: "lint",
    },
  });

  if (sharedConfig.configFile && !config) {
    throw new Error(`Could not find config file: ${sharedConfig.configFile}`);
  }

  if (config?.linter?.enabled === false) {
    process.stderr.write("[vize] Skipping lint because linter.enabled is false in vize.config.\n");
    return;
  }

  options.preset ??= config?.linter?.preset;

  if (patterns.length === 0) {
    patterns.push(".");
  }

  const native = loadNative("lint");
  const result = native.lint(patterns, {
    format: options.format,
    max_warnings: options.maxWarnings,
    quiet: options.quiet,
    fix: options.fix,
    help_level: options.helpLevel,
    preset: options.preset,
  });

  if (result.output) {
    process.stdout.write(result.output);
    if (!result.output.endsWith("\n")) {
      process.stdout.write("\n");
    }
  }

  if (options.fix) {
    process.stderr.write("\nNote: --fix is not yet implemented\n");
  }

  if (result.errorCount > 0) {
    process.exit(1);
  }

  if (options.maxWarnings !== undefined && result.warningCount > options.maxWarnings) {
    process.stderr.write(
      `\nToo many warnings (${result.warningCount} > max ${options.maxWarnings})\n`,
    );
    process.exit(1);
  }
}

// ============================================================================
// Upgrade command
// ============================================================================

type PackageManager = "bun" | "npm" | "pnpm" | "vp" | "yarn";

interface UpgradeOptions {
  packageManager?: PackageManager;
  global?: boolean;
  dryRun?: boolean;
  help?: boolean;
}

function parseUpgradeCommand(args: string[]): UpgradeOptions {
  const options: UpgradeOptions = {};

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--package-manager") {
      const packageManager = args[++i];
      if (
        packageManager === "bun" ||
        packageManager === "npm" ||
        packageManager === "pnpm" ||
        packageManager === "vp" ||
        packageManager === "yarn"
      ) {
        options.packageManager = packageManager;
      }
    } else if (arg === "--global" || arg === "-g") {
      options.global = true;
    } else if (arg === "--dry-run") {
      options.dryRun = true;
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    }
  }

  return options;
}

function readCwdPackageJson(): {
  packageManager?: string;
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
} | null {
  const packageJsonPath = path.join(process.cwd(), "package.json");
  if (!existsSync(packageJsonPath)) {
    return null;
  }
  return JSON.parse(readFileSync(packageJsonPath, "utf8"));
}

function detectPackageManager(explicit?: PackageManager): PackageManager {
  if (explicit) {
    return explicit;
  }

  const userAgent = process.env.npm_config_user_agent ?? "";
  if (userAgent.startsWith("pnpm")) {
    return "pnpm";
  }
  if (userAgent.startsWith("yarn")) {
    return "yarn";
  }
  if (userAgent.startsWith("bun")) {
    return "bun";
  }
  if (userAgent.startsWith("npm")) {
    return "npm";
  }

  const packageManager = readCwdPackageJson()?.packageManager;
  if (packageManager?.startsWith("pnpm")) {
    return "pnpm";
  }
  if (packageManager?.startsWith("yarn")) {
    return "yarn";
  }
  if (packageManager?.startsWith("bun")) {
    return "bun";
  }
  return "npm";
}

function buildUpgradeCommand(
  packageManager: PackageManager,
  options: UpgradeOptions,
): { command: string; args: string[] } {
  const packageJson = readCwdPackageJson();
  const saveDev = !packageJson?.dependencies?.vize;
  const packageSpec = "vize@latest";

  if (packageManager === "vp") {
    return {
      command: "vp",
      args: ["install", ...(options.global ? ["-g"] : saveDev ? ["-D"] : []), packageSpec],
    };
  }
  if (packageManager === "pnpm") {
    return {
      command: "pnpm",
      args: ["add", ...(options.global ? ["-g"] : saveDev ? ["-D"] : []), packageSpec],
    };
  }
  if (packageManager === "yarn") {
    return {
      command: "yarn",
      args: options.global
        ? ["global", "add", packageSpec]
        : ["add", ...(saveDev ? ["-D"] : []), packageSpec],
    };
  }
  if (packageManager === "bun") {
    return {
      command: "bun",
      args: ["add", ...(options.global ? ["-g"] : saveDev ? ["-d"] : []), packageSpec],
    };
  }
  return {
    command: "npm",
    args: ["install", ...(options.global ? ["-g"] : saveDev ? ["-D"] : []), packageSpec],
  };
}

function runUpgrade(args: string[]): void {
  const options = parseUpgradeCommand(args);
  if (options.help) {
    printUpgradeUsage();
    return;
  }

  const packageManager = detectPackageManager(options.packageManager);
  const command = buildUpgradeCommand(packageManager, options);

  if (options.dryRun) {
    process.stdout.write(`${command.command} ${command.args.join(" ")}\n`);
    return;
  }

  const result = spawnSync(command.command, command.args, {
    stdio: "inherit",
    cwd: process.cwd(),
    env: process.env,
  });

  if (result.error) {
    throw result.error;
  }

  process.exit(result.status ?? 1);
}

// ============================================================================
// Ready command
// ============================================================================

interface ReadyOptions {
  output: string;
  ssr?: boolean;
  scriptExt: "preserve" | "downcompile";
  help?: boolean;
}

interface ParsedReadyCommand {
  patterns: string[];
  options: ReadyOptions;
}

function parseReadyCommand(args: string[]): ParsedReadyCommand {
  const patterns: string[] = [];
  const options: ReadyOptions = {
    output: "./dist",
    scriptExt: "downcompile",
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--output" || arg === "-o") {
      options.output = args[++i] ?? options.output;
    } else if (arg === "--ssr") {
      options.ssr = true;
    } else if (arg === "--script-ext") {
      const scriptExt = args[++i];
      if (scriptExt === "preserve" || scriptExt === "downcompile") {
        options.scriptExt = scriptExt;
      }
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else if (!arg.startsWith("-")) {
      patterns.push(arg);
    }
  }

  return { patterns, options };
}

async function runReady(args: string[]): Promise<void> {
  const { patterns, options } = parseReadyCommand(args);
  if (options.help) {
    printReadyUsage();
    return;
  }

  process.stderr.write("vize ready: fmt\n");
  await runFmt(["--write", ...patterns]);

  process.stderr.write("vize ready: lint\n");
  await runLint(patterns);

  process.stderr.write("vize ready: check\n");
  await runCheck(patterns);

  process.stderr.write("vize ready: build\n");
  await runBuild([
    "--output",
    options.output,
    "--script-ext",
    options.scriptExt,
    ...(options.ssr ? ["--ssr"] : []),
    ...patterns,
  ]);
}

// ============================================================================
// Command router
// ============================================================================

const NAPI_COMMANDS = new Set(["build", "check", "fmt", "lint"]);
const JS_COMMANDS = new Set(["musea", "ready", "upgrade"]);

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const command = args[0];

  if (!command || command === "--help" || command === "-h") {
    printUsage();
    process.exit(1);
  }

  if (NAPI_COMMANDS.has(command)) {
    const commandArgs = args.slice(1);
    switch (command) {
      case "build":
        await runBuild(commandArgs);
        break;
      case "check":
        await runCheck(commandArgs);
        break;
      case "fmt":
        await runFmt(commandArgs);
        break;
      case "lint":
        await runLint(commandArgs);
        break;
    }
  } else if (JS_COMMANDS.has(command)) {
    const commandArgs = args.slice(1);
    switch (command) {
      case "musea":
        runMusea(commandArgs);
        break;
      case "ready":
        await runReady(commandArgs);
        break;
      case "upgrade":
        runUpgrade(commandArgs);
        break;
    }
  } else {
    printUsage();
    console.error(`Unknown command: ${command}`);
    console.error(
      "For commands not yet available via NAPI, install from source: cargo install vize",
    );
    process.exit(1);
  }
}

if (!import.meta.vitest) {
  void main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}

if (import.meta.vitest) {
  const { describe, expect, it } = import.meta.vitest;

  describe("shouldPreferWorkspaceBinding", () => {
    it("detects the local workspace native package", () => {
      expect(
        shouldPreferWorkspaceBinding(
          `${path.sep}Users${path.sep}example${path.sep}repo${path.sep}npm${path.sep}vize-native${path.sep}index.js`,
        ),
      ).toBe(true);
    });

    it("ignores published platform packages", () => {
      expect(
        shouldPreferWorkspaceBinding(
          `${path.sep}repo${path.sep}node_modules${path.sep}.pnpm${path.sep}@vizejs+native-darwin-arm64${path.sep}node_modules${path.sep}@vizejs${path.sep}native-darwin-arm64${path.sep}index.js`,
        ),
      ).toBe(false);
    });

    it("returns false when the fallback package cannot be resolved", () => {
      expect(shouldPreferWorkspaceBinding(null)).toBe(false);
    });
  });
}
