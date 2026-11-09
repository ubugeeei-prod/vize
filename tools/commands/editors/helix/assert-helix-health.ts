import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { runEntrypoint } from "../../../support/command-runtime.ts";
import { repositoryRoot } from "../../../support/compat/editor-e2e/real-vue-workspace.ts";

export function stripAnsi(value: string): string {
  let plain = "";
  for (let index = 0; index < value.length; index += 1) {
    if (value.charCodeAt(index) !== 0x1b || value[index + 1] !== "[") {
      plain += value[index];
      continue;
    }
    index += 2;
    while (index < value.length) {
      const code = value.charCodeAt(index);
      if (code >= 0x40 && code <= 0x7e) break;
      index += 1;
    }
  }
  return plain;
}

export function linesBetween(value: string, start: string, end: string): string[] {
  const lines = value.split(/\r?\n/u).map((line) => line.trim());
  const startAt = lines.indexOf(start);
  if (startAt < 0) throw new Error(`missing ${JSON.stringify(start)} in Helix health output`);
  const endAt = lines.findIndex((line) => line.startsWith(end));
  if (endAt <= startAt)
    throw new Error(`missing ${JSON.stringify(end)} after ${JSON.stringify(start)}`);
  return lines.slice(startAt + 1, endAt).filter(Boolean);
}

function resolveServerPath(): string {
  const configured = process.env.VIZE_SERVER_PATH?.trim();
  const candidate = configured || path.join(repositoryRoot, "target", "ci", "vize");
  try {
    return fs.realpathSync(candidate);
  } catch (error) {
    throw new Error(`Vize server does not exist: ${String(error)}`);
  }
}

function requireEnvironmentPath(name: "HELIX_RUNTIME" | "XDG_CONFIG_HOME"): string {
  const value = process.env[name]?.trim();
  if (!value) throw new Error(`${name} must point at the isolated Helix environment`);
  return value;
}

function main(): number {
  const helixBinary = process.argv[2] ?? "hx";
  const serverPath = resolveServerPath();
  const configHome = requireEnvironmentPath("XDG_CONFIG_HOME");
  const runtime = requireEnvironmentPath("HELIX_RUNTIME");
  if (path.dirname(helixBinary) !== "." && !fs.existsSync(helixBinary))
    throw new Error(`Helix binary does not exist: ${helixBinary}`);
  if (!fs.existsSync(runtime)) throw new Error(`Helix runtime does not exist: ${runtime}`);

  const installed = path.join(configHome, "helix", "languages.toml");
  const packaged = path.join(repositoryRoot, "editors", "helix", "languages.toml");
  if (fs.readFileSync(installed, "utf8") !== fs.readFileSync(packaged, "utf8"))
    throw new Error("Helix must inspect the exact packaged languages.toml");

  const expectedServer = `✓ vize: ${serverPath}`;
  for (const language of ["vue", "art-vue"]) {
    const result = spawnSync(helixBinary, ["--health", language], {
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${path.dirname(serverPath)}${path.delimiter}${process.env.PATH ?? ""}`,
      },
    });
    if (result.error !== undefined)
      throw new Error(`failed to run hx --health ${language}: ${result.error}`);
    const output = stripAnsi(`${result.stdout ?? ""}${result.stderr ?? ""}`);
    if (result.status !== 0) throw new Error(`hx --health ${language} failed:\n${output}`);
    const configured = linesBetween(
      output,
      "Configured language servers:",
      "Configured debug adapter:",
    );
    if (configured.length !== 1 || configured[0] !== expectedServer)
      throw new Error(
        `hx --health ${language} did not resolve exactly the packaged Vize server: ${JSON.stringify(configured)}`,
      );
  }
  console.log("helix package health passed for vue and art-vue");
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href)
  runEntrypoint(main);
