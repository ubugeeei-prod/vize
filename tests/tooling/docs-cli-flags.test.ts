// Documented CLI flags must exist, and every user-facing flag must be documented.
// The ground truth is the committed `--help` snapshots that `crates/vize` keeps
// in sync with clap, so this check needs no Rust build.
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";

const root = resolve(import.meta.dirname, "../..");
const read = (path: string) => readFileSync(resolve(root, path), "utf8");

const commands = [
  "build",
  "fmt",
  "lint",
  "check",
  "doctor",
  "inspector",
  "clean",
  "ready",
] as const;
type Command = (typeof commands)[number];

/** `cli.md` section heading for each command. */
const sections: Record<Command, string> = {
  build: "Build",
  fmt: "Format",
  lint: "Lint",
  check: "Check",
  doctor: "Doctor",
  inspector: "Inspector",
  clean: "Clean",
  ready: "Ready",
};

/** Flags every command shares, documented once under Global Options. */
const shared = new Set(["--help", "--config", "--no-config"]);

/** Contributor-only flags that the user CLI reference intentionally omits. */
const internal: Partial<Record<Command, string[]>> = {
  build: ["--dump-dir", "--dump-after-change", "--profile-json", "--slow-threshold"],
  fmt: ["--slow-threshold"],
  lint: ["--profile-json"],
  check: ["--profile-json"],
};

/** Pages that document a command in more depth than the CLI reference. */
const companions: Partial<Record<Command, string>> = {
  doctor: "docs/content/guide/doctor.md",
};

/** Hidden clap aliases accepted by the binary but absent from `--help`. */
const aliases: Partial<Record<Command, string[]>> = {
  build: ["--dts"],
};

function helpFlags(command: Command): Set<string> {
  const help = read(`crates/vize/src/snapshots/vize__cli__tests__cli_${command}_help.snap`);
  const flags = [...help.matchAll(/^\s+(?:-[A-Za-z], )?(--[a-z0-9-]+)/gm)].map((match) => match[1]);
  assert.ok(flags.length > 0, `${command}: help snapshot lists options`);
  return new Set([...flags, ...(aliases[command] ?? [])]);
}

function section(markdown: string, heading: string): string {
  const start = markdown.search(new RegExp(`^## ${heading}$`, "m"));
  assert.ok(start >= 0, `cli.md has a "## ${heading}" section`);
  const rest = markdown.slice(start + 3);
  const end = rest.search(/^## /m);
  return end < 0 ? rest : rest.slice(0, end);
}

const flagsIn = (text: string) =>
  new Set([...text.matchAll(/(?<![\w-])(--[a-z][a-z0-9-]*)/g)].map((match) => match[1]));

/** `vize <command> ...` invocations in shell fences and package.json scripts. */
function invocations(markdown: string): [Command, string][] {
  const found: [Command, string][] = [];
  const pattern = new RegExp(`\\bvize (${commands.join("|")})\\b([^\\n"\`]*)`, "g");
  for (const fence of markdown.matchAll(/^```(?:bash|sh|shell|json|jsonc)\n([\s\S]*?)^```/gm)) {
    for (const match of fence[1].matchAll(pattern)) found.push([match[1] as Command, match[2]]);
  }
  return found;
}

test("CLI reference options exist in the current help output", () => {
  const cli = read("docs/content/guide/cli.md");
  for (const command of commands) {
    const known = helpFlags(command);
    for (const flag of flagsIn(section(cli, sections[command]))) {
      // Inline prose may mention another command's flag, e.g. `fmt --write` under Ready.
      if (commands.some((other) => other !== command && helpFlags(other).has(flag))) continue;
      assert.ok(known.has(flag), `vize ${command} ${flag} is documented but does not exist`);
    }
  }
});

test("every user-facing option appears in the CLI reference", () => {
  const cli = read("docs/content/guide/cli.md");
  for (const command of commands) {
    const companion = companions[command];
    const documented = flagsIn(
      section(cli, sections[command]) + (companion ? read(companion) : ""),
    );
    const skipped = new Set([...shared, ...(internal[command] ?? [])]);
    for (const flag of helpFlags(command)) {
      if (skipped.has(flag)) continue;
      assert.ok(documented.has(flag), `vize ${command} ${flag} is missing from guide/cli.md`);
    }
  }
});

test("guide examples only pass options the command accepts", () => {
  const directories = ["docs/content/guide", "docs/content/integrations", "docs/content/ja/guide"];
  let checked = 0;
  for (const directory of directories) {
    for (const name of readdirSync(resolve(root, directory))) {
      if (!name.endsWith(".md")) continue;
      for (const [command, args] of invocations(read(`${directory}/${name}`))) {
        const known = helpFlags(command);
        for (const flag of flagsIn(args)) {
          assert.ok(known.has(flag), `${directory}/${name}: vize ${command} has no ${flag}`);
          checked += 1;
        }
      }
    }
  }
  assert.ok(checked > 50, `examples were found (${checked})`);
});
