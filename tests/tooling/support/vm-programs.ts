import { readFileSync } from "node:fs";
import { Script, type Context } from "node:vm";

export function vmProgramSource(name: string): string {
  return readFileSync(new URL(`../programs/vm/${name}`, import.meta.url), "utf8");
}

export function runVmProgram(name: string, context: Context) {
  return new Script(vmProgramSource(name), { filename: name }).runInContext(context);
}
