/**
 * Generates the `@vizejs/vize` JSR package.
 *
 * JSR cannot host native addons or install scripts, so the package is a typed
 * ESM facade: every entry point re-exports one npm package at the exact Vize
 * release version. The generated files are staged outside the checkout and
 * published from there.
 */
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));
const versionPattern = /^0\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

interface Entrypoint {
  /** Checkout directory of the npm package that the entry point re-exports. */
  readonly directory: string;
  readonly name: string;
  readonly subpath?: string;
  readonly hasDefault?: boolean;
}

export const entrypoints: Readonly<Record<string, Entrypoint>> = {
  "mod.ts": { directory: "npm/cli", name: "vize" },
  "config.ts": { directory: "npm/cli", name: "vize", subpath: "/config" },
  "native.ts": { directory: "npm/native", name: "@vizejs/native" },
  "vite.ts": { directory: "npm/builder/vite", name: "@vizejs/vite-plugin", hasDefault: true },
};

export interface PrepareOptions {
  readonly root?: string;
  readonly version?: string;
}

function readJson(file: string): Record<string, unknown> {
  return JSON.parse(readFileSync(file, "utf8")) as Record<string, unknown>;
}

/** The npm specifier that a facade entry point re-exports. */
export function facadeSpecifier(entrypoint: Entrypoint, version: string): string {
  return `npm:${entrypoint.name}@${version}${entrypoint.subpath ?? ""}`;
}

/** Writes the publishable package into `output` and returns its version. */
export function preparePackage(output: string, options: PrepareOptions = {}): string {
  const root = options.root ?? repository;
  const current = String(readJson(resolve(root, "npm/cli/package.json")).version);
  const version = options.version ?? current;
  if (!versionPattern.test(version) || version !== current) {
    throw new Error(`JSR version must match the checked-out Vize release (${current})`);
  }
  if (!output || resolve(output) === resolve(root, "jsr/vize")) {
    throw new Error("Generate the JSR package in a separate staging directory");
  }
  for (const { directory } of Object.values(entrypoints)) {
    if (readJson(resolve(root, directory, "package.json")).version !== version) {
      throw new Error(`${directory} must match JSR version ${version}`);
    }
  }
  const manifest = readJson(resolve(root, "jsr/vize/jsr.json"));
  mkdirSync(output, { recursive: true });
  // Publication follows the npm release immediately; Deno's default dependency
  // age would otherwise reject the versions that were published minutes ago.
  writeFileSync(
    resolve(output, "jsr.json"),
    `${JSON.stringify({ ...manifest, version, minimumDependencyAge: 0 }, null, 2)}\n`,
  );
  for (const [filename, entrypoint] of Object.entries(entrypoints)) {
    const specifier = JSON.stringify(facadeSpecifier(entrypoint, version));
    const lines = [
      `/** Re-exports ${entrypoint.name}${entrypoint.subpath ?? ""} ${version} from npm. */`,
      `export * from ${specifier};`,
    ];
    if (entrypoint.hasDefault) lines.push(`export { default } from ${specifier};`);
    writeFileSync(resolve(output, filename), `${lines.join("\n")}\n`);
  }
  writeFileSync(resolve(output, "README.md"), readFileSync(resolve(root, "jsr/vize/README.md")));
  writeFileSync(resolve(output, "LICENSE"), readFileSync(resolve(root, "LICENSE")));
  return version;
}

export interface Channel {
  /** Whether a release publishes to JSR. Off until the JSR package exists. */
  readonly enabled: boolean;
}

/** Parses `jsr/vize/channel.json`; an unknown shape never reads as disabled. */
export function parseChannel(text: string): Channel {
  const value: unknown = JSON.parse(text);
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    Object.keys(value).join(",") !== "enabled" ||
    typeof (value as { enabled: unknown }).enabled !== "boolean"
  ) {
    throw new Error('jsr/vize/channel.json must be exactly {"enabled": true|false}');
  }
  return { enabled: (value as Channel).enabled };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [command, ...args] = process.argv.slice(2);
  if (command === "prepare" && args[0]) {
    console.log(preparePackage(args[0], { version: args[1]?.replace(/^v/, "") }));
  } else if (command === "channel" && args.length === 0) {
    const channel = parseChannel(
      readFileSync(resolve(repository, "jsr/vize/channel.json"), "utf8"),
    );
    console.log(`enabled=${channel.enabled}`);
  } else {
    throw new Error("Usage: package.ts prepare <staging-directory> [version] | channel");
  }
}
