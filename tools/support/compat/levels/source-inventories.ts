// Source preflights gate every PR build. Their lanes are independent read-only
// checks, so they run together; each lane keeps its own command order, output
// is replayed lane by lane, and any failing lane fails the gate.
import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

export type Lane = readonly (readonly string[])[];

/** Node argv for each lane, in the order the action used to run them. */
export const sourceInventoryLanes: readonly Lane[] = [
  [["tools/support/compat/davinci/croquis-consumers.ts", "--check"]],
  [["tools/support/compat/davinci/consumer-migration-surfaces.ts", "--check"]],
  [["--test", "tests/tooling/davinci-storage-policy.test.ts"]],
  [
    ["tools/support/compat/davinci/generated-ledgers.ts", "--write"],
    ["tools/support/compat/davinci/generated-ledgers.ts", "--check"],
  ],
];

type LaneResult = { output: string; status: number };

function runCommand(args: readonly string[], cwd: string, output: Buffer[]): Promise<number> {
  return new Promise((done) => {
    const child = spawn(process.execPath, args, { cwd, stdio: ["ignore", "pipe", "pipe"] });
    child.stdout.on("data", (chunk: Buffer) => output.push(chunk));
    child.stderr.on("data", (chunk: Buffer) => output.push(chunk));
    child.on("error", (error) => {
      output.push(Buffer.from(`${String(error)}\n`));
      done(1);
    });
    child.on("close", (status) => done(status ?? 1));
  });
}

async function runLane(lane: Lane, cwd: string): Promise<LaneResult> {
  const output: Buffer[] = [];
  for (const args of lane) {
    output.push(Buffer.from(`$ node ${args.join(" ")}\n`));
    const status = await runCommand(args, cwd, output);
    // A later command in a lane depends on the earlier one (write, then check).
    if (status !== 0) return { output: Buffer.concat(output).toString("utf8"), status };
  }
  return { output: Buffer.concat(output).toString("utf8"), status: 0 };
}

export async function runLanes(lanes: readonly Lane[], cwd: string): Promise<LaneResult[]> {
  return Promise.all(lanes.map((lane) => runLane(lane, cwd)));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = fileURLToPath(new URL("../../../../", import.meta.url));
  const results = await runLanes(sourceInventoryLanes, root);
  for (const { output, status } of results) {
    process.stdout.write(output);
    if (status !== 0) process.stdout.write(`(exit ${status})\n`);
  }
  process.exitCode = results.some(({ status }) => status !== 0) ? 1 : 0;
}
