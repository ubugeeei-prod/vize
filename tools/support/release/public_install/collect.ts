// Prospective public-install custody producer. No source executable fallback.
//
// The reviewed collector snapshot is hashed before and after collection. Node's
// optional compile cache is keyed by source content, so no stale compiled form
// can execute instead of the reviewed bytes.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { isDeepStrictEqual, parseArgs } from "node:util";
import { archiveComparison, recheckFiles, type InstalledFile } from "./archive.ts";
import {
  CustodyError,
  corsaVersionPattern,
  digest,
  exactPath,
  freshOutputs,
  lookup,
  majorAtLeast7,
  overrides,
  producerAuthority,
  rejectOverrides,
  sourceFile,
  sourceIdentity,
  sourceManifests,
} from "./identity.ts";
import { publicProbe } from "./probe.ts";
import { packageMetadata, readBytes } from "./registry.ts";

const platform = "@vizejs/native-darwin-arm64";
const corsa = "@typescript/typescript-darwin-arm64";
const vizePackages = ["vize", "oxlint-plugin-vize", "@vizejs/native", platform] as const;
const producerRoot = fileURLToPath(new URL("../../../../", import.meta.url)).replace(/\/$/u, "");
export interface CollectArguments {
  readonly root: string;
  readonly cut: string;
  readonly head: string;
  readonly tag: string;
  readonly run: string;
  readonly "install-root": string;
  readonly node: string;
  readonly output: string;
  readonly "collector-sha256": string;
}

const readJson = (file: string): unknown => JSON.parse(fs.readFileSync(file, "utf8"));

/** Python `json.dumps(..., indent=2)`: non-ASCII characters are escaped. */
const asciiJson = (value: unknown, indent?: number): string =>
  JSON.stringify(value, null, indent).replace(/[\u{80}-\u{10ffff}]/gu, (character) =>
    Array.from(
      { length: character.length },
      (_, index) => `\\u${character.charCodeAt(index).toString(16).padStart(4, "0")}`,
    ).join(""),
  );

function writeNew(file: string, packet: unknown): void {
  fs.writeFileSync(file, `${asciiJson(packet, 2)}\n`, { flag: "wx" });
}

