#!/usr/bin/env node
/**
 * Musea CLI
 *
 * Usage:
 *   musea-vrt [command] [options]
 *
 * Commands:
 *   (default)       Run VRT tests
 *   approve [pat]   Approve failed snapshots (optionally filtered by pattern)
 *   clean           Remove orphaned snapshots
 *
 * Options:
 *   -u, --update     Update baseline snapshots
 *   -c, --config     Path to vite config (default: vite.config.ts)
 *   -o, --output     Output directory for reports (default: .vize)
 *   -t, --threshold  Diff threshold percentage (default: 0.1)
 *   --json           Output JSON report instead of HTML
 *   --ci             CI mode - exit with non-zero code on failures
 *   --a11y           Run accessibility audits alongside VRT
 *   -h, --help       Show help
 */

import type { ArtFileInfo } from "../types/index.js";
import { scanArtFiles, parseArtFile } from "./utils.js";
import { runVrt, runApprove, runClean, runGenerate } from "./commands.js";

type Command = "run" | "approve" | "clean" | "generate";

export interface CliOptions {
  command: Command;
  update: boolean;
  config: string;
  output: string;
  threshold: number;
  json: boolean;
  ci: boolean;
  a11y: boolean;
  help: boolean;
  baseUrl: string;
  pattern?: string;
  componentPath?: string;
}

function parseArgs(args: string[]): CliOptions {
  const options: CliOptions = {
    command: "run",
    update: false,
    config: "vite.config.ts",
    output: ".vize",
    threshold: 0.1,
    json: false,
    ci: false,
    a11y: false,
    help: false,
    baseUrl: "http://localhost:5173",
  };

  let i = 0;

  // Check for subcommand as first arg
  if (args.length > 0 && !args[0].startsWith("-")) {
    const sub = args[0];
    if (sub === "approve") {
      options.command = "approve";
      i = 1;
      // Optional pattern argument after approve
      if (args.length > 1 && !args[1].startsWith("-")) {
        options.pattern = args[1];
        i = 2;
      }
    } else if (sub === "clean") {
      options.command = "clean";
      i = 1;
    } else if (sub === "generate") {
      options.command = "generate";
      i = 1;
      // Required component path argument
      if (args.length > 1 && !args[1].startsWith("-")) {
        options.componentPath = args[1];
        i = 2;
      }
    }
  }

  for (; i < args.length; i++) {
    const arg = args[i];
    switch (arg) {
      case "-u":
      case "--update":
        options.update = true;
        break;
      case "-c":
      case "--config":
        options.config = args[++i] || "vite.config.ts";
        break;
      case "-o":
      case "--output":
        options.output = args[++i] || ".vize";
        break;
      case "-t":
      case "--threshold":
        options.threshold = parseFloat(args[++i]) || 0.1;
        break;
      case "--json":
        options.json = true;
        break;
      case "--ci":
        options.ci = true;
        break;
      case "--a11y":
        options.a11y = true;
        break;
      case "-b":
      case "--base-url":
        options.baseUrl = args[++i] || "http://localhost:5173";
        break;
      case "-h":
      case "--help":
        options.help = true;
        break;
    }
  }

  return options;
}

function printHelp(): void {
  console.log(`
Musea VRT - Visual Regression Testing for Component Gallery

Usage:
  musea-vrt [command] [options]

Commands:
  (default)             Run VRT tests
  approve [pattern]     Approve failed snapshots and update baselines
                        Optional pattern filters which snapshots to approve
  clean                 Remove orphaned snapshots (no matching art/variant)
  generate <component>  Auto-generate .art.vue from a Vue component

Options:
  -u, --update         Update baseline snapshots with current screenshots
  -c, --config <path>  Path to vite config file (default: vite.config.ts)
  -o, --output <dir>   Output directory for reports (default: .vize)
  -t, --threshold <n>  Diff threshold percentage (default: 0.1)
  -b, --base-url <url> Base URL for dev server (default: http://localhost:5173)
  --json               Output JSON report instead of HTML
  --ci                 CI mode - exit with non-zero code on failures
  --a11y               Run accessibility audits alongside VRT
  -h, --help           Show this help message

Examples:
  # Run VRT tests
  musea-vrt

  # Update baseline snapshots
  musea-vrt -u

  # Run with custom threshold
  musea-vrt -t 0.5

  # CI mode with JSON output
  musea-vrt --ci --json

  # Run with accessibility audits
  musea-vrt --a11y

  # Approve all failed snapshots
  musea-vrt approve

  # Approve specific snapshots by pattern
  musea-vrt approve "Button/*"

  # Clean orphaned snapshots
  musea-vrt clean

  # Auto-generate .art.vue from component
  musea-vrt generate src/components/Button.vue

  # Custom base URL
  musea-vrt -b http://localhost:3000
`);
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const options = parseArgs(args);

  if (options.help) {
    printHelp();
    process.exit(0);
  }

  const cwd = process.cwd();

  console.log("\n  Musea VRT");
  console.log("  =========\n");

  // Handle generate command early (doesn't need art file scanning)
  if (options.command === "generate") {
    try {
      await runGenerate(options);
    } catch (error) {
      console.error("\n  Error:", error);
      process.exit(1);
    }
    return;
  }

  // Scan for art files
  console.log("  Scanning for art files...");
  const artFilePaths = await scanArtFiles(cwd);

  if (artFilePaths.length === 0) {
    console.log("  No art files found.\n");
    process.exit(0);
  }

  console.log(`  Found ${artFilePaths.length} art file(s)\n`);

  // Parse art files
  const artFiles: ArtFileInfo[] = [];
  for (const filePath of artFilePaths) {
    const art = await parseArtFile(filePath);
    if (art) {
      artFiles.push(art);
    }
  }

  try {
    switch (options.command) {
      case "run":
        await runVrt(options, artFiles);
        break;
      case "approve":
        await runApprove(options, artFiles);
        break;
      case "clean":
        await runClean(options, artFiles);
        break;
      case "generate":
        // Handled above before art file scanning
        break;
    }
  } catch (error) {
    console.error("\n  Error:", error);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
