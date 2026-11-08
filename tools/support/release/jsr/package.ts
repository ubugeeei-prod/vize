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
import { jsrPackageDirectory, jsrPackages, type JsrPackageDescriptor } from "./packages.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));
const versionPattern = /^0\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;
const defaultJsrPackage = "@vizejs/vize";
export const publicPollIntervalMs = 30_000;
export const publicPollAttempts = 201;

interface Entrypoint {
  /** Checkout directory of the npm package that the entry point re-exports. */
  readonly directory: string;
  readonly name: string;
  readonly subpath?: string;
  readonly hasDefault?: boolean;
}

export const entrypoints: Readonly<Record<string, Entrypoint>> = {
  "mod.ts": { directory: "npm/cli", name: "vize" },
  "babel.ts": {
    directory: "npm/builder/babel",
    name: "@vizejs/babel-plugin",
    hasDefault: true,
  },
  "config.ts": { directory: "npm/cli", name: "vize", subpath: "/config" },
  "native.ts": { directory: "npm/native", name: "@vizejs/native" },
  "oxlint.ts": {
    directory: "npm/linter/oxlint",
    name: "oxlint-plugin-vize",
    hasDefault: true,
  },
  "vite.ts": { directory: "npm/builder/vite", name: "@vizejs/vite-plugin", hasDefault: true },
};

export interface PrepareOptions {
  readonly root?: string;
  readonly version?: string;
}

export interface PreparedJsrPackage extends JsrPackageDescriptor {
  readonly output: string;
  readonly version: string;
}

/** Overrides the npm metadata reader for deterministic release tests. */
export interface NpmPublicationOptions {
  readonly readMetadata?: (packageName: string, version: string) => Promise<unknown>;
}

function readJson(file: string): Record<string, unknown> {
  return JSON.parse(readFileSync(file, "utf8")) as Record<string, unknown>;
}

/** The npm specifier that a facade entry point re-exports. */
export function facadeSpecifier(entrypoint: Entrypoint, version: string): string {
  return `npm:${entrypoint.name}@${version}${entrypoint.subpath ?? ""}`;
}

