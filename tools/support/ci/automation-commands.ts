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

export function isProgramInvocation(source: string): boolean {
  const command = source.trim();
  if (!command || /[\r\n]/u.test(command)) return false;
  const tokens: string[] = [];
  let quote = "";
  let word = "";
  const finish = () => {
    if (word) tokens.push(word);
    word = "";
  };
  const line = command.replace(expression, "ACTIONS_VALUE");
  for (let i = 0; i < line.length; i++) {
    const ch = line[i]!;
    if (ch === "\\" && quote !== "'") {
      if (++i === line.length) return false;
      word += line[i];
    } else if (quote && ch === quote) {
      quote = "";
    } else if (!quote && (ch === '"' || ch === "'")) {
      quote = ch;
    } else if (quote !== "'" && (ch === "`" || line.startsWith("$(", i))) {
      return false;
    } else if (!quote && shellOperators.test(ch)) {
      return false;
    } else if (!quote && /\s/u.test(ch)) {
      finish();
    } else {
      word += ch;
    }
  }
  if (quote) return false;
  finish();
  const executable = tokens[0] ?? "";
  if (
    !/^[\w./-]+$/u.test(executable) ||
    executable === "ACTIONS_VALUE" ||
    shellKeywords.test(executable)
  )
    return false;
  for (const [index, token] of tokens.entries()) {
    const commandName = path.posix.basename(token);
    if (!interpreter.test(commandName)) continue;
    const shell = /^(?:ba|da|z|k)?sh$/u.test(commandName);
    const powershell = /^(?:pwsh|powershell)/iu.test(commandName);
    for (let cursor = index + 1; cursor < tokens.length; cursor++) {
      const argument = tokens[cursor]!;
      if (argument === "--") break;
      if (["deno", "bun"].includes(commandName) && argument === "eval") return false;
      if (powershell && /^-(?:c|command|ec|encodedcommand)(?::|$)/iu.test(argument)) return false;
      if (shell && /^-[a-z]*c[a-z]*$/iu.test(argument)) return false;
      if (!shell && !powershell && /^(?:--(?:eval|print)(?:=|$)|-[cep])/u.test(argument)) {
        return false;
      }
      // Flags after the entry point are arguments to that program (e.g. tsc -p).
      if (!argument.startsWith("-")) break;
      if (
        ["-r", "--require", "--import", "--loader", "--input-type", "--env-file"].includes(argument)
      )
        cursor++;
    }
  }
  return true;
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
