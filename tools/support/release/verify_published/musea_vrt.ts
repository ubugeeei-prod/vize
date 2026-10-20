// Post-publish check of Musea visual regression testing: install the
// published @vizejs/vite-plugin-musea into a fresh Vite project, serve its
// gallery and drive `musea-vrt` in Chromium through new, match, diff, update
// and repeat.
import { spawn, type ChildProcess } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { collectOutput, run, runMain, writeStderr } from "../command_io.ts";
import { cliArgs } from "../command_runtime.ts";
import { install, runSteps, write, type Step } from "./config_free.ts";

const BLUE = "rgb(0, 128, 255)";
const RED = "rgb(255, 64, 0)";
const swatch = (color: string) => `<script setup lang="ts">
defineProps<{ label: string }>()
</script>

<template>
  <div class="swatch">{{ label }}</div>
</template>

<style scoped>
.swatch { width: 120px; height: 60px; background: ${color}; color: white; }
</style>
`;
const ART = `<script setup lang="ts">
defineArt("./Swatch.vue", { title: "Swatch" });
</script>

<art>
  <variant name="Default" default>
    <Swatch label="Swatch" />
  </variant>
</art>
`;
const VITE_CONFIG = `import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["src/**/*.art.vue"],
      vrt: { viewports: [{ width: 320, height: 200, name: "small" }] },
    }),
  ],
});
`;
const BASELINE = ".vize/snapshots/Swatch--Default--small.png";
const DIFF = ".vize/snapshots/diff/Swatch--Default--small.png";

type Summary = Record<"total" | "passed" | "failed" | "new" | "errors", number>;

function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      server.close(() =>
        typeof address === "object" && address ? resolve(address.port) : reject(new Error("port")),
      );
    });
  });
}

async function waitForGallery(url: string, server: ChildProcess): Promise<string | null> {
  for (let attempt = 0; attempt < 120; attempt++) {
    if (server.exitCode !== null) return `vite exited with ${server.exitCode}`;
    const status = await fetch(url).then(
      (response) => response.status,
      () => 0,
    );
    if (status === 200) return null;
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  return `${url} did not answer 200 within 60s`;
}

/** Runs `musea-vrt` and checks its exit status and JSON report summary. */
async function vrt(
  dir: string,
  url: string,
  args: string[],
  expected: { status: "zero" | "nonzero"; summary: Partial<Summary> },
): Promise<string | null> {
  const bin = join(dir, "node_modules/@vizejs/vite-plugin-musea/dist/cli/index.mjs");
  const out = await collectOutput(process.execPath, [bin, "--base-url", url, "--json", ...args], {
    cwd: dir,
  });
  if ((out.status === 0) !== (expected.status === "zero"))
    return `musea-vrt exited ${out.status}:\n${out.stdout}${out.stderr}`;
  let summary: Summary;
  try {
    summary = JSON.parse(readFileSync(join(dir, ".vize/vrt-report.json"), "utf8")).summary;
  } catch {
    return `no vrt-report.json:\n${out.stdout}${out.stderr}`;
  }
  const mismatched = Object.entries(expected.summary).filter(
    ([key, value]) => summary[key as keyof Summary] !== value,
  );
  return mismatched.length === 0 ? null : `summary ${JSON.stringify(summary)}`;
}

function steps(dir: string, version: string, server: { process?: ChildProcess }): Step[] {
  let base = "";
  const url = () => base;
  const packages = ["vite", "vue", "playwright"].concat(
    [`@vizejs/vite-plugin`, `@vizejs/vite-plugin-musea`].map((name) => `${name}@${version}`),
  );
  let baseline = "";
  return [
    ["install", () => install(dir, packages), true],
    [
      "install Chromium",
      async () => {
        const cli = join(dir, "node_modules/playwright/cli.js");
        const args = [cli, "install", ...(process.platform === "linux" ? ["--with-deps"] : [])];
        const status = await run(process.execPath, [...args, "chromium"], { cwd: dir });
        return status === 0 ? null : "playwright install chromium failed";
      },
      true,
    ],
    [
      "serve the gallery with vite",
      async () => {
        const port = await freePort();
        base = `http://127.0.0.1:${port}`;
        const vite = join(dir, "node_modules/vite/bin/vite.js");
        const args = [vite, "--host", "127.0.0.1", "--port", String(port), "--strictPort"];
        server.process = spawn(process.execPath, args, { cwd: dir, stdio: "ignore" });
        return waitForGallery(`${base}/__musea__/`, server.process);
      },
      true,
    ],
    [
      "new baseline",
      async () => {
        const failure = await vrt(dir, url(), ["--ci"], {
          status: "zero",
          summary: { total: 1, new: 1, failed: 0, errors: 0 },
        });
        if (failure) return failure;
        if (!existsSync(join(dir, BASELINE))) return `no ${BASELINE}`;
        baseline = readFileSync(join(dir, BASELINE), "base64");
        return null;
      },
      true,
    ],
    [
      "match",
      () => vrt(dir, url(), ["--ci"], { status: "zero", summary: { total: 1, passed: 1 } }),
    ],
    [
      "diff after a style change",
      async () => {
        write(join(dir, "src/Swatch.vue"), swatch(RED));
        const failure = await vrt(dir, url(), ["--ci"], {
          status: "nonzero",
          summary: { total: 1, failed: 1, passed: 0, errors: 0 },
        });
        return failure ?? (existsSync(join(dir, DIFF)) ? null : `no ${DIFF}`);
      },
    ],
    [
      "update",
      async () => {
        const failure = await vrt(dir, url(), ["--update"], {
          status: "zero",
          summary: { total: 1, errors: 0 },
        });
        if (failure) return failure;
        return baseline === readFileSync(join(dir, BASELINE), "base64")
          ? "--update left the baseline unchanged"
          : null;
      },
    ],
    [
      "match after update",
      () => vrt(dir, url(), ["--ci"], { status: "zero", summary: { total: 1, passed: 1 } }),
    ],
  ];
}

export async function verifyMuseaVrt(version: string): Promise<number> {
  const dir = mkdtempSync(join(tmpdir(), "vize-musea-vrt-"));
  const server: { process?: ChildProcess } = {};
  try {
    write(
      join(dir, "package.json"),
      '{ "name": "musea-vrt", "private": true, "type": "module" }\n',
    );
    write(join(dir, "vite.config.ts"), VITE_CONFIG);
    write(join(dir, "src/Swatch.vue"), swatch(BLUE));
    write(join(dir, "src/Swatch.art.vue"), ART);
    const ok = await runSteps("musea", steps(dir, version, server));
    return ok ? 0 : 1;
  } finally {
    server.process?.kill();
    rmSync(dir, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  runMain(async () => {
    const [version] = cliArgs();
    if (version === undefined) {
      writeStderr("Usage: verify_published/musea_vrt.ts <version>\n");
      return 1;
    }
    return verifyMuseaVrt(version);
  });
}