function writePackage(output: string, root: string, version: string, exactSource: boolean): string {
  if (!versionPattern.test(version)) throw new Error(`Invalid JSR version ${version}`);
  if (!output || resolve(output) === resolve(root, "jsr/vize")) {
    throw new Error("Generate the JSR package in a separate staging directory");
  }
  if (exactSource)
    for (const { directory } of Object.values(entrypoints)) {
      if (readJson(resolve(root, directory, "package.json")).version === version) continue;
      throw new Error(`${directory} must match JSR version ${version}`);
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

function writeStandalonePackage(
  output: string,
  root: string,
  version: string,
  descriptor: JsrPackageDescriptor,
  exactSource: boolean,
): string {
  if (!versionPattern.test(version)) throw new Error(`Invalid JSR version ${version}`);
  const npmManifest = readJson(resolve(root, descriptor.directory, "package.json"));
  if (npmManifest.name !== descriptor.npmName) {
    throw new Error(`${descriptor.directory} must publish ${descriptor.npmName}`);
  }
  if (exactSource && npmManifest.version !== version) {
    throw new Error(`${descriptor.directory} must match JSR version ${version}`);
  }
  mkdirSync(output, { recursive: true });
  writeFileSync(
    resolve(output, "jsr.json"),
    `${JSON.stringify(
      {
        name: descriptor.jsrName,
        version,
        exports: { ".": "./mod.ts" },
        publish: { include: ["mod.ts", "README.md", "LICENSE"] },
        minimumDependencyAge: 0,
      },
      null,
      2,
    )}\n`,
  );
  const specifier = JSON.stringify(`npm:${descriptor.npmName}@${version}`);
  const source = [
    `/** Re-exports ${descriptor.npmName} ${version} from npm. */`,
    `export * from ${specifier};`,
  ];
  if (descriptor.hasDefault) source.push(`export { default } from ${specifier};`);
  writeFileSync(resolve(output, "mod.ts"), `${source.join("\n")}\n`);
  writeFileSync(
    resolve(output, "README.md"),
    `# ${descriptor.jsrName}\n\nJSR source facade for npm \`${descriptor.npmName}\`. Both registries use the same exact Vize release version.\n`,
  );
  writeFileSync(resolve(output, "LICENSE"), readFileSync(resolve(root, "LICENSE")));
  return version;
}

function writePackageSet(
  output: string,
  root: string,
  version: string,
  exactSource: boolean,
): PreparedJsrPackage[] {
  if (!output || resolve(output) === resolve(root, "jsr")) {
    throw new Error("Generate the JSR packages in a separate staging directory");
  }
  return jsrPackages.map((descriptor) => {
    const packageOutput = resolve(output, jsrPackageDirectory(descriptor.jsrName));
    if (descriptor.jsrName === "@vizejs/vize") {
      writePackage(packageOutput, root, version, exactSource);
    } else {
      writeStandalonePackage(packageOutput, root, version, descriptor, exactSource);
    }
    return { ...descriptor, output: packageOutput, version };
  });
}

function packageDescriptor(name: string): JsrPackageDescriptor {
  const descriptor = jsrPackages.find(({ jsrName }) => jsrName === name);
  if (!descriptor) throw new Error(`Unknown JSR package ${name}`);
  return descriptor;
}

/** Refuses metadata that does not prove one immutable npm package version. */
export function assertPublishedNpmVersion(
  metadata: unknown,
  packageName: string,
  version: string,
): void {
  if (!versionPattern.test(version)) throw new Error(`Invalid JSR version ${version}`);
  const value = record(metadata);
  const dist = record(value?.dist);
  if (
    value?.name !== packageName ||
    value.version !== version ||
    typeof dist?.integrity !== "string" ||
    !/^sha512-[A-Za-z0-9+/]+={0,2}$/u.test(dist.integrity)
  ) {
    throw new Error(`npm registry did not prove ${packageName}@${version} with integrity`);
  }
}

/** Reads one exact, immutable npm package version from the public registry. */
async function readPublicNpmMetadata(packageName: string, version: string): Promise<unknown> {
  const response = await fetch(
    `https://registry.npmjs.org/${encodeURIComponent(packageName)}/${version}`,
    {
      headers: { accept: "application/json", "user-agent": "vize-release-jsr/1.0" },
      signal: AbortSignal.timeout(30_000),
    },
  );
  if (!response.ok) {
    throw new Error(`npm metadata for ${packageName}@${version} returned HTTP ${response.status}`);
  }
  return response.json();
}

/** Verifies the registry response for one exact npm package version. */
async function verifyPublishedNpmVersion(
  packageName: string,
  version: string,
  options: NpmPublicationOptions,
): Promise<void> {
  if (!versionPattern.test(version)) throw new Error(`Invalid JSR version ${version}`);
  const readMetadata = options.readMetadata ?? readPublicNpmMetadata;
  assertPublishedNpmVersion(await readMetadata(packageName, version), packageName, version);
}

function writeOnePackage(
  output: string,
  root: string,
  version: string,
  descriptor: JsrPackageDescriptor,
  exactSource: boolean,
): string {
  return descriptor.jsrName === defaultJsrPackage
    ? writePackage(output, root, version, exactSource)
    : writeStandalonePackage(output, root, version, descriptor, exactSource);
}

/** Writes the publishable package into `output` and returns its exact source version. */
export function preparePackage(output: string, options: PrepareOptions = {}): string {
  const root = options.root ?? repository;
  const current = String(readJson(resolve(root, "npm/cli/package.json")).version);
  const version = options.version ?? current;
  if (version !== current) {
    throw new Error(`JSR version must match the checked-out Vize release (${current})`);
  }
  return writePackage(output, root, version, true);
}

/**
 * Writes a non-publishable PR preview against one already-public npm release.
 * Release publication always uses `preparePackage` and exact source versions.
 */
export function preparePreviewPackage(
  output: string,
  version: string,
  options: Pick<PrepareOptions, "root"> = {},
): string {
  return writePackage(output, options.root ?? repository, version, false);
}

/** Writes every npm release package's JSR facade under one staging root. */
export function preparePackageSet(
  output: string,
  options: PrepareOptions = {},
): PreparedJsrPackage[] {
  const root = options.root ?? repository;
  const current = String(readJson(resolve(root, "npm/cli/package.json")).version);
  const version = options.version ?? current;
  if (version !== current) {
    throw new Error(`JSR version must match the checked-out Vize release (${current})`);
  }
  return writePackageSet(output, root, version, true);
}

/** Writes non-publishable facades against one already-public npm release. */
export function preparePreviewPackageSet(
  output: string,
  version: string,
  options: Pick<PrepareOptions, "root"> = {},
): PreparedJsrPackage[] {
  return writePackageSet(output, options.root ?? repository, version, false);
}

/** Writes one named JSR facade for a release matrix cell. */
export function prepareOnePackage(
  output: string,
  packageName: string,
  options: PrepareOptions = {},
): string {
  const root = options.root ?? repository;
  const current = String(readJson(resolve(root, "npm/cli/package.json")).version);
  const version = options.version ?? current;
  if (version !== current) {
    throw new Error(`JSR version must match the checked-out Vize release (${current})`);
  }
  return writeOnePackage(output, root, version, packageDescriptor(packageName), true);
}

/**
 * Stages the compatibility facade for an older npm release after proving every
 * referenced package and its immutable integrity directly from the registry.
 */
export async function prepareBackfillPackage(
  output: string,
  version: string,
  options: Pick<PrepareOptions, "root"> & NpmPublicationOptions = {},
): Promise<string> {
  const packageNames = new Set(Object.values(entrypoints).map(({ name }) => name));
  await Promise.all(
    [...packageNames].map((packageName) =>
      verifyPublishedNpmVersion(packageName, version, options),
    ),
  );
  return writePackage(output, options.root ?? repository, version, false);
}

/** Stages one older release facade only after its exact npm version is public. */
export async function prepareBackfillOnePackage(
  output: string,
  packageName: string,
  version: string,
  options: Pick<PrepareOptions, "root"> & NpmPublicationOptions = {},
): Promise<string> {
  const descriptor = packageDescriptor(packageName);
  await verifyPublishedNpmVersion(descriptor.npmName, version, options);
  return writeOnePackage(output, options.root ?? repository, version, descriptor, false);
}

export interface Channel {
  /** Whether a release publishes to JSR. Off until the JSR package exists. */
  readonly enabled: boolean;
}

export interface PublicationWaitOptions {
  readonly attempts?: number;
  readonly intervalMs?: number;
  readonly readMetadata?: () => Promise<unknown>;
  readonly sleep?: (milliseconds: number) => Promise<void>;
}

function record(value: unknown): Record<string, unknown> | undefined {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

/** Returns the registry publication time, or undefined while the version is absent. */
export function publishedAt(metadata: unknown, version: string): string | undefined {
  if (!versionPattern.test(version)) throw new Error(`Invalid JSR version ${version}`);
  const versions = record(record(metadata)?.versions);
  if (!versions || versions[version] === undefined) return undefined;
  const createdAt = record(versions[version])?.createdAt;
  if (typeof createdAt !== "string" || Number.isNaN(Date.parse(createdAt))) {
    throw new Error(`JSR metadata for ${version} has no valid createdAt`);
  }
  return createdAt;
}

/** Returns the latest version only when it has an immutable public record. */
export function latestPublishedVersion(metadata: unknown): string {
  const latest = record(metadata)?.latest;
  if (typeof latest !== "string" || !publishedAt(metadata, latest)) {
    throw new Error("JSR metadata has no valid latest public version");
  }
  return latest;
}

async function readPublicMetadata(packageName = defaultJsrPackage): Promise<unknown> {
  packageDescriptor(packageName);
  const response = await fetch(`https://jsr.io/${packageName}/meta.json`, {
    headers: { accept: "application/json", "user-agent": "vize-release-jsr/1.0" },
  });
  if (!response.ok) throw new Error(`JSR metadata returned HTTP ${response.status}`);
  return response.json();
}

/** Waits for the immutable public registry record after JSR accepts an upload. */
export async function waitForPublishedVersion(
  version: string,
  options: PublicationWaitOptions = {},
): Promise<string> {
  const attempts = options.attempts ?? publicPollAttempts;
  const intervalMs = options.intervalMs ?? publicPollIntervalMs;
  if (!Number.isSafeInteger(attempts) || attempts < 1) throw new Error("attempts must be positive");
  if (!Number.isSafeInteger(intervalMs) || intervalMs < 0) {
    throw new Error("intervalMs must be non-negative");
  }
  const readMetadata = options.readMetadata ?? readPublicMetadata;
  const sleep =
    options.sleep ??
    ((milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds)));
  let lastError: unknown;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      const createdAt = publishedAt(await readMetadata(), version);
      if (createdAt) return createdAt;
      lastError = undefined;
    } catch (error) {
      lastError = error;
    }
    if (attempt < attempts) await sleep(intervalMs);
  }
  const duration = ((attempts - 1) * intervalMs) / 60_000;
  const suffix = lastError instanceof Error ? `; last check failed: ${lastError.message}` : "";
  throw new Error(`JSR ${version} was not public after ${duration} minutes${suffix}`);
}

