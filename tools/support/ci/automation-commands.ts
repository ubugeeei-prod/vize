import path from "node:path";
import type { ConfigValue } from "./repository-yaml.ts";

function record(value: ConfigValue | undefined): value is Record<string, ConfigValue> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

// Actions expressions are evaluated before the shell parses the command.
const expression = /\$\{\{(?:[^}'"]|'(?:[^']|'')*'|"[^"]*"|\}(?!\}))*\}\}/gu;
const shellOperators = /[;&|<>(){}]/u;
const shellKeywords =
  /^(?:if|then|for|while|until|case|function|test|true|false|cd|echo|printf|eval|execscript|invoke-expression|iex|trap|exit|return|break|continue|shift|unset|export)$/iu;
const interpreter =
  /^(?:node(?:js)?|bun|deno|python(?:\d+(?:\.\d+)*)?|ruby|perl|(?:ba|da|z|k)?sh|pwsh|powershell)(?:\.exe)?$/iu;

export function invocationWords(source: string): string[] | undefined {
  const command = source.trim();
  if (!command || /[\r\n]/u.test(command)) return undefined;
  const tokens: string[] = [];
  let quote = "";
  let word = "";
  let started = false;
  const finish = () => {
    if (started) tokens.push(word);
    started = false;
    word = "";
  };
  const line = command.replace(expression, "ACTIONS_VALUE");
  for (let i = 0; i < line.length; i++) {
    const ch = line[i]!;
    if (ch === "\\" && quote !== "'") {
      if (++i === line.length) return undefined;
      started = true;
      word += line[i];
    } else if (quote && ch === quote) {
      quote = "";
    } else if (!quote && (ch === '"' || ch === "'")) {
      started = true;
      quote = ch;
    } else if (quote !== "'" && (ch === "`" || line.startsWith("$(", i))) {
      return undefined;
    } else if (!quote && shellOperators.test(ch)) {
      return undefined;
    } else if (!quote && /\s/u.test(ch)) {
      finish();
    } else {
      started = true;
      word += ch;
    }
  }
  if (quote) return undefined;
  finish();
  return tokens;
}

export function isProgramInvocation(source: string): boolean {
  const tokens = invocationWords(source);
  if (!tokens) return false;
  return isProgramCommand(tokens);
}

const textFilter = /^(?:awk|gawk|mawk|jq|sed)$/u;
const namedFilterOption = /^(?:-[A-Za-z]*f|--(?:file|from-file)(?:=.+)?)$/u;
const informationalOption = /^(?:--version|--help|-V)$/u;

export function isProgramCommand(tokens: readonly string[]): boolean {
  const executable = tokens[0] ?? "";
  if (
    !/^[\w@./-]+$/u.test(executable) ||
    executable === "ACTIONS_VALUE" ||
    shellKeywords.test(executable)
  )
    return false;
  const name = path.win32
    .basename(executable)
    .replace(/\.exe$/iu, "")
    .toLowerCase();
  if (name === "gh" && tokens.some((word) => /^(?:--jq(?:=|$)|-q$)/u.test(word))) return false;
  if (textFilter.test(name)) {
    return tokens
      .slice(1)
      .some((word) => namedFilterOption.test(word) || informationalOption.test(word));
  }
  for (const [index, token] of tokens.entries()) {
    const commandName = path.win32
      .basename(token)
      .replace(/\.exe$/iu, "")
      .toLowerCase();
    if (!interpreter.test(commandName)) continue;
    const shell = /^(?:ba|da|z|k)?sh$/u.test(commandName);
    const powershell = /^(?:pwsh|powershell)/iu.test(commandName);
    for (let cursor = index + 1; cursor < tokens.length; cursor++) {
      const argument = tokens[cursor]!;
      if (/^(?:--(?:import|loader|experimental-loader)=)?data:/iu.test(argument)) return false;
      if (argument === "--") break;
      if (argument === "-" || (shell && /^-[a-z]*s[a-z]*$/iu.test(argument))) return false;
      if (["deno", "bun"].includes(commandName) && argument === "eval") return false;
      if (powershell && /^-(?:c|command|ec|encodedcommand)(?::|$)/iu.test(argument)) return false;
      if (shell && /^-[a-z]*c[a-z]*$/iu.test(argument)) return false;
      if (!shell && !powershell && /^(?:--(?:eval|print)(?:=|$)|-[cep])/u.test(argument)) {
        return false;
      }
      // Flags after the entry point are arguments to that program (e.g. tsc -p).
      if (!argument.startsWith("-")) break;
      if (
        [
          "-r",
          "--require",
          "--import",
          "--loader",
          "--experimental-loader",
          "--input-type",
          "--env-file",
        ].includes(argument)
      ) {
        if (/^data:/iu.test(tokens[cursor + 1] ?? "")) return false;
        cursor++;
      }
    }
  }
  return true;
}

// Look only at interpreter arguments, before any named entrypoint. Commands such as
// "node tools/check.ts -p file" pass -p to the program, not to Node.
const inlineProgram =
  /(?:^|[\s(/;&|])(?:(?:node(?:js)?|bun|deno|ruby|perl|python(?:\d+(?:\.\d+)*)?)\s+(?:(?:--(?:input-type|conditions|env-file|import|require|loader)(?:=|\s+)\S+|--[a-z][\w-]*)\s+)*(?:-(?:[ep]|c)(?:\s|=|["']|$)|--(?:eval|print)(?:\s|=)|(?:-\s+)?<<)|(?:ba|da|z|k)?sh\s+(?:-[a-z]+\s+)*-[a-z]*c[a-z]*(?:\s|$)|(?:pwsh|powershell)(?:\.exe)?\s+(?:-\w+\s+)*-(?:c|command|ec|encodedcommand)(?:\s|:)|(?:deno|bun)\s+eval\s|\beval\s+|\bInvoke-Expression\s+)/imu;

// These tools take executable filter text in the same argument slot as a file-based program.
const shellTextFilter = /(?:^|[;&|(])\s*(?:sudo\s+)?(awk|gawk|mawk|jq|sed)\s+([^;&|)\r\n]*)/gmu;

export function validateShellProgram(file: string, source: string): void {
  const joined = source.replace(/\\\r?\n/gu, " ");
  const match = inlineProgram.exec(joined);
  if (match)
    throw new Error(
      file + ": move embedded interpreter code into a named script: " + match[0].trim(),
    );
  if (file.startsWith(".github/") || file.startsWith("tools/")) {
    const githubQuery =
      /(?:^|[;&|(])\s*gh(?:\.exe)?\s+[^\r\n]*?\s(?:--jq[=\t ]+|-q[\t ]+)([^\r\n]*)/gmu;
    for (const [, query] of joined.matchAll(githubQuery)) {
      if (!/^"\$\(\s*<\s*"[^"]+\.jq"\s*\)"/u.test(query!)) {
        throw new Error(file + ": move the GitHub CLI query into a named script");
      }
    }
    for (const [, command, args] of joined.matchAll(shellTextFilter)) {
      const options = args!.split(/\s+/u);
      if (!options.some((word) => namedFilterOption.test(word) || informationalOption.test(word))) {
        throw new Error(file + ": move the " + command + " filter into a named script");
      }
    }
  }
}

/** Paths are relative to the workflow directory or to a composite action's script directory. */
export function automationProgramReferences(command: string): string[] {
  const names =
    /(?:\$\{\{ env\.VIZE_CI_FILES \}\}|\$VIZE_CI_FILES|\$GITHUB_ACTION_PATH\/scripts)\/([.\w/-]+\.(?:sh|ps1|ts|mts|cts|cjs|mjs|js|py|awk|jq|sed))(?=["'\s:]|$)/gu;
  return Array.from(command.matchAll(names), ([, name]) => name!);
}

export function validateAutomationSteps(file: string, document: ConfigValue): void {
  if (!record(document)) throw new Error(`Invalid automation document: ${file}`);
  const jobs = record(document.jobs) ? document.jobs : { action: document.runs };
  for (const [job, data] of Object.entries(jobs)) {
    if (!record(data) || !Array.isArray(data.steps)) continue;
    for (const step of data.steps) {
      if (!record(step)) continue;
      const embedded =
        (typeof step.uses === "string" && /^actions\/github-script(?:@|$)/iu.test(step.uses)) ||
        (record(step.with) && typeof step.with.script === "string");
      if (embedded || (typeof step.run === "string" && !isProgramInvocation(step.run))) {
        const name =
          typeof step.name === "string"
            ? step.name
            : typeof step.id === "string"
              ? step.id
              : "unnamed step";
        throw new Error(`${file} / ${job} / ${name}: move inline code into a named script`);
      }
    }
  }
}

// A step command longer than this belongs in a named program, where it can wrap.
export const maxInlineCommandLength = 160;
// Longer conditions belong in a planning step whose output the job reads.
export const maxConditionLength = 200;
const testFile = /\.test\.[cm]?[jt]s$/u;

function stepName(step: Record<string, ConfigValue>): string {
  if (typeof step.name === "string") return step.name;
  return typeof step.id === "string" ? step.id : "unnamed step";
}

/** Inline commands and conditions that are too long to review in a workflow file. */
export function automationSizeViolations(file: string, document: ConfigValue): string[] {
  if (!record(document)) return [];
  const violations: string[] = [];
  const jobs = record(document.jobs) ? document.jobs : { action: document.runs };
  for (const [job, data] of Object.entries(jobs)) {
    if (!record(data)) continue;
    if (typeof data.if === "string" && data.if.length > maxConditionLength)
      violations.push(`${file} / ${job}: condition longer than ${maxConditionLength} characters`);
    if (!Array.isArray(data.steps)) continue;
    for (const step of data.steps) {
      if (!record(step)) continue;
      const where = `${file} / ${job} / ${stepName(step)}`;
      if (typeof step.if === "string" && step.if.length > maxConditionLength)
        violations.push(`${where}: condition longer than ${maxConditionLength} characters`);
      if (typeof step.run !== "string") continue;
      if (step.run.length > maxInlineCommandLength)
        violations.push(`${where}: command longer than ${maxInlineCommandLength} characters`);
      const words = invocationWords(step.run) ?? [];
      if (words[0] === "node" && words.includes("--test"))
        if (words.filter((word) => testFile.test(word)).length > 2)
          violations.push(`${where}: more than two inline test files`);
    }
  }
  return violations;
}
