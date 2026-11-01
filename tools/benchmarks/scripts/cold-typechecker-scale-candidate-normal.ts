/** The first22 declared target commands; native profiling remains a separate final eight. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import {
  prepareColdScale,
  captureColdScale,
  verifyColdDiagnostics,
  coldScaleSupportScriptFiles,
} from "./cold-typechecker-scale.ts";
import { strictDebugParts, verifyStrictDebug } from "./cold-typechecker-scale-strict-pair.ts";
import {
  prepareNativeForwarder,
  collectNativePreflight,
  compareNativePreflights,
} from "./cold-typechecker-scale-native-preflight.ts";
import { verifyComponentChecksTransition } from "./cold-typechecker-scale-candidate-law.ts";
import { materializedCandidateHelpers } from "./cold-typechecker-scale-candidate-inputs.ts";
import {
  captureCandidateUntimed,
  verifyChildAllocationObservation,
  verifyRouteCensus,
  verifyCandidateAllocationProfile,
} from "./cold-typechecker-scale-candidate-observations.ts";
import {
  loadInheritedOptionsInputs,
  verifyInheritedInputSources,
  materializeInheritedOptions,
  verifyInheritedOwnedInputs,
  captureInheritedInputBytes,
  verifyInheritedOptionsOutput,
  compareInheritedOptionsRaw,
} from "./cold-typechecker-scale-candidate-originals.ts";
import { captureReadBufferCommand } from "./cold-typechecker-scale-read-buffer-command.ts";
import { coldScalePacketView } from "./cold-typechecker-scale-packet.ts";
import { nearestRank } from "./cold-typechecker-scale-paired-stats.ts";
import {
  verifyCandidateNativeLibraries,
  type CandidateSources,
} from "./cold-typechecker-scale-candidate-source.ts";

export interface CandidateBuild {
  sourceSha: string;
  treeSha: string;
  sourceRoot: string;
  binary: string;
  binarySha256: string;
  allocation: boolean;
  manifests: unknown;
  rustc: string;
  cargo: { args: string[]; version: string };
  compilerEnvironment: unknown;
  cargoArtifact: { features: string[] };
}
type Role = "base" | "head";
type NativeCalls = ReturnType<typeof collectNativePreflight>;
type Raw = { stdout: Buffer; stderr: Buffer; status: number | null };
type Sample = {
  pair: number;
  role: Role;
  order: number;
  directory: string;
  report: Awaited<ReturnType<typeof captureReadBufferCommand>>["report"];
};
const hash = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const nativeSha = "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70";
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
const read = (file: string) => JSON.parse(fs.readFileSync(file, "utf8"));
const orders = [
  ["base", "head"],
  ["head", "base"],
  ["head", "base"],
  ["base", "head"],
] as const;

/** Only the existing two owned app-cache roots are retired, never the authored project/provider. */
export function resetCandidateCache(common: string, createTmp = true) {
  assert.equal(fs.realpathSync(common), common);
  for (const name of ["xdg-cache", "tmp"]) {
    const root = path.join(common, name),
      stat = fs.lstatSync(root, { throwIfNoEntry: false });
    if (stat) {
      assert(stat.isDirectory() && !stat.isSymbolicLink());
      fs.rmSync(root, { recursive: true });
    }
  }
  if (createTmp) fs.mkdirSync(path.join(common, "tmp"));
}

