/**
 * The Format surface. Each variant formats the same unformatted copy of the
 * input in one work dir it keeps for the whole surface, so the CLI's
 * `vite.config` evaluation cache (`node_modules/.vize/config-cache`) is warm
 * after the first warmup, as it is for lint in the fixed input dir. The input
 * files are restored before every run, outside the timed command.
 */

import { copyFileSync, existsSync } from "node:fs";
import { join, resolve } from "node:path";

type Sample = number | { ms: number; cpuMs?: number | null };
type Variant = { id: string; label: string; files: number; measure: () => Sample };
type RunCommand = (
  binary: string,
  args: string[],
  options: { cwd: string; allowNonZeroExit: boolean; env?: Record<string, string> },
) => Sample;

export type FormatSurfaceDeps = {
  workRoot: string;
  runCommand: RunCommand;
  resolveWorkspaceBin: (name: string) => string;
  copySelectedFiles: (
    inputDir: string,
    outputDir: string,
    files: string[],
    extra: string[],
  ) => void;
  totalFileBytes: (inputDir: string, files: string[]) => number;
  measureVariants: (variants: Variant[], options: unknown) => Promise<unknown[]>;
  createSurface: (surface: Record<string, unknown>) => unknown;
};

/** A dir per label, created once; later calls restore the unformatted input. */
export function warmFormatDirs(
  inputDir: string,
  files: string[],
  { workRoot, copySelectedFiles }: Pick<FormatSurfaceDeps, "workRoot" | "copySelectedFiles">,
): (label: string) => string {
  const dirs = new Map<string, string>();
  return (label) => {
    const existing = dirs.get(label);
    if (existing) {
      for (const file of files) copyFileSync(join(inputDir, file), join(existing, file));
      return existing;
    }
    const dir = join(workRoot, "fmt", label);
    copySelectedFiles(inputDir, dir, files, ["vite.config.mjs"]);
    dirs.set(label, dir);
    return dir;
  };
}

export async function measureFormat(
  inputDir: string,
  files: string[],
  options: { vizeBin: string },
  deps: FormatSurfaceDeps,
): Promise<unknown> {
  const prettierBin = deps.resolveWorkspaceBin("prettier");
  const vizeBin = resolve(options.vizeBin);
  if (!existsSync(vizeBin)) {
    throw new Error(`Vize CLI not found: ${vizeBin}`);
  }
  const workDir = warmFormatDirs(inputDir, files, deps);
  const vizeFmt = (label: string, env?: Record<string, string>) => () =>
    deps.runCommand(vizeBin, ["fmt", "--write", "*.vue"], {
      cwd: workDir(label),
      allowNonZeroExit: false,
      ...(env ? { env } : {}),
    });
  const variants: Variant[] = [
    {
      id: "prettier-cli",
      label: "Prettier CLI",
      files: files.length,
      measure: () =>
        deps.runCommand(prettierBin, ["--write", "*.vue", "--log-level", "error"], {
          cwd: workDir("prettier"),
          allowNonZeroExit: false,
        }),
    },
    {
      id: "vize-fmt-1t",
      label: "Vize fmt (1T)",
      files: files.length,
      measure: vizeFmt("vize-1t", { RAYON_NUM_THREADS: "1" }),
    },
    {
      id: "vize-fmt-max",
      label: "Vize fmt (max)",
      files: files.length,
      measure: vizeFmt("vize-max"),
    },
  ];
  return deps.createSurface({
    id: "fmt",
    label: "Format",
    files: files.length,
    bytes: deps.totalFileBytes(inputDir, files),
    variants: await deps.measureVariants(variants, options),
    baselineId: "prettier-cli",
    vizeSingleId: "vize-fmt-1t",
    vizeMaxId: "vize-fmt-max",
  });
}
