// Capture full source-built CLI facts; never use a guessed or partial oracle.
import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

const [binary, directory] = process.argv.slice(2);
if (!binary || !directory) throw new Error("usage: capture-native-script-cli.ts BINARY DIRECTORY");
const executable = path.resolve(binary);
mkdirSync(directory, { recursive: true });
const cases = [
  {
    name: "js-module",
    language: "js",
    input: "/* α */ import { ref } from 'vue';\r\nexport const value = ref(0);",
  },
  {
    name: "ts-module",
    language: "ts",
    input: "interface Props { value: number } export const value: Props = { value: 1 };",
  },
  {
    name: "jsx-module",
    language: "jsx",
    input: "const View = () => <section>{values.map(value => <span>{value}</span>)}</section>;",
  },
  {
    name: "tsx-module",
    language: "tsx",
    input:
      "type Props = { value: number }; export const View = ({ value }: Props) => <span>{value}</span>;",
  },
  {
    name: "tsx-script-hole",
    language: "tsx",
    goal: "script",
    input: "/* α */ const value: number = ;\r\n// β",
  },
];
function capture(name: string, args: string[], expected: number): void {
  const result = spawnSync(executable, args, { maxBuffer: 1024 * 1024 });
  if (result.error) throw result.error;
  writeFileSync(path.join(directory, `${name}.stdout`), result.stdout);
  writeFileSync(path.join(directory, `${name}.stderr`), result.stderr);
  writeFileSync(
    path.join(directory, `${name}.status.json`),
    JSON.stringify({ args, status: result.status, signal: result.signal }) + "\n",
  );
  if (result.status !== expected || result.signal !== null)
    throw new Error(
      `${name}: expected exit ${expected}, received ${result.status}/${result.signal}`,
    );
}
for (const fixture of cases) {
  const input = path.join(directory, `${fixture.name}.input`);
  writeFileSync(input, fixture.input);
  const args = ["dump", "--level", "l1", "--script", fixture.language];
  if (fixture.goal) args.push("--script-goal", fixture.goal);
  args.push("--roundtrip", input);
  capture(fixture.name, args, 0);
}
capture("wrong-level", ["dump", "--level", "l2", "--script", "ts", "--roundtrip", "missing"], 2);
capture("missing-roundtrip", ["dump", "--script", "ts"], 2);
capture(
  "missing-script",
  ["dump", "--level", "l1", "--roundtrip", "missing", "--script-goal", "script"],
  2,
);
