import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

type RunReceipt = {
  output: { status: number | null; stdout: string; stderr: string };
  wallMs: number;
  cpuMs: number | null;
  maxRssKb: number | null;
};

export function writeRunReceipt(options: {
  out: string;
  tool: string;
  tag: string;
  phase: string;
  bin: string;
  args: readonly string[];
  cwd: string;
  cacheDir: string;
  timed: boolean;
  nodeOptions: string;
  extraEnv?: Readonly<Record<string, string>>;
  run: RunReceipt;
}): void {
  const rawDir = join(options.out, "raw", options.tag);
  const phaseTag = options.phase.replaceAll(" ", "-");
  mkdirSync(rawDir, { recursive: true });
  writeFileSync(join(rawDir, `${phaseTag}.stdout`), options.run.output.stdout);
  writeFileSync(join(rawDir, `${phaseTag}.stderr`), options.run.output.stderr);
  writeFileSync(
    join(rawDir, `${phaseTag}.json`),
    `${JSON.stringify(
      {
        tool: options.tool,
        phase: options.phase,
        bin: options.bin,
        args: options.args,
        cwd: options.cwd,
        status: options.run.output.status,
        wallMs: options.run.wallMs,
        cpuMs: options.run.cpuMs,
        maxRssKb: options.run.maxRssKb,
        timed: options.timed,
        env: {
          FORCE_COLOR: "0",
          NO_COLOR: "1",
          NODE_OPTIONS: options.nodeOptions,
          XDG_CACHE_HOME: options.cacheDir,
          ...options.extraEnv,
        },
      },
      null,
      1,
    )}\n`,
  );
}
