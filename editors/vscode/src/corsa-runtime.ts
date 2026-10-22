import * as crypto from "crypto";
import * as fs from "fs";
import * as https from "https";
import * as path from "path";
import * as zlib from "zlib";

const OVERRIDE_KEYS = ["CORSA_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE"] as const;
const NPM_HOST = "registry.npmjs.org";
const DOWNLOAD_TIMEOUT_MS = 120_000;
const MAX_ARCHIVE_BYTES = 64 * 1024 * 1024;

const installs = new Map<string, Promise<void>>();

export type CorsaServerEnvInput = {
  extensionPath: string;
  globalStoragePath: string;
  output?: { appendLine(line: string): void };
  env?: NodeJS.ProcessEnv;
  platform?: NodeJS.Platform;
  arch?: string;
  install?: (request: CorsaInstallRequest) => Promise<void>;
};

export type CorsaInstallRequest = {
  packageName: string;
  version: string;
  destination: string;
  executableName: string;
};

/** Platform `lib/tsc`. Overrides win; the vsix does not vendor the optional package. */
export async function resolveCorsaServerEnv(
  input: CorsaServerEnvInput,
): Promise<NodeJS.ProcessEnv> {
  const env = { ...(input.env ?? process.env) };
  if (hasOverride(env)) return env;

  const platform = input.platform ?? process.platform;
  const arch = input.arch ?? process.arch;
  const packageName = `@typescript/typescript-${platform}-${arch}`;
  const executableName = platform === "win32" ? "tsc.exe" : "tsc";
  const version = declaredVersion(input.extensionPath, packageName);
  const adjacent = path.join(input.extensionPath, "node_modules", packageName);
  const cached = version
    ? path.join(input.globalStoragePath, "corsa", version, `${platform}-${arch}`)
    : undefined;
  const expected = path.join(
    cached ?? path.join(input.globalStoragePath, "corsa", `${platform}-${arch}`),
    "lib",
    executableName,
  );

  const use = (executable: string): NodeJS.ProcessEnv => {
    env.CORSA_PATH = executable;
    input.output?.appendLine(`Using Corsa runtime: ${executable}`);
    return env;
  };

  const found =
    existingRuntime(adjacent, packageName, version, executableName) ??
    (cached ? existingRuntime(cached, packageName, version, executableName) : undefined);
  if (found) return use(found);

  let failure = "";
  if (version && cached && isExactVersion(version)) {
    try {
      await installRuntime(input, { packageName, version, destination: cached, executableName });
      const installed = existingRuntime(cached, packageName, version, executableName);
      if (installed) return use(installed);
      failure = " Download did not produce the executable.";
    } catch (error) {
      failure = ` ${String(error)}`;
    }
  }

  const required = version ? `${packageName} ${version}` : packageName;
  input.output?.appendLine(`Missing Corsa runtime ${expected}. ${required} is required.${failure}`);
  return env;
}

export function serverProcessEnv(env: NodeJS.ProcessEnv, debug: boolean): NodeJS.ProcessEnv {
  return debug ? { ...env, RUST_BACKTRACE: "1" } : env;
}

function hasOverride(env: NodeJS.ProcessEnv): boolean {
  return OVERRIDE_KEYS.some((key) => {
    const value = env[key];
    return value != null && value !== "";
  });
}

function declaredVersion(extensionPath: string, packageName: string): string | undefined {
  try {
    const manifest = JSON.parse(
      fs.readFileSync(path.join(extensionPath, "package.json"), "utf8"),
    ) as { optionalDependencies?: Record<string, unknown> };
    const version = manifest.optionalDependencies?.[packageName];
    return typeof version === "string" && version !== "" ? version : undefined;
  } catch {
    return undefined;
  }
}

function isExactVersion(version: string): boolean {
  return /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version);
}

function existingRuntime(
  packageRoot: string,
  packageName: string,
  version: string | undefined,
  executableName: string,
): string | undefined {
  if (!version) return undefined;
  try {
    const manifest = JSON.parse(
      fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"),
    ) as {
      name?: unknown;
      version?: unknown;
    };
    if (manifest.name !== packageName || manifest.version !== version) return undefined;
  } catch {
    return undefined;
  }
  const executable = path.join(packageRoot, "lib", executableName);
  return isFile(executable) ? executable : undefined;
}

function isFile(file: string): boolean {
  try {
    return fs.statSync(file).isFile();
  } catch {
    return false;
  }
}

async function installRuntime(
  input: CorsaServerEnvInput,
  request: CorsaInstallRequest,
): Promise<void> {
  if (input.install) {
    await input.install(request);
    return;
  }
  const pending = installs.get(request.destination);
  if (pending) {
    await pending;
    return;
  }
  const task = downloadRuntime(request).finally(() => {
    installs.delete(request.destination);
  });
  installs.set(request.destination, task);
  await task;
}

async function downloadRuntime(request: CorsaInstallRequest): Promise<void> {
  const metadata = (await fetchJson(
    `https://${NPM_HOST}/${request.packageName}/${request.version}`,
  )) as { dist?: { tarball?: unknown; integrity?: unknown } };
  const tarball = metadata.dist?.tarball;
  const integrity = metadata.dist?.integrity;
  if (typeof tarball !== "string" || typeof integrity !== "string") {
    throw new Error(`npm metadata for ${request.packageName}@${request.version} has no tarball`);
  }
  const archive = await fetchBuffer(tarball);
  verifyIntegrity(archive, integrity);
  const staging = `${request.destination}.staging-${process.pid}`;
  await fs.promises.rm(staging, { recursive: true, force: true });
  await fs.promises.mkdir(staging, { recursive: true });
  try {
    extractNpmPackage(zlib.gunzipSync(archive), staging);
    const executable = path.join(staging, "lib", request.executableName);
    if (!isFile(executable)) {
      throw new Error(`archive did not contain lib/${request.executableName}`);
    }
    if (process.platform !== "win32") await fs.promises.chmod(executable, 0o755);
    await fs.promises.rm(request.destination, { recursive: true, force: true });
    await fs.promises.mkdir(path.dirname(request.destination), { recursive: true });
    await fs.promises.rename(staging, request.destination);
  } catch (error) {
    await fs.promises.rm(staging, { recursive: true, force: true });
    throw error;
  }
}

