import path from "node:path";
import { pathToFileURL } from "node:url";

import {
  UI_SOURCE_REGISTRY_SCHEMA_VERSION,
  createUiSourceRegistryManifest,
  getUiSourceFamilyInfo,
  listUiSourceFamilies,
  searchUiSourceFamilies,
  type UiSourceFamilyManifest,
  type UiSourceFamilySummary,
  type UiSourceRegistryOutputFormat,
  type UiSourceSearchResult,
} from "../src/source-registry.ts";

interface WritableOutput {
  write(chunk: string): void;
}

interface CliIo {
  readonly stdout: WritableOutput;
  readonly stderr: WritableOutput;
}

interface ParsedArgs {
  readonly format: UiSourceRegistryOutputFormat;
  readonly help: boolean;
  readonly positional: readonly string[];
  readonly error: string | null;
}

type CliRecord =
  | {
      readonly schemaVersion: typeof UI_SOURCE_REGISTRY_SCHEMA_VERSION;
      readonly command: "list";
      readonly family: UiSourceFamilySummary;
    }
  | {
      readonly schemaVersion: typeof UI_SOURCE_REGISTRY_SCHEMA_VERSION;
      readonly command: "search";
      readonly query: string;
      readonly match: UiSourceSearchResult;
    }
  | {
      readonly schemaVersion: typeof UI_SOURCE_REGISTRY_SCHEMA_VERSION;
      readonly command: "info";
      readonly family: UiSourceFamilyManifest;
    };

const usage = `Usage:
  node scripts/source-registry.ts list [--format json|jsonl]
  node scripts/source-registry.ts search <query> [--format json|jsonl]
  node scripts/source-registry.ts info <name-or-alias> [--format json|jsonl]

Commands are read-only. Source install, update, diff, cache, and rollback commands are tracked by issue #4896 but are not implemented in this foundation slice.
`;

const mutatingCommands = new Set([
  "init",
  "add",
  "add-many",
  "remove",
  "diff",
  "update",
  "doctor",
  "audit",
]);

function parseFormat(value: string): UiSourceRegistryOutputFormat | null {
  return value === "json" || value === "jsonl" ? value : null;
}

function parseArgs(args: readonly string[]): ParsedArgs {
  let format: UiSourceRegistryOutputFormat = "json";
  let help = false;
  const positional: string[] = [];

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg == null) continue;

    if (arg === "--help" || arg === "-h") {
      help = true;
      continue;
    }

    if (arg === "--json") {
      format = "json";
      continue;
    }

    if (arg === "--jsonl") {
      format = "jsonl";
      continue;
    }

    if (arg === "--format") {
      const value = args[index + 1];
      if (value == null) {
        return { format, help, positional, error: "--format requires json or jsonl" };
      }

      const parsedFormat = parseFormat(value);
      if (parsedFormat == null) {
        return { format, help, positional, error: `Unsupported output format "${value}"` };
      }

      format = parsedFormat;
      index += 1;
      continue;
    }

    if (arg.startsWith("--format=")) {
      const parsedFormat = parseFormat(arg.slice("--format=".length));
      if (parsedFormat == null) {
        return { format, help, positional, error: `Unsupported output format "${arg}"` };
      }

      format = parsedFormat;
      continue;
    }

    if (arg.startsWith("-")) {
      return { format, help, positional, error: `Unsupported option "${arg}"` };
    }

    positional.push(arg);
  }

  return { format, help, positional, error: null };
}

function writeJson(output: WritableOutput, value: unknown): void {
  output.write(`${JSON.stringify(value, null, 2)}\n`);
}

function writeJsonl(output: WritableOutput, records: readonly CliRecord[]): void {
  for (const record of records) output.write(`${JSON.stringify(record)}\n`);
}

function writeError(io: CliIo, message: string): number {
  io.stderr.write(`${message}\n\n${usage}`);
  return 1;
}

function queryFromArgs(args: readonly string[]): string {
  return args.join(" ").trim();
}

export function runUiSourceRegistryCli(
  args: readonly string[],
  io: CliIo = { stdout: process.stdout, stderr: process.stderr },
): number {
  const parsed = parseArgs(args);
  if (parsed.help) {
    io.stdout.write(usage);
    return 0;
  }

  if (parsed.error != null) return writeError(io, parsed.error);

  const command = parsed.positional[0];
  if (command == null) {
    io.stdout.write(usage);
    return 0;
  }

  if (mutatingCommands.has(command)) {
    return writeError(
      io,
      `Command "${command}" is read-only in this foundation slice; install workflows remain tracked by issue #4896.`,
    );
  }

  const manifest = createUiSourceRegistryManifest();

  if (command === "list") {
    if (parsed.positional.length !== 1) {
      return writeError(io, "The list command does not accept a query");
    }

    const families = listUiSourceFamilies(manifest);
    if (parsed.format === "jsonl") {
      writeJsonl(
        io.stdout,
        families.map((family) => ({
          schemaVersion: UI_SOURCE_REGISTRY_SCHEMA_VERSION,
          command,
          family,
        })),
      );
    } else {
      writeJson(io.stdout, {
        schemaVersion: UI_SOURCE_REGISTRY_SCHEMA_VERSION,
        command,
        familyCount: families.length,
        families,
      });
    }
    return 0;
  }

  if (command === "search") {
    const query = queryFromArgs(parsed.positional.slice(1));
    if (query.length === 0) return writeError(io, "The search command requires a query");

    const matches = searchUiSourceFamilies(query, manifest);
    if (parsed.format === "jsonl") {
      writeJsonl(
        io.stdout,
        matches.map((match) => ({
          schemaVersion: UI_SOURCE_REGISTRY_SCHEMA_VERSION,
          command,
          query,
          match,
        })),
      );
    } else {
      writeJson(io.stdout, {
        schemaVersion: UI_SOURCE_REGISTRY_SCHEMA_VERSION,
        command,
        query,
        matchCount: matches.length,
        matches,
      });
    }
    return 0;
  }

  if (command === "info") {
    const target = queryFromArgs(parsed.positional.slice(1));
    if (target.length === 0) return writeError(io, "The info command requires a family name");

    const family = getUiSourceFamilyInfo(target, manifest);
    if (family == null) return writeError(io, `Unknown UI source family "${target}"`);

    if (parsed.format === "jsonl") {
      writeJsonl(io.stdout, [
        {
          schemaVersion: UI_SOURCE_REGISTRY_SCHEMA_VERSION,
          command,
          family,
        },
      ]);
    } else {
      writeJson(io.stdout, {
        schemaVersion: UI_SOURCE_REGISTRY_SCHEMA_VERSION,
        command,
        family,
      });
    }
    return 0;
  }

  return writeError(io, `Unsupported command "${command}"`);
}

const isMain =
  process.argv[1] != null && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url;

if (isMain) {
  process.exitCode = runUiSourceRegistryCli(process.argv.slice(2));
}