/** Waits for one package in the release set to expose the immutable version. */
export function waitForPublishedPackage(
  packageName: string,
  version: string,
  options: PublicationWaitOptions = {},
): Promise<string> {
  packageDescriptor(packageName);
  return waitForPublishedVersion(version, {
    ...options,
    readMetadata: options.readMetadata ?? (() => readPublicMetadata(packageName)),
  });
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
  } else if (command === "prepare-all" && args[0]) {
    const prepared = preparePackageSet(args[0], { version: args[1]?.replace(/^v/, "") });
    console.log(JSON.stringify(prepared));
  } else if (command === "prepare-one" && args.length >= 2) {
    console.log(prepareOnePackage(args[0], args[1], { version: args[2]?.replace(/^v/, "") }));
  } else if (command === "prepare-backfill" && args.length === 2) {
    console.log(await prepareBackfillPackage(args[0], args[1].replace(/^v/, "")));
  } else if (command === "prepare-backfill-one" && args.length === 3) {
    console.log(await prepareBackfillOnePackage(args[0], args[1], args[2].replace(/^v/, "")));
  } else if (command === "prepare-preview" && args.length === 2) {
    console.log(preparePreviewPackage(args[0], args[1].replace(/^v/, "")));
  } else if (command === "prepare-preview-all" && args.length === 2) {
    console.log(JSON.stringify(preparePreviewPackageSet(args[0], args[1].replace(/^v/, ""))));
  } else if (command === "list" && args.length === 0) {
    console.log(JSON.stringify(jsrPackages));
  } else if (command === "matrix" && args.length === 0) {
    console.log(`packages=${JSON.stringify({ include: jsrPackages.slice(1) })}`);
  } else if (command === "latest-public" && args.length === 0) {
    console.log(latestPublishedVersion(await readPublicMetadata()));
  } else if (command === "channel" && args.length === 0) {
    const channel = parseChannel(
      readFileSync(resolve(repository, "jsr/vize/channel.json"), "utf8"),
    );
    console.log(`enabled=${channel.enabled}`);
  } else if (command === "is-public" && (args.length === 1 || args.length === 2)) {
    const [packageName, version] = args.length === 1 ? [defaultJsrPackage, args[0]] : args;
    const createdAt = publishedAt(await readPublicMetadata(packageName), version);
    console.log(createdAt ? `published=${createdAt}` : "published=");
    if (!createdAt) process.exitCode = 1;
  } else if (command === "wait-public" && (args.length === 1 || args.length === 2)) {
    const [packageName, version] = args.length === 1 ? [defaultJsrPackage, args[0]] : args;
    console.log(`published=${await waitForPublishedPackage(packageName, version)}`);
  } else {
    throw new Error(
      "Usage: package.ts prepare <staging-directory> [version] | prepare-all <staging-directory> [version] | prepare-one <staging-directory> <package-name> [version] | prepare-backfill <staging-directory> <published-version> | prepare-backfill-one <staging-directory> <package-name> <published-version> | prepare-preview <staging-directory> <published-version> | prepare-preview-all <staging-directory> <published-version> | list | matrix | latest-public | channel | is-public [package-name] <version> | wait-public [package-name] <version>",
    );
  }
}