const executable = (file: string): boolean => {
  try {
    fs.accessSync(file, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
};

export async function collect(args: CollectArguments): Promise<Record<string, unknown>> {
  rejectOverrides(process.env);
  const authority = producerAuthority(producerRoot, args["collector-sha256"]);
  const root = exactPath(args.root, undefined, true);
  const source = sourceIdentity(root, args.cut, args.head, args.tag, args.run);
  const sourceDeclarations = sourceManifests(root, source);
  const version = args.tag.slice(1);
  const install = exactPath(args["install-root"], undefined, true);
  if (
    install === root ||
    install.startsWith(`${root}/`) ||
    fs.existsSync(path.join(install, ".git"))
  )
    throw new CustodyError("fresh public installation must be outside the source workspace");
  const [output, payloadManifest, journal] = freshOutputs(args.output);
  if (output.startsWith(`${install}/`))
    throw new CustodyError("custody outputs must be outside installed payloads");
  const node = exactPath(args.node);
  if (!executable(node)) throw new CustodyError("explicit Node executable required");
  const hook = exactPath(
    path.join(producerRoot, "tools/support/release/public_install/native-custody.cjs"),
  );
  const lockPath = exactPath(path.join(install, "package-lock.json"), install);
  const lockBytes = fs.readFileSync(lockPath);
  const lock: unknown = JSON.parse(lockBytes.toString("utf8"));
  const dependencies = Object.fromEntries(vizePackages.map((name) => [name, version]));
  const consumer = readJson(exactPath(path.join(install, "package.json"), install));
  if (
    lookup(lock, "lockfileVersion") !== 3 ||
    lookup(consumer, "private") !== true ||
    !isDeepStrictEqual(lookup(consumer, "dependencies"), dependencies) ||
    !isDeepStrictEqual(lookup(lock, "packages", "", "dependencies"), dependencies)
  )
    throw new CustodyError(
      "fresh private npm consumer and lock v3 with all four exact Vize versions required",
    );
  const cliRoot = path.join(install, "node_modules/vize");
  const cliManifest = readJson(exactPath(path.join(cliRoot, "package.json"), install));
  const corsaVersion = lookup(cliManifest, "optionalDependencies", corsa);
  if (
    typeof corsaVersion !== "string" ||
    !corsaVersionPattern.test(corsaVersion) ||
    !majorAtLeast7(corsaVersion)
  )
    throw new CustodyError("exact public bundled Corsa version declaration missing");
  if (
    corsaVersion !== sourceDeclarations.corsaVersion ||
    lookup(cliManifest, "dependencies", "@vizejs/native") !== version
  )
    throw new CustodyError(
      "public CLI native/Corsa dependency differs from its actual H declaration",
    );
  const records: Record<string, unknown>[] = [];
  const archiveReceipts: (Record<string, unknown> & {
    packageDirectory: string;
    files: readonly InstalledFile[];
  })[] = [];
  let bundledCorsa: Record<string, unknown> & { path: string } = { path: "" };
  const plans = [
    ...vizePackages.map((name) => [name, version, true] as const),
    [corsa, corsaVersion, false] as const,
  ];
  for (const [name, expected, vizeSource] of plans) {
    const packageDir = exactPath(path.join(install, "node_modules", name), install, true);
    const manifest = readJson(exactPath(path.join(packageDir, "package.json"), packageDir));
    const entry = lookup(lock, "packages", `node_modules/${name}`) ?? {};
    if (
      lookup(manifest, "name") !== name ||
      lookup(manifest, "version") !== expected ||
      lookup(entry, "version") !== expected ||
      lookup(entry, "link") === true
    )
      throw new CustodyError(`installed package identity differs: ${name}`);
    const proof = await packageMetadata(name, expected, entry, vizeSource ? source : undefined);
    const resolved = String(lookup(entry, "resolved"));
    const integrity = String(lookup(entry, "integrity"));
    const comparison = archiveComparison(packageDir, await readBytes(resolved), integrity);
    archiveReceipts.push({
      name,
      version: expected,
      resolved,
      integrity,
      packageDirectory: packageDir,
      source: vizeSource ? { H: source.H, R: source.R } : { thirdPartyRegistry: true },
      provenancePayloadBinding: proof,
      ...comparison,
    });
    if (vizeSource) {
      records.push({
        name,
        version,
        resolved,
        integrity,
        provenanceSourceH: source.H,
        provenanceR: source.R,
        provenancePayloadBinding: proof,
      });
    } else {
      const launcher = exactPath(path.join(packageDir, "lib/tsc"), packageDir);
      if (!executable(launcher))
        throw new CustodyError("public bundled Corsa launcher is not executable");
      bundledCorsa = {
        packageName: name,
        version: expected,
        declaredVersion: corsaVersion,
        path: launcher,
        sha256: digest(fs.readFileSync(launcher)),
        resolved,
        integrity,
        allInstalledFileBytesEqualPublicArchive: true,
      };
    }
  }
  const cliBin = exactPath(path.join(cliRoot, String(lookup(cliManifest, "bin", "vize"))), cliRoot);
  const distCli = exactPath(path.join(cliRoot, "dist/cli.mjs"), install);
  if (!fs.readFileSync(cliBin).equals(sourceFile(root, source.H, "npm/cli/bin/vize")))
    throw new CustodyError("public CLI wrapper differs from H");
  recheckFiles(archiveReceipts);
  writeNew(payloadManifest, {
    schema: "vize-public-registry-payload-files-v1",
    installRoot: install,
    source,
    packages: archiveReceipts,
  });
  const payloadBytes = fs.readFileSync(exactPath(payloadManifest));
  const probe = publicProbe(
    node,
    hook,
    install,
    platform,
    cliBin,
    journal,
    version,
    bundledCorsa.path,
  );
  recheckFiles(archiveReceipts);
  if (
    !fs.readFileSync(lockPath).equals(lockBytes) ||
    !fs.readFileSync(exactPath(payloadManifest)).equals(payloadBytes) ||
    !isDeepStrictEqual(producerAuthority(producerRoot), authority)
  )
    throw new CustodyError(
      "lock, payload manifest or reviewed collector bytes changed during collection",
    );
  const report = {
    schema: "vize-public-registry-install-v1",
    checkedAt: new Date().toISOString(),
    version,
    source,
    installRoot: install,
    packageLockPath: lockPath,
    packageLockSha256: digest(lockBytes),
    registry: records,
    bundledCorsa,
    publicArchiveComparison: archiveReceipts.map(({ files: _files, ...item }) => item),
    payloadManifestPath: payloadManifest,
    payloadManifestSha256: digest(payloadBytes),
    node: probe.node,
    cli: {
      binPath: cliBin,
      binSha256: digest(fs.readFileSync(cliBin)),
      distCliPath: distCli,
      distCliSha256: digest(fs.readFileSync(distCli)),
      versionStdout: probe.stdout,
      versionStderr: probe.stderr,
      versionExitCode: probe.exitCode,
    },
    native: probe.native,
    custodyHook: {
      path: hook,
      sha256: digest(fs.readFileSync(hook)),
      configurationEnvironment: "VIZE_PUBLIC_NATIVE_CUSTODY",
    },
    sourceOverrides: Object.fromEntries(overrides.map((key) => [key, ""])),
    collectorAuthority: authority,
    sourceManifestPayloads: sourceDeclarations.files,
    success: true,
  };
  writeNew(output, report);
  return {
    output,
    receiptSHA256: digest(fs.readFileSync(exactPath(output))),
    version,
    source: source.H,
    successfulReturnObserved: true,
    success: true,
  };
}

async function main(): Promise<void> {
  rejectOverrides(process.env);
  const text = { type: "string" } as const;
  const { values } = parseArgs({
    options: {
      "print-collector-authority": { type: "boolean", default: false },
      root: text,
      cut: text,
      head: text,
      tag: text,
      run: text,
      "install-root": text,
      node: text,
      output: text,
      "collector-sha256": text,
    },
  });
  if (values["print-collector-authority"] === true) {
    process.stdout.write(`${JSON.stringify(producerAuthority(producerRoot), null, 2)}\n`);
    return;
  }
  const required = (value: string | undefined): string => {
    if (value === undefined)
      throw new CustodyError(
        "all explicit source/install/Node/output/reviewed collector arguments are required",
      );
    return value;
  };
  const args: CollectArguments = {
    root: required(values.root),
    cut: required(values.cut),
    head: required(values.head),
    tag: required(values.tag),
    run: required(values.run),
    "install-root": required(values["install-root"]),
    node: required(values.node),
    output: required(values.output),
    "collector-sha256": required(values["collector-sha256"]),
  };
  process.stdout.write(`${JSON.stringify(await collect(args))}\n`);
}

if (import.meta.main) {
  try {
    await main();
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
