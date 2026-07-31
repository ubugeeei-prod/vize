import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { initProject, type InitCommand, type InitPlan } from "../src/init.ts";
import type { PromptDeps } from "../src/init/prompt.ts";

/**
 * Harness for the non-interactive `init` path.
 *
 * Every run writes to a real temporary project under `os.tmpdir()` and records
 * what it wrote, so the same helper serves both the "assert the exact bytes"
 * tests and the idempotence test, where the second run has to observe the first
 * run's output on disk.
 */

export interface RunResult {
  readonly plan: InitPlan | null;
  readonly commands: readonly InitCommand[];
  readonly output: string;
  /** Files the run wrote, relative to the project root, in write order. */
  readonly written: readonly string[];
}

export function temporaryProject(name: string): string {
  return fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), `vize-init-${name}-`)));
}

export function write(root: string, filename: string, source: string): void {
  const target = path.join(root, filename);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, source);
}

export function read(root: string, filename: string): string {
  return fs.readFileSync(path.join(root, filename), "utf8");
}

export function exists(root: string, filename: string): boolean {
  return fs.existsSync(path.join(root, filename));
}

/** Reads several files at once so a test can `deepEqual` the whole result set. */
export function readAll(root: string, filenames: readonly string[]): Record<string, string> {
  const contents: Record<string, string> = {};
  for (const filename of filenames) {
    contents[filename] = read(root, filename);
  }
  return contents;
}

/**
 * Runs `init` against a fixture project.
 *
 * The install command is simulated rather than executed: it appends the
 * requested dev dependencies to `package.json` exactly as a package manager
 * would, which is what makes the second run of the idempotence test see them.
 */
export async function runInit(
  root: string,
  args: readonly string[],
  prompt?: ScriptedPrompt,
): Promise<RunResult> {
  const commands: InitCommand[] = [];
  const written: string[] = [];
  let output = "";

  const plan = await initProject({
    root,
    args,
    stdin: prompt === undefined ? nonTtyStdin() : ttyStdin(),
    promptDeps: prompt?.deps,
    output(chunk) {
      output += chunk;
    },
    writeFile(filename, source) {
      fs.mkdirSync(path.dirname(filename), { recursive: true });
      fs.writeFileSync(filename, source);
      written.push(path.relative(root, filename));
    },
    runCommand(command) {
      commands.push(command);
      applyInstall(root, command);
    },
  });

  return { plan, commands, output, written };
}

/** A stdin stand-in that is explicitly not a TTY, matching a CI environment. */
export function nonTtyStdin(): NodeJS.ReadableStream {
  return { isTTY: false } as unknown as NodeJS.ReadableStream;
}

/** A stdin stand-in that reports itself as a TTY without being one. */
export function ttyStdin(): NodeJS.ReadableStream {
  return { isTTY: true } as unknown as NodeJS.ReadableStream;
}

export interface ScriptedPrompt {
  readonly deps: PromptDeps;
  /** Everything the checklist rendered, in order. */
  readonly transcript: () => string;
  /** The prompts the run asked for, in order. */
  readonly asked: () => readonly string[];
}

/**
 * A prompt that replays a fixed list of answers.
 *
 * Drives the real `selectFeatures` / `confirm` code rather than a stand-in for
 * it, so the checklist rendering and the toggle parsing are both covered. Once
 * the script runs out the prompt reports a closed input, matching what readline
 * does at EOF.
 */
export function scriptedPrompt(answers: readonly (string | null)[]): ScriptedPrompt {
  const asked: string[] = [];
  let transcript = "";
  let index = 0;
  const deps: PromptDeps = {
    input: ttyStdin(),
    output: {
      write(chunk: string): boolean {
        transcript += chunk;
        return true;
      },
    } as unknown as NodeJS.WritableStream,
    question(query: string): Promise<string | null> {
      asked.push(query);
      const answer = index < answers.length ? answers[index]! : null;
      index += 1;
      return Promise.resolve(answer);
    },
  };
  return { deps, transcript: () => transcript, asked: () => asked };
}

function applyInstall(root: string, command: InitCommand): void {
  const packagePath = path.join(root, "package.json");
  const manifest = JSON.parse(fs.readFileSync(packagePath, "utf8")) as {
    devDependencies?: Record<string, string>;
  };
  const devDependencies = manifest.devDependencies ?? {};
  for (const dependency of command.args.slice(2)) {
    devDependencies[dependency] = "^0.306.0";
  }
  manifest.devDependencies = devDependencies;
  fs.writeFileSync(packagePath, `${JSON.stringify(manifest, null, 2)}\n`);
}

export function manifest(
  overrides: Readonly<Record<string, unknown>> = {},
): Readonly<Record<string, unknown>> {
  return { name: "fixture", private: true, type: "module", ...overrides };
}

export function writeManifest(root: string, value: Readonly<Record<string, unknown>>): void {
  write(root, "package.json", `${JSON.stringify(value, null, 2)}\n`);
}

/** All five features on, no prompting. The flags CI would pass. */
export const ALL_FEATURES = [
  "--yes",
  "--lint",
  "--bundler",
  "--fmt",
  "--typecheck",
  "--editor",
] as const;
