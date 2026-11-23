import path from "node:path";
import type { ConfigValue } from "./repository-yaml.ts";

function record(value: ConfigValue | undefined): value is Record<string, ConfigValue> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

// Actions expressions are evaluated before the shell parses the command.
const expression = /\$\{\{(?:[^}'"]|'(?:[^']|'')*'|"[^"]*"|\}(?!\}))*\}\}/gu;
const shellOperators = /[;&|<>(){}]/u;
const shellKeywords =
  /^(?:if|then|for|while|until|case|function|eval|execscript|invoke-expression|iex|trap|export)$/iu;
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

export function isProgramCommand(tokens: readonly string[]): boolean {
  const executable = tokens[0] ?? "";
  if (
    !/^[\w@./-]+$/u.test(executable) ||
    executable === "ACTIONS_VALUE" ||
    shellKeywords.test(executable)
  )
    return false;
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

export function validateShellProgram(file: string, source: string): void {
  const match = inlineProgram.exec(source.replace(/\\\r?\n/gu, " "));
  if (match)
    throw new Error(
      file + ": move embedded interpreter code into a named script: " + match[0].trim(),
    );
}

/** Paths are relative to the workflow directory or to a composite action's script directory. */
export function automationProgramReferences(command: string): string[] {
  const names =
    /(?:\$\{\{ env\.VIZE_CI_FILES \}\}|\$VIZE_CI_FILES|\$GITHUB_ACTION_PATH\/scripts)\/([.\w/-]+\.(?:sh|ps1|ts|mts|cts|cjs|mjs|js|py))(?=["'\s:]|$)/gu;
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
