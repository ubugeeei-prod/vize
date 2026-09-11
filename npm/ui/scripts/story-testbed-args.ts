import {
  uiStoryTestbedSurfaces,
  type UiStoryTestbedSurface,
} from "../src/story-testbed/story-testbed.ts";

export type StoryTestbedOutputFormat = "json" | "jsonl";

export interface ParsedStoryTestbedArgs {
  readonly format: StoryTestbedOutputFormat;
  readonly help: boolean;
  readonly positional: readonly string[];
  readonly surface: UiStoryTestbedSurface | null;
  readonly error: string | null;
}

function parseFormat(value: string): StoryTestbedOutputFormat | null {
  return value === "json" || value === "jsonl" ? value : null;
}

export function parseStoryTestbedSurface(value: string): UiStoryTestbedSurface | null {
  return (uiStoryTestbedSurfaces as readonly string[]).includes(value)
    ? (value as UiStoryTestbedSurface)
    : null;
}

export function parseStoryTestbedArgs(args: readonly string[]): ParsedStoryTestbedArgs {
  let format: StoryTestbedOutputFormat = "json";
  let help = false;
  let surface: UiStoryTestbedSurface | null = null;
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
      if (value == null)
        return { format, help, positional, surface, error: "--format requires json or jsonl" };

      const parsedFormat = parseFormat(value);
      if (parsedFormat == null)
        return { format, help, positional, surface, error: `Unsupported output format "${value}"` };

      format = parsedFormat;
      index += 1;
      continue;
    }

    if (arg.startsWith("--format=")) {
      const value = arg.slice("--format=".length);
      const parsedFormat = parseFormat(value);
      if (parsedFormat == null)
        return { format, help, positional, surface, error: `Unsupported output format "${value}"` };

      format = parsedFormat;
      continue;
    }

    if (arg === "--surface") {
      const value = args[index + 1];
      if (value == null) {
        return {
          format,
          help,
          positional,
          surface,
          error: "--surface requires a story-testbed surface",
        };
      }

      const parsedSurface = parseStoryTestbedSurface(value);
      if (parsedSurface == null) {
        return {
          format,
          help,
          positional,
          surface,
          error: `Unsupported story-testbed surface "${value}"`,
        };
      }

      surface = parsedSurface;
      index += 1;
      continue;
    }

    if (arg.startsWith("--surface=")) {
      const value = arg.slice("--surface=".length);
      const parsedSurface = parseStoryTestbedSurface(value);
      if (parsedSurface == null) {
        return {
          format,
          help,
          positional,
          surface,
          error: `Unsupported story-testbed surface "${value}"`,
        };
      }

      surface = parsedSurface;
      continue;
    }

    if (arg.startsWith("-")) {
      return { format, help, positional, surface, error: `Unsupported option "${arg}"` };
    }

    positional.push(arg);
  }

  return { format, help, positional, surface, error: null };
}