export async function candidateNormalCommands(options: {
  provider: string;
  out: string;
  frozen: CandidateSources;
  normal: Record<Role, CandidateBuild>;
  allocation: Record<Role, { directory: string; build: CandidateBuild }>;
  assets: { before: Buffer; after: Buffer } | null;
  enter: (id: string) => void;
}) {
  const { provider, out, frozen, normal, enter } = options;
  const common = path.join(out, "work");
  fs.mkdirSync(common);
  const files = 2500 + coldScaleSupportScriptFiles;
  const corpus = prepareColdScale(path.join(common, "project"), provider, files, false);
  write(path.join(out, "corpus.json"), corpus);
  const forwarder = prepareNativeForwarder(out);
  const select = (build: CandidateBuild) =>
    coldScaleCommand({ provider, out: common, app: corpus.app, tool: "cli", binary: build.binary });
  const selected = select(normal.base),
    backend = selected.backend;
  assert.equal(hash(backend), nativeSha);
  write(path.join(out, "native-libraries.json"), verifyCandidateNativeLibraries(backend));
  assert.equal(select(normal.head).backend, backend);
  const originals = loadInheritedOptionsInputs();
  verifyInheritedInputSources(normal.base.sourceRoot, normal.head.sourceRoot, originals);
  const originalRoot = path.join(out, "inherited-project");
  const originalPackets = new Map<string, Raw>();
  for (const role of ["base", "head"] as const)
    for (const state of ["invalid", "typed"] as const) {
      const id = `inherited-${role}-${state}`,
        directory = path.join(out, id);
      materializeInheritedOptions(originalRoot, provider, originals, state);
      const before = verifyInheritedOwnedInputs(originalRoot, provider, originals, state);
      const beforeBytes = captureInheritedInputBytes(originalRoot, originals);
      write(path.join(out, id + ".inputs-before.json"), beforeBytes);
      assert(beforeBytes.every((row) => row.available));
      resetCandidateCache(common);
      enter(id);
      let result: ReturnType<typeof captureCandidateUntimed>;
      try {
        result = captureCandidateUntimed({
          directory,
          executable: normal[role].binary,
          imageSha256: normal[role].binarySha256,
          args: originals.args,
          cwd: originalRoot,
          env: { ...process.env, ...selected.command.env, CORSA_PATH: backend },
        });
      } finally {
        write(
          path.join(out, id + ".inputs-after.json"),
          captureInheritedInputBytes(originalRoot, originals),
        );
      }
      const packet: Raw = { status: result.status, stdout: result.stdout!, stderr: result.stderr! };
      const after = verifyInheritedOwnedInputs(originalRoot, provider, originals, state);
      assert.deepEqual(captureInheritedInputBytes(originalRoot, originals), beforeBytes);
      write(path.join(directory, "owned-inputs.json"), { before, after });
      assert.deepEqual(after, before);
      verifyInheritedOptionsOutput(
        packet.stdout,
        packet.stderr,
        packet.status,
        originalRoot,
        state,
        originals,
      );
      if (role === "base") originalPackets.set(state, packet);
      else compareInheritedOptionsRaw(originalPackets.get(state)!, packet);
    }
  const preflights = {} as Record<Role, { raw: Raw; native: NativeCalls; helper: Buffer }>;
  for (const role of ["base", "head"] as const) {
    resetCandidateCache(common);
    const selection = select(normal[role]),
      args = [...selection.command.args];
    assert.equal(selection.backend, backend);
    const at = args.indexOf("--corsa-path");
    assert(at >= 0 && args[at + 1] === backend);
    args[at + 1] = forwarder.executable;
    const capture = path.join(out, "native-preflight-" + role);
    fs.mkdirSync(capture);
    const id = "normal-preflight-" + role;
    enter(id);
    const result = captureCandidateUntimed({
      directory: path.join(out, id),
      executable: normal[role].binary,
      imageSha256: normal[role].binarySha256,
      args: [...args, "--show-virtual-ts", "--quiet"],
      cwd: corpus.app,
      env: {
        ...process.env,
        ...selection.command.env,
        VIZE_COLD_NATIVE: backend,
        VIZE_COLD_CAPTURE: capture,
        VIZE_COLD_PROJECT: path.join(common, "project"),
        VIZE_COLD_PROVIDER: provider,
      },
    });
    assert.equal(result.status, 0);
    verifyColdDiagnostics(result.stdout!.toString(), "cli", files, false);
    const value = JSON.parse(result.stdout!.toString());
    verifyStrictDebug(result.stderr!, value, corpus.app);
    preflights[role] = {
      raw: { status: result.status, stdout: result.stdout!, stderr: result.stderr! },
      native: collectNativePreflight(capture),
      helper: strictDebugParts(result.stderr!, value, corpus.app).helper,
    };
  }
  let helperTransition: { before: Buffer; after: Buffer } | undefined;
  if (frozen.kind === "type-work") {
    assert(options.assets);
    write(
      path.join(out, "asset-transition.json"),
      verifyComponentChecksTransition(
        preflights.base.raw,
        preflights.head.raw,
        corpus.app,
        options.assets,
      ),
    );
    const main = preflights.base.native.find((call) => call.request.kind === "main-cli")!;
    assert(main);
    const rows = read(path.join(main.directory, "inputs-before.json")).rows as {
      path: string;
      bytes: number;
      sha256: string;
    }[];
    const helper = rows.filter((row) => path.basename(row.path) === "__vize_helpers.d.ts");
    assert.equal(helper.length, 1);
    helperTransition = materializedCandidateHelpers(
      { before: preflights.base.helper, after: preflights.head.helper },
      helper[0],
    );
  } else compareInheritedOptionsRaw(preflights.base.raw, preflights.head.raw);
  const expected = {
    executable: backend,
    sha256: nativeSha,
    forwarder: forwarder.executable,
    project: path.join(common, "project"),
    provider,
    materializedRoot: path.join(common, "xdg-cache"),
    authoredCwd: corpus.app,
  };
  const nativeProof = compareNativePreflights(preflights.base.native, preflights.head.native, {
    ...expected,
    materializedHelperBodies: helperTransition,
  });
  write(path.join(out, "normal-native-proof.json"), nativeProof);
  const allocationPackets = {} as Record<Role, Raw>,
    allocationReports = [];
  let routePopulation: unknown;
  for (const role of ["base", "head"] as const) {
    const observation = options.allocation[role],
      build = observation.build,
      source = read(path.join(observation.directory, "source.json"));
    assert.equal(source.productSource, normal[role].sourceSha);
    assert.equal(source.instrumentedSource, build.sourceSha);
    assert.equal(build.allocation, true);
    assert.equal(hash(build.binary), build.binarySha256);
    assert.deepEqual(build.manifests, normal[role].manifests);
    assert.equal(build.rustc, normal[role].rustc);
    assert.equal(build.cargo.version, normal[role].cargo.version);
    assert.deepEqual(build.compilerEnvironment, normal[role].compilerEnvironment);
    assert.deepEqual(build.cargo.args, [...normal[role].cargo.args, "--features", "profiling"]);
    resetCandidateCache(common);
    const selection = select(build),
      args = [...selection.command.args],
      at = args.indexOf("--corsa-path");
    assert(at >= 0 && args[at + 1] === backend);
    args[at + 1] = forwarder.executable;
    const capture = path.join(out, "native-allocation-" + role);
    fs.mkdirSync(capture);
    const profile = path.join(common, "allocation-profile.json"),
      child = path.join(common, "child-allocation.json"),
      route = path.join(common, "route-memory.json");
    for (const file of [profile, child, route]) assert(!fs.existsSync(file));
    const id = "allocation-" + role,
      directory = path.join(out, id);
    enter(id);
    const result = captureCandidateUntimed({
      directory,
      executable: build.binary,
      imageSha256: build.binarySha256,
      args: [...args, "--profile-json", profile],
      cwd: corpus.app,
      env: {
        ...process.env,
        ...selection.command.env,
        VIZE_COLD_NATIVE: backend,
        VIZE_COLD_CAPTURE: capture,
        VIZE_COLD_PROJECT: path.join(common, "project"),
        VIZE_COLD_PROVIDER: provider,
        VIZE_CHILD_ALLOCATION_OBSERVATION: child,
        ...(frozen.kind === "route-move" ? { VIZE_ROUTE_MEMORY_OBSERVATION: route } : {}),
      },
    });
    // Persist all overlay products BEFORE any profile/census/output verdict.
    for (const [name, file] of [
      ["profile.json", profile],
      ["child-allocation.json", child],
      ...(frozen.kind === "route-move" ? [["route-memory.json", route]] : []),
    ]) {
      if (fs.existsSync(file)) {
        fs.copyFileSync(file, path.join(directory, name));
        fs.unlinkSync(file);
      }
    }
    assert.equal(result.status, 0);
    const calls = collectNativePreflight(capture);
    compareNativePreflights(preflights[role].native, calls, { ...expected, persistClasses: false });
    const main = calls.find((call) => call.request.kind === "main-cli")!;
    assert(main);
    const invocation = read(path.join(main.directory, "invocation.json"));
    const childReport = verifyChildAllocationObservation(
      read(path.join(directory, "child-allocation.json")),
      {
        vizePid: result.pid!,
        carrierPid: invocation.observerPid,
        nativePid: Number(main.status.invokedPid),
        status: Number(main.status.status),
        stdout: main.stdout,
        stderr: main.stderr,
      },
    );
    const profileReport = verifyCandidateAllocationProfile(
      read(path.join(directory, "profile.json")),
    );
    let routeReport: ReturnType<typeof verifyRouteCensus> | null = null;
    if (frozen.kind === "route-move") {
      routeReport = verifyRouteCensus(read(path.join(directory, "route-memory.json")), corpus.app);
      if (routePopulation === undefined) routePopulation = routeReport.raw;
      else
        assert.deepEqual(
          routeReport.raw,
          routePopulation,
          "full actual route input census changed",
        );
    }
    allocationPackets[role] = {
      status: result.status,
      stdout: result.stdout!,
      stderr: result.stderr!,
    };
    write(path.join(directory, "verification.json"), {
      childReport,
      profileReport,
      routeReport,
      outputParity: "pending first SAME-ROLE normal primary; no diagnostic filtering",
    });
    allocationReports.push({ role, childReport, profileReport, routeReport });
  }
  if (
    frozen.kind === "route-move" &&
    allocationReports.some((row) => row.routeReport?.eligibleInitializedBindingWork !== true)
  )
    return {
      eligible: false as const,
      allocationReports,
      reason:
        "empty initialized route binding input; outer Vec capacity and retained memory impact remain unknown",
    };
  const samples: Sample[] = [];
  let completeRaw: Raw | undefined;
  for (const [pairIndex, order] of orders.entries())
    for (const [position, role] of order.entries()) {
      const id = `primary-${pairIndex + 1}-${role}`;
      enter(id);
      const observed = await captureReadBufferCommand({
        provider,
        runRoot: common,
        directory: path.join(out, id),
        files,
        binary: normal[role].binary,
      });
      assert.equal(observed.report.corpusSha256, corpus.corpusSha256);
      const raw = { status: 0, stdout: observed.stdout, stderr: observed.stderr };
      compareInheritedOptionsRaw(allocationPackets[role], raw); // Full primary-argv parity closes here, after this raw measured command.
      if (completeRaw) compareInheritedOptionsRaw(completeRaw, raw);
      else completeRaw = raw;
      samples.push({
        pair: pairIndex + 1,
        role,
        order: position + 1,
        directory: path.join(out, id),
        report: observed.report,
      });
    }
  let invalidPacket: unknown;
  for (const role of ["base", "head"] as const) {
    const id = "invalid-" + role;
    enter(id);
    const directory = path.join(out, id);
    captureColdScale({
      provider,
      out: directory,
      files,
      tool: "cli",
      invalid: true,
      binary: normal[role].binary,
    });
    const census = read(path.join(directory, "corpus.json"));
    const packet = coldScalePacketView(read(path.join(directory, "stdout.txt")), {
      ...census,
      root: path.join(directory, "project"),
    });
    if (invalidPacket === undefined) invalidPacket = packet;
    else assert.deepEqual(packet, invalidPacket);
  }
  for (const tool of ["mapper", "vue-tsc"])
    for (const invalid of [false, true]) {
      const id = `${invalid ? "invalid" : "valid"}-${tool}`;
      enter(id);
      captureColdScale({ provider, out: path.join(out, id), files, tool, invalid });
    }
  const describe = (select: (report: (typeof samples)[number]["report"]) => number) => {
    const values = (role: Role) =>
      samples.filter((row) => row.role === role).map((row) => select(row.report));
    return {
      base: values("base"),
      head: values("head"),
      baseP50: nearestRank(values("base"), 0.5),
      headP50: nearestRank(values("head"), 0.5),
      scope: "four pairs descriptive only, no significance or precise tail claim",
    };
  };
  return {
    eligible: true as const,
    common,
    corpus,
    backend,
    helperTransition,
    preflights,
    nativeProof,
    allocationReports,
    samples,
    statistics: {
      wallMs: describe((row) => row.wallMs),
      cpuMs: describe((row) => row.cpu.totalCpuMs),
      gnuChildHwm: describe((row) => row.peakRssBytes),
      rootObservedRss: describe((row) => row.rootPeakBytes),
      concurrentObservedTreeRss: describe((row) => row.concurrentObservedTreePeakBytes),
    },
  };
}
