import { createRequire } from "module";
import { readFileSync } from "fs";
import { loadConfig } from "./config.js";

const require = createRequire(import.meta.url);

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

function loadNative(): NativeBinding {
  const pkg = getBindingPackageName();
  try {
    return require(pkg);
  } catch (e) {
    console.error(`Failed to load native binding: ${pkg}`);
    console.error("Try reinstalling: npm install vize");
    throw e;
  }
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

  const native = loadNative();
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
// Command router
// ============================================================================

const NAPI_COMMANDS = new Set(["lint"]);

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const command = args[0];

  if (!command) {
    console.error("Usage: vize <command> [options]");
    console.error("Commands: lint");
    process.exit(1);
  }

  if (NAPI_COMMANDS.has(command)) {
    const commandArgs = args.slice(1);
    switch (command) {
      case "lint":
        await runLint(commandArgs);
        break;
    }
  } else {
    console.error(`Unknown command: ${command}`);
    console.error(
      "For commands not yet available via NAPI, install from source: cargo install vize",
    );
    process.exit(1);
  }
}

void main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