function verifyIntegrity(contents: Buffer, integrity: string): void {
  const actual = crypto.createHash("sha512").update(contents).digest();
  const matches = integrity.split(/\s+/).some((token) => {
    if (!token.startsWith("sha512-")) return false;
    const expected = Buffer.from(token.slice("sha512-".length), "base64");
    return expected.length === actual.length && crypto.timingSafeEqual(expected, actual);
  });
  if (!matches) throw new Error("Corsa runtime tarball failed its sha512 integrity check");
}

function fetchJson(url: string): Promise<unknown> {
  return fetchBuffer(url).then((body) => JSON.parse(body.toString("utf8")) as unknown);
}

function fetchBuffer(url: string, redirects = 0): Promise<Buffer> {
  assertNpmUrl(url);
  if (redirects > 5)
    return Promise.reject(new Error(`too many redirects while downloading ${url}`));
  return new Promise((resolve, reject) => {
    let settled = false;
    const fail = (error: Error): void => {
      if (settled) return;
      settled = true;
      reject(error);
    };
    const succeed = (value: Buffer): void => {
      if (settled) return;
      settled = true;
      resolve(value);
    };
    const request = https.get(
      url,
      { headers: { "User-Agent": "vscode-vize", Accept: "*/*" } },
      (response) => {
        const status = response.statusCode ?? 0;
        if (status >= 300 && status < 400 && response.headers.location) {
          response.resume();
          fetchBuffer(new URL(response.headers.location, url).toString(), redirects + 1).then(
            succeed,
            fail,
          );
          return;
        }
        if (status !== 200) {
          response.resume();
          fail(new Error(`download failed with HTTP ${status}: ${url}`));
          return;
        }
        const chunks: Buffer[] = [];
        let received = 0;
        response.on("data", (chunk: Buffer) => {
          received += chunk.length;
          if (received > MAX_ARCHIVE_BYTES) {
            request.destroy(new Error("Corsa runtime download exceeded 64MB"));
            return;
          }
          chunks.push(chunk);
        });
        response.on("end", () => succeed(Buffer.concat(chunks)));
        response.on("error", fail);
      },
    );
    request.setTimeout(DOWNLOAD_TIMEOUT_MS, () => {
      request.destroy(new Error(`download timed out after ${DOWNLOAD_TIMEOUT_MS}ms`));
    });
    request.on("error", fail);
  });
}

function assertNpmUrl(url: string): void {
  const parsed = new URL(url);
  if (
    parsed.protocol !== "https:" ||
    parsed.hostname !== NPM_HOST ||
    parsed.port !== "" ||
    parsed.username !== "" ||
    parsed.password !== ""
  ) {
    throw new Error(`refusing to download Corsa runtime from untrusted URL: ${url}`);
  }
}

export function extractNpmPackage(tar: Buffer, destination: string): void {
  for (let offset = 0; offset + 512 <= tar.length;) {
    const header = tar.subarray(offset, offset + 512);
    if (header.every((byte) => byte === 0)) break;
    const size = parseTarOctal(header, 124, 12);
    const type = header[156] ?? 0;
    const dataStart = offset + 512;
    if (dataStart + size > tar.length) throw new Error("truncated Corsa runtime archive");
    offset = dataStart + Math.ceil(size / 512) * 512;
    if (type !== 0 && type !== 48) continue;
    const relative = packagedPath(tarName(header));
    if (!relative) continue;
    const target = containedPath(destination, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, Buffer.from(tar.subarray(dataStart, dataStart + size)));
  }
}

function tarName(header: Buffer): string {
  const name = readTarString(header, 0, 100);
  const prefix = readTarString(header, 345, 155);
  return prefix ? `${prefix}/${name}` : name;
}

function readTarString(buffer: Buffer, start: number, length: number): string {
  const raw = buffer.subarray(start, start + length);
  const end = raw.indexOf(0);
  return raw.subarray(0, end === -1 ? raw.length : end).toString("utf8");
}

function parseTarOctal(buffer: Buffer, start: number, length: number): number {
  const text = readTarString(buffer, start, length).trim();
  if (!text) return 0;
  const value = Number.parseInt(text, 8);
  if (!Number.isFinite(value) || value < 0) throw new Error("invalid tar entry size");
  return value;
}

function packagedPath(name: string): string | undefined {
  const normalized = name.replace(/^\.\//, "");
  if (!normalized.startsWith("package/")) return undefined;
  const relative = normalized.slice("package/".length);
  if (!relative || path.isAbsolute(relative) || relative.split(/[/\\]/).includes("..")) {
    return undefined;
  }
  return relative;
}

function containedPath(root: string, relative: string): string {
  const target = path.resolve(root, relative);
  const base = path.resolve(root);
  if (target !== base && !target.startsWith(`${base}${path.sep}`)) {
    throw new Error(`refusing to extract ${relative}`);
  }
  return target;
}
