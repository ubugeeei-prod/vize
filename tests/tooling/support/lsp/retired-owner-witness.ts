import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { stripVTControlCharacters } from "node:util";
import {
  isReaped,
  observeReaped,
} from "../../../performance/support/warm-type-backed-processes.ts";
import { offsetToPosition } from "./assertions.ts";
import { LspRequestError, LspSession } from "./session.ts";

type Witness = {
  pid: number;
  birth_ticks: number;
  parent: number;
  executable: string;
  executableSha256: string;
  commandBase64: string;
  commandArguments: string[];
  wholeStatBefore: string;
  wholeStatAfter: string;
};
type ChildObserver = (parent: number, executable: string) => Witness[];
type OwnerGraph = Array<{
  owner: string;
  role: "editor-lsp" | "project-api";
  proxy: Witness;
  native: Witness;
}>;
type PhysicalStart = {
  phase: "before-protocol-forwarding";
  invocation: "editor-lsp" | "project-api" | "version-probe";
  args: string[];
  proxy: Witness;
  native: Witness;
};
type CustodyJournal = {
  kind: "retired" | "active" | "recovered";
  owner?: "old" | "new";
  rawLine: string;
  json: string;
  receipt: Record<string, unknown>;
};
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const pause = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const fixture = new URL(
  "../../../_fixtures/differential/lsp/canceled-native-hover/",
  import.meta.url,
);

function actualProbe(identity: Witness) {
  try {
    const wholeStat = fs.readFileSync(`/proc/${identity.pid}/stat`, "utf8");
    const fields = wholeStat
      .slice(wholeStat.lastIndexOf(") ") + 2)
      .trim()
      .split(/\s+/u);
    assert.match(fields[19], /^\d+$/u);
    return {
      ...identity,
      current: { birth_ticks: Number(fields[19]), state: fields[0], wholeStat },
    };
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return { ...identity, current: null };
    throw error;
  }
}

function object(value: unknown, label: string): Record<string, unknown> {
  assert.ok(
    value !== null && typeof value === "object" && !Array.isArray(value),
    `${label} must be a complete object`,
  );
  return value as Record<string, unknown>;
}

function exactKeys(value: Record<string, unknown>, keys: string[]) {
  assert.deepEqual(
    Object.keys(value).sort(),
    keys.toSorted(),
    "the frozen custody schema must stay complete",
  );
}

function rawErrors(value: unknown): string[] {
  assert.ok(
    Array.isArray(value) && value.every((entry) => typeof entry === "string" && entry.length > 0),
  );
  return value;
}

function physicalIdentity(value: unknown): Witness {
  const identity = object(value, "physical identity before protocol forwarding");
  exactKeys(identity, [
    "pid",
    "birth_ticks",
    "parent",
    "executable",
    "executableSha256",
    "commandBase64",
    "commandArguments",
    "wholeStatBefore",
    "wholeStatAfter",
  ]);
  for (const field of ["pid", "birth_ticks", "parent"])
    assert.ok(Number.isSafeInteger(identity[field]) && Number(identity[field]) > 0);
  for (const field of [
    "executable",
    "executableSha256",
    "commandBase64",
    "wholeStatBefore",
    "wholeStatAfter",
  ])
    assert.equal(typeof identity[field], "string");
  assert.match(identity.executableSha256 as string, /^[a-f0-9]{64}$/u);
  assert.ok(Array.isArray(identity.commandArguments));
  assert.ok(identity.commandArguments.every((arg) => typeof arg === "string"));
  const command = Buffer.from(identity.commandBase64 as string, "base64");
  assert.equal(command.toString("base64"), identity.commandBase64);
  assert.equal(command.at(-1), 0);
  assert.deepEqual(command.toString("utf8").split("\0").filter(Boolean), identity.commandArguments);
  for (const field of ["wholeStatBefore", "wholeStatAfter"]) {
    const stat = identity[field] as string;
    assert.equal(Number(stat.slice(0, stat.indexOf(" ("))), identity.pid);
    const fields = stat
      .slice(stat.lastIndexOf(") ") + 2)
      .trim()
      .split(/\s+/u);
    assert.match(fields[19], /^\d+$/u);
    assert.equal(Number(fields[19]), identity.birth_ticks);
    assert.equal(Number(fields[1]), identity.parent);
    assert.notEqual(fields[0], "Z", "birth evidence must observe a live process");
  }
  return identity as unknown as Witness;
}

function physicalStarts(
  gate: { root: string; proxy: string },
  images: {
    vizePid: number;
    node: string;
    nodeSha256: string;
    native: string;
    nativeSha256: string;
  },
): PhysicalStart[] {
  const lines = fs.readFileSync(path.join(gate.root, "physical-starts.jsonl"), "utf8");
  assert.ok(lines.endsWith("\n"), "the complete birth census must have no partial row");
  return lines
    .trimEnd()
    .split("\n")
    .map((line) => {
      const row = object(JSON.parse(line), "physical spawn before protocol forwarding");
      exactKeys(row, ["phase", "invocation", "args", "proxy", "native"]);
      assert.equal(row.phase, "before-protocol-forwarding");
      assert.ok(Array.isArray(row.args) && row.args.every((arg) => typeof arg === "string"));
      const args: string[] = row.args;
      const invocation =
        args.includes("--lsp") && args.includes("--stdio") && !args.includes("--api")
          ? "editor-lsp"
          : args.includes("--api") && !args.includes("--lsp")
            ? "project-api"
            : args.length === 1 && args[0] === "--version"
              ? "version-probe"
              : "unknown";
      assert.notEqual(
        invocation,
        "unknown",
        "every invocation must be classified from actual argv",
      );
      assert.equal(row.invocation, invocation);
      if (invocation === "editor-lsp") assert.deepEqual(args, ["--lsp", "--stdio"]);
      if (invocation === "project-api") {
        assert.equal(args.length, 4);
        assert.deepEqual(args.slice(0, 3), ["--api", "--async", "--cwd"]);
        assert.ok(path.isAbsolute(args[3]));
      }
      const proxy = physicalIdentity(row.proxy);
      const native = physicalIdentity(row.native);
      assert.equal(proxy.parent, images.vizePid);
      assert.equal(native.parent, proxy.pid);
      assert.equal(proxy.executable, images.node);
      assert.equal(proxy.executableSha256, images.nodeSha256);
      assert.equal(native.executable, images.native);
      assert.equal(native.executableSha256, images.nativeSha256);
      assert.equal(proxy.commandArguments[0], "node");
      assert.deepEqual(proxy.commandArguments.slice(1), [gate.proxy, ...args]);
      assert.deepEqual(native.commandArguments, [images.native, ...args]);
      assert.notEqual(proxy.pid, native.pid);
      return {
        phase: "before-protocol-forwarding",
        invocation: row.invocation as PhysicalStart["invocation"],
        args,
        proxy,
        native,
      };
    });
}

/** Keep complete source event lines; JSON is the sole final trace field. */
function custodyJournal(stderr: string): CustodyJournal[] {
  const journal: CustodyJournal[] = [];
  const events = [
    {
      kind: "recovered" as const,
      target: "vize_canon::lsp_client::startup_control",
      message: "recovered native owner terminal custody",
    },
    {
      kind: "retired" as const,
      target: "vize_maestro::server::state::corsa_shutdown::retired",
      message: "retired native owner terminal custody",
    },
    {
      kind: "active" as const,
      target: "vize_canon::corsa_bridge::bridge::shutdown",
      message: "native active owner shutdown custody",
    },
  ];
  for (const rawLine of stderr.split("\n")) {
    const line = stripVTControlCharacters(rawLine);
    for (const event of events) {
      const marker = `${event.target}: ${event.message} receipt=`;
      const start = line.indexOf(marker);
      if (start < 0) continue;
      assert.match(line.slice(0, start), /\bINFO\s+$/u);
      const json = line.slice(start + marker.length).trim();
      assert.ok(json.startsWith("{"), "the custody event must retain its complete JSON tail");
      journal.push({
        kind: event.kind,
        rawLine,
        json,
        receipt: object(JSON.parse(json), "custody receipt"),
      });
    }
  }
  return journal;
}

function assertCustodyJournal(
  journal: CustodyJournal[],
  owners: { old: OwnerGraph; new: OwnerGraph },
  warmLive: { old: OwnerGraph; new: OwnerGraph },
  mode: "held" | "killed",
  faultNativePid: number | null,
) {
  const livePids = {
    old: new Set(warmLive.old.map((entry) => entry.proxy.pid)),
    new: new Set(warmLive.new.map((entry) => entry.proxy.pid)),
  };
  for (const owner of ["old", "new"] as const) {
    assert.equal(owners[owner].length, 4);
    assert.equal(warmLive[owner].length, 2);
    assert.equal(livePids[owner].size, 2);
    assert.deepEqual(warmLive[owner].map((entry) => entry.role).sort(), [
      "editor-lsp",
      "project-api",
    ]);
    const early = owners[owner].filter((entry) => !livePids[owner].has(entry.proxy.pid));
    assert.equal(early.length, 2);
    assert.deepEqual(early.map((entry) => entry.role).sort(), ["editor-lsp", "project-api"]);
    for (const current of warmLive[owner]) {
      const birth = owners[owner].find((entry) => entry.proxy.pid === current.proxy.pid);
      assert.ok(
        birth,
        "every independently observed current owner must have its physical birth record",
      );
      assert.equal(current.role, birth.role);
      for (const [live, born] of [
        [current.proxy, birth.proxy],
        [current.native, birth.native],
      ])
        for (const key of [
          "pid",
          "birth_ticks",
          "parent",
          "executable",
          "executableSha256",
          "commandBase64",
          "commandArguments",
        ] as const)
          assert.deepEqual(live[key], born[key]);
    }
  }
  const closed = new Set<string>();
  const terminals = new Set<number>();
  const sequences = { old: new Set<number>(), new: new Set<number>() };
  const published = { old: 0, new: 0 };
  const observed = { old: 0, new: 0 };
  const historicalFailures: CustodyJournal[] = [];
  const historicalError = "LSP close: process is closed: jsonrpc connection";
  // The Vue-only config has no ordinary TS member: project_identity::startup_member
  // rejects editor bootstrap; configured_project records and closes that failed
  // construction before shared_transport uses the independent API. Reaping is clean,
  // but the historical construction failure remains raw and is never called clean.
  const constructorError = "Native startup configuration has no existing absolute member";
  for (const event of journal) {
    const receipt = event.receipt;
    const recovered = event.kind === "recovered";
    exactKeys(
      receipt,
      recovered
        ? ["children", "retirementError", "observedCount", "publishedCount"]
        : [
            "custody_complete",
            "raw_errors",
            "expectedOwnedPids",
            "children",
            "attachedBindings",
            "missingAttachmentBinding",
            "observedCount",
            "publishedCount",
            "pendingCleanupHandles",
            "pendingStartupCustody",
          ],
    );
    assert.ok(Array.isArray(receipt.children));
    const children = receipt.children.map((child) => object(child, "owned child"));
    const pids = children.map((child) => child.expectedPid);
    assert.ok(pids.every((pid) => Number.isSafeInteger(pid) && Number(pid) > 0));
    assert.equal(new Set(pids).size, pids.length);
    assert.ok(
      Number.isSafeInteger(receipt.observedCount) && Number.isSafeInteger(receipt.publishedCount),
    );
    assert.ok((receipt.observedCount as number) >= 0 && (receipt.publishedCount as number) >= 0);
    if (recovered) {
      assert.ok(
        children.length > 0,
        "recovery publication must retain actual complete terminal rows",
      );
      assert.ok(
        receipt.retirementError === null ||
          (typeof receipt.retirementError === "string" && receipt.retirementError.length > 0),
      );
    } else {
      assert.equal(
        receipt.custody_complete,
        true,
        "missing or incomplete process custody cannot permit a successful shutdown",
      );
      assert.equal(receipt.pendingCleanupHandles, 0);
      assert.equal(receipt.pendingStartupCustody, false);
      assert.equal(receipt.missingAttachmentBinding, false);
      assert.deepEqual(
        receipt.attachedBindings,
        [],
        "this fixture observes owned SDK proxy processes, not an API attachment",
      );
      assert.ok(Array.isArray(receipt.expectedOwnedPids));
      assert.deepEqual(
        receipt.expectedOwnedPids.toSorted((a, b) => a - b),
        pids.toSorted((a, b) => Number(a) - Number(b)),
      );
      if (receipt.observedCount === 0) {
        assert.equal(receipt.publishedCount, 0);
        assert.deepEqual(children, []);
        assert.deepEqual(rawErrors(receipt.raw_errors), []);
        continue; // Empty owners cannot replace either physically witnessed owner's final closure.
      }
    }
    const candidates = Object.entries(owners).filter(
      ([, graph]) =>
        (recovered || (event.kind === "retired" ? graph === owners.old : graph === owners.new)) &&
        pids.every((pid) => graph.some((entry) => entry.proxy.pid === pid)),
    );
    assert.equal(
      candidates.length,
      1,
      "every reported owned PID must match the physically witnessed API/editor proxy graph",
    );
    const [ownerName, graph] = candidates[0];
    assert.ok(ownerName === "old" || ownerName === "new");
    const owner = ownerName;
    event.owner = owner;
    assert.equal(receipt.publishedCount, published[owner]);
    const frontier = Math.max(
      observed[owner],
      ...children.map((child) => Number(child.spawnSequence)),
    );
    assert.equal(receipt.observedCount, recovered ? frontier : graph.length);
    observed[owner] = Number(receipt.observedCount);
    if (recovered) {
      assert.ok(!closed.has(owner), "recovery cannot publish after the owner's final closure");
      assert.ok(published[owner] + children.length <= observed[owner]);
    } else {
      assert.equal(published[owner] + children.length, graph.length);
      closed.add(owner);
    }
    const failedOld = mode === "killed" && owner === "old";
    const errors = recovered ? [] : rawErrors(receipt.raw_errors);
    const faultInReceipt =
      failedOld &&
      children.some((child) =>
        graph.some(
          (entry) => entry.proxy.pid === child.expectedPid && entry.native.pid === faultNativePid,
        ),
      );
    // The retained 32af packet has successful constructor cleanup and one
    // SDK close error aggregated with that exact faulted child's final row.
    if (recovered) assert.equal(receipt.retirementError, null);
    assert.deepEqual(errors, faultInReceipt ? [historicalError] : []);
    if (faultInReceipt) historicalFailures.push(event);
    for (const child of children) {
      const expected = graph.find((entry) => entry.proxy.pid === child.expectedPid);
      assert.ok(expected);
      assert.ok(
        !terminals.has(expected.proxy.pid),
        "each physical owner terminal must appear exactly once",
      );
      terminals.add(expected.proxy.pid);
      assert.ok(
        Number.isSafeInteger(child.spawnSequence) &&
          Number(child.spawnSequence) >= 1 &&
          Number(child.spawnSequence) <= graph.length,
      );
      const sequence = Number(child.spawnSequence);
      assert.equal(
        livePids[owner].has(expected.proxy.pid),
        sequence >= 3,
        "typed lifecycle phase must match independently observed warm-live ownership",
      );
      assert.ok(!sequences[owner].has(sequence));
      sequences[owner].add(sequence);
      assert.equal(expected.role, sequence === 1 || sequence === 4 ? "editor-lsp" : "project-api");
      if (sequence <= 2) {
        assert.equal(recovered, true, "initial constructor custody precedes current owner closure");
        assert.equal(receipt.observedCount, sequence);
        assert.equal(receipt.publishedCount, sequence - 1);
        assert.equal(children.length, 1);
        assert.equal(
          receipt.retirementError,
          null,
          "successful cleanup does not erase failed construction",
        );
      } else {
        assert.equal(recovered, false);
        assert.equal(receipt.observedCount, 4);
        assert.equal(receipt.publishedCount, 2);
        assert.deepEqual(
          children.map((entry) => Number(entry.spawnSequence)).sort((a, b) => a - b),
          [3, 4],
        );
      }
      exactKeys(child, [
        "spawnSequence",
        "expectedPid",
        "role",
        "terminal",
        "raw_errors",
        "close_error",
        "stderrCustody",
      ]);
      assert.notEqual(
        child.expectedPid,
        expected.native.pid,
        "SDK custody names the Node proxy; the actual pinned native leaf is witnessed separately",
      );
      assert.equal(child.role, expected.role === "project-api" ? "api" : "editor-lsp");
      assert.equal(child.close_error, null);
      const killed = failedOld && expected.native.pid === faultNativePid;
      if (killed) {
        assert.equal(expected.role, "editor-lsp");
        assert.equal(sequence, 4);
      }
      const status = `ExitStatus(unix_wait_status(${killed ? 9 : 0}))`;
      assert.deepEqual(child.terminal, {
        pid: expected.proxy.pid,
        status,
        exitCode: killed ? null : 0,
        signal: killed ? 9 : null,
        killRequested: false,
        timedOut: false,
        reaped: true,
      });
      const childErrors = rawErrors(child.raw_errors);
      assert.deepEqual(
        childErrors,
        sequence === 1 ? [constructorError] : killed ? [historicalError] : [],
      );
      if (!recovered)
        for (const error of childErrors)
          assert.ok(
            errors.includes(error),
            "the complete top-level journal must retain every child error verbatim",
          );
      const stderr = object(child.stderrCustody, "stderr custody");
      if (expected.role === "project-api")
        assert.deepEqual(stderr, { mode: "inherited-server-stderr" });
      else {
        exactKeys(stderr, ["mode", "receipts"]);
        assert.equal(stderr.mode, "observed-editor");
        assert.ok(Array.isArray(stderr.receipts));
        assert.equal(stderr.receipts.length, 1);
        const row = object(stderr.receipts[0], "complete editor stderr receipt");
        exactKeys(row, [
          "pid",
          "completed",
          "bytes",
          "stderrComplete",
          "stderrError",
          "processError",
          "status",
          "timedOut",
          "killRequested",
          "reaped",
        ]);
        assert.ok(
          Array.isArray(row.bytes) &&
            row.bytes.every((byte) => Number.isInteger(byte) && byte >= 0 && byte <= 255),
          "every native stderr byte must remain lossless",
        );
        assert.deepEqual(row, {
          pid: expected.proxy.pid,
          completed: true,
          bytes: killed ? row.bytes : [],
          stderrComplete: true,
          stderrError: null,
          processError: null,
          status,
          timedOut: false,
          killRequested: false,
          reaped: true,
        });
      }
    }
    published[owner] += children.length;
  }
  assert.deepEqual(
    [...closed].sort(),
    ["new", "old"],
    "both retired and current final closed-custody receipts remain mandatory after recovery pruning",
  );
  assert.deepEqual(
    [...terminals].sort((a, b) => a - b),
    [...owners.old, ...owners.new].map((entry) => entry.proxy.pid).sort((a, b) => a - b),
    "the complete recovery/final union must contain every exact physical proxy PID once",
  );
  for (const owner of ["old", "new"] as const) {
    assert.equal(observed[owner], owners[owner].length);
    assert.equal(published[owner], owners[owner].length);
    assert.deepEqual(
      [...sequences[owner]].sort((a, b) => a - b),
      Array.from({ length: owners[owner].length }, (_, index) => index + 1),
    );
  }
  if (mode === "killed")
    assert.ok(
      historicalFailures.length === 1,
      "the killed old owner's exact SDK close failure must appear once in its final custody context",
    );
}

function assertWarmLivePhaseControl(
  journal: CustodyJournal[],
  owners: { old: OwnerGraph; new: OwnerGraph },
  warmLive: { old: OwnerGraph; new: OwnerGraph },
  mode: "held" | "killed",
  faultNativePid: number | null,
) {
  const derived = structuredClone(journal);
  const constructorEvent = derived.find(
    (event) =>
      event.owner === "new" &&
      event.kind === "recovered" &&
      (event.receipt.children as Record<string, unknown>[]).some(
        (child) => child.spawnSequence === 1,
      ),
  );
  const currentEvent = derived.find(
    (event) =>
      event.owner === "new" &&
      event.kind === "active" &&
      (event.receipt.children as Record<string, unknown>[]).some(
        (child) => child.spawnSequence === 4,
      ),
  );
  assert.ok(constructorEvent && currentEvent);
  const constructors = constructorEvent.receipt.children as Record<string, unknown>[];
  const currents = currentEvent.receipt.children as Record<string, unknown>[];
  const initialIndex = constructors.findIndex((child) => child.spawnSequence === 1);
  const currentIndex = currents.findIndex((child) => child.spawnSequence === 4);
  const initialRow = constructors[initialIndex];
  const currentRow = currents[currentIndex];
  const changedInitial = { ...currentRow, spawnSequence: 1, raw_errors: initialRow.raw_errors };
  const changedCurrent = { ...initialRow, spawnSequence: 4, raw_errors: currentRow.raw_errors };
  constructors[initialIndex] = changedInitial;
  currents[currentIndex] = changedCurrent;
  currentEvent.receipt.expectedOwnedPids = currents.map((child) => child.expectedPid);
  const expectedRejection =
    "typed lifecycle phase must match independently observed warm-live ownership";
  let actualRejection = "";
  assert.throws(
    () => {
      try {
        assertCustodyJournal(derived, owners, warmLive, mode, faultNativePid);
      } catch (error) {
        actualRejection = error instanceof Error ? error.message : String(error);
        throw error;
      }
    },
    { message: new RegExp(expectedRejection, "u") },
  );
  return {
    kind: "derived-input oracle negative control, not runtime process evidence",
    owner: "new",
    sourceJournalUnmodified: true,
    mutations: [
      { before: initialRow, after: changedInitial },
      { before: currentRow, after: changedCurrent },
    ],
    expectedRejection,
    actualRejection,
  };
}

export async function retiredOwnerWitness(
  children: ChildObserver,
  mode: "held" | "killed" = "held",
) {
  assert.equal(
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD,
    "1",
    "complete source-bound raw wire/stderr capture is required",
  );
  assert.ok(process.env.VIZE_LSP_BIN, "this proof uses the action's prebuilt attested CLI");
  const manifest = createRequire(import.meta.url).resolve(
    "@typescript/typescript-linux-x64/package.json",
  );
  const manifestBytes = fs.readFileSync(manifest);
  const metadata = JSON.parse(manifestBytes.toString("utf8"));
  assert.equal(metadata.version, "7.0.2");
  const native = fs.realpathSync(path.join(path.dirname(manifest), "lib/tsc"));
  const nativeSha256 = digest(fs.readFileSync(native));
  const node = fs.realpathSync(process.execPath);
  const nodeSha256 = digest(fs.readFileSync(node));
  const provider = process.env.VIZE_EDITOR_CONFIG_VUE_ROOT;
  assert.ok(provider);
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-retired-inflight-"));
  fs.mkdirSync(path.join(workspace, "node_modules"));
  fs.mkdirSync(path.join(workspace, "src"));
  fs.symlinkSync(fs.realpathSync(provider), path.join(workspace, "node_modules/vue"), "dir");
  const source = fs.readFileSync(new URL("SlotAuthoring.vue.txt", fixture), "utf8");
  fs.writeFileSync(path.join(workspace, "src/SlotAuthoring.vue"), source);
  fs.copyFileSync(new URL("tsconfig.json", fixture), path.join(workspace, "tsconfig.json"));
  const proxyBytes = fs.readFileSync(new URL("held-native.js", fixture));
  const gates = ["old", "new"].map((role) => {
    const root = path.join(workspace, role);
    fs.mkdirSync(root);
    fs.writeFileSync(path.join(root, "package.json"), '{"type":"commonjs"}');
    const proxy = path.join(root, "tsgo.js");
    fs.writeFileSync(proxy, proxyBytes, { mode: 0o755 });
    fs.writeFileSync(
      path.join(root, "gate.json"),
      JSON.stringify({ runtime: native, root, physicalWitness: true }),
    );
    return { role, root, proxy };
  });
  const config = (proxy: string) => ({
    typeChecker: { corsaPath: proxy, checkFallthroughAttrs: false },
    lsp: { typecheck: true, lint: false, hover: true },
  });
  const configPath = path.join(workspace, "vize.config.json");
  fs.writeFileSync(configPath, JSON.stringify(config(gates[0].proxy)));
  const session = new LspSession();
  const events: Array<Record<string, unknown>> = [];
  const record = (name: string, value: unknown = null) =>
    events.push({ name, at: Date.now(), value });
  const observation: Record<string, unknown> = {
    mode,
    workspace,
    source,
    native,
    nativeSha256,
    nativeBytes: fs.statSync(native).size,
    nativeManifestBytesBase64: manifestBytes.toString("base64"),
    nativeManifestSha256: digest(manifestBytes),
    proxyBytesBase64: proxyBytes.toString("base64"),
    proxySha256: digest(proxyBytes),
    node,
    nodeSha256,
    vizePid: session.processId,
    events,
    configurations: gates.map((gate) => config(gate.proxy)),
    tsconfigBytesBase64: fs.readFileSync(path.join(workspace, "tsconfig.json")).toString("base64"),
  };
  const responses: unknown[] = [];
  session.responseObservers.push((message) => responses.push(message));
  const physicalByOwner = new Map<string, PhysicalStart[]>();
  const census = (gate: (typeof gates)[number]): OwnerGraph => {
    const starts = physicalStarts(gate, {
      vizePid: session.processId,
      node,
      nodeSha256,
      native,
      nativeSha256,
    });
    physicalByOwner.set(gate.role, starts);
    observation[`${gate.role}PhysicalStarts`] = starts;
    const sdk = starts.filter((start) => start.invocation !== "version-probe");
    assert.equal(
      sdk.length,
      4,
      "each actual initialized owner has four independently witnessed SDK spawns",
    );
    assert.equal(sdk.filter((start) => start.invocation === "project-api").length, 2);
    assert.equal(sdk.filter((start) => start.invocation === "editor-lsp").length, 2);
    return sdk.map((start) => ({
      owner: gate.role,
      role: start.invocation as "editor-lsp" | "project-api",
      proxy: start.proxy,
      native: start.native,
    }));
  };
  const graph = (gate: (typeof gates)[number], born: OwnerGraph): OwnerGraph => {
    const proxies = children(session.processId, node).filter((child) =>
      child.commandArguments.includes(gate.proxy),
    );
    assert.equal(proxies.length, 2, `${gate.role} owner must expose actual API and LSP proxies`);
    return proxies.map((proxy) => {
      const nativeChildren = children(proxy.pid, native);
      assert.equal(
        nativeChildren.length,
        1,
        "each proxy owns one physically verified pinned native child",
      );
      const image = nativeChildren[0];
      assert.equal(image.executableSha256, nativeSha256);
      const role = proxy.commandArguments.includes("--lsp") ? "editor-lsp" : "project-api";
      assert.ok(proxy.commandArguments.includes(role === "editor-lsp" ? "--lsp" : "--api"));
      assert.ok(image.commandArguments.includes(role === "editor-lsp" ? "--lsp" : "--api"));
      const birth = born.find((entry) => entry.proxy.pid === proxy.pid);
      assert.ok(birth, "a live SDK proxy must already have its before-forwarding birth witness");
      assert.equal(birth.role, role);
      for (const [current, original] of [
        [proxy, birth.proxy],
        [image, birth.native],
      ])
        for (const key of [
          "pid",
          "birth_ticks",
          "parent",
          "executable",
          "executableSha256",
          "commandBase64",
          "commandArguments",
        ] as const)
          assert.deepEqual(current[key], original[key]);
      return { ...birth, proxy, native: image };
    });
  };
  let success = false;
  let polling = true;
  try {
    observation.initialize = await session.initialize(workspace, {
      editor: true,
      hover: true,
      typecheck: true,
      lint: false,
    });
    const uri = pathToFileURL(path.join(workspace, "src/SlotAuthoring.vue")).href;
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    const member = source.lastIndexOf("names.current") + "names.".length;
    const params = { textDocument: { uri }, position: offsetToPosition(source, member + 1) };
    const expected = {
      contents: {
        kind: "markdown",
        value:
          '```typescript\n(property) current: "summary"\n```\n**Primary** invoice slot shown in the summary.',
      },
      range: { start: offsetToPosition(source, member), end: offsetToPosition(source, member + 7) },
    };
    observation.warmQuery = {
      method: "textDocument/hover",
      params,
      result: await session.request("textDocument/hover", params, 10000),
    };
    assert.deepEqual((observation.warmQuery as { result: unknown }).result, expected);
    const allOldGraph = census(gates[0]);
    observation.oldFullOwnerGraph = allOldGraph;
    const oldGraph = graph(gates[0], allOldGraph);
    observation.oldOwnerGraph = oldGraph;
    assert.deepEqual(oldGraph.map((entry) => entry.role).sort(), ["editor-lsp", "project-api"]);
    fs.writeFileSync(path.join(gates[0].root, "arm"), "");
    let heldId = 0;
    const held = session
      .request("textDocument/hover", params, 10000, (id) => {
        heldId = id;
        return [];
      })
      .then(
        (result) => ({ result }),
        (error: unknown) => ({ error }),
      );
    const enteredDeadline = Date.now() + 5_000;
    while (!fs.existsSync(path.join(gates[0].root, "entered"))) {
      assert.ok(
        Date.now() < enteredDeadline,
        "the actual native hover response must enter the byte-preserving gate",
      );
      await pause(5);
    }
    record("old-native-response-held");
    const nativeRequest = JSON.parse(
      fs.readFileSync(path.join(gates[0].root, "held-request.json"), "utf8"),
    );
    const nativeResponse = JSON.parse(
      fs.readFileSync(path.join(gates[0].root, "held-response.json"), "utf8"),
    );
    observation.heldNative = { request: nativeRequest, response: nativeResponse };
    assert.equal(nativeRequest.method, "textDocument/hover");
    const position = nativeRequest.params.position;
    assert.deepEqual(nativeResponse, {
      jsonrpc: "2.0",
      id: nativeRequest.id,
      result: {
        contents: expected.contents,
        range: {
          start: { line: position.line, character: position.character - 1 },
          end: { line: position.line, character: position.character + 6 },
        },
      },
    });
    session.notify("$/cancelRequest", { id: heldId });
    const canceled = await held;
    assert.ok(
      "error" in canceled &&
        canceled.error instanceof LspRequestError &&
        canceled.error.code === -32800,
    );
    record("old-caller-canceled", { code: canceled.error.code, id: heldId });
    assert.deepEqual(
      responses.find((response) => (response as { id?: number }).id === heldId),
      {
        jsonrpc: "2.0",
        id: heldId,
        error: { code: -32800, message: "Canceled" },
      },
    );
    const oldLsp = oldGraph.find((entry) => entry.role === "editor-lsp")!;
    if (mode === "killed") {
      const proxy = children(session.processId, node).find(
        (entry) => entry.pid === oldLsp.proxy.pid && entry.birth_ticks === oldLsp.proxy.birth_ticks,
      );
      assert.ok(proxy, "the fault target's proxy is still a direct Vize child");
      const target = children(proxy.pid, native).find(
        (entry) =>
          entry.pid === oldLsp.native.pid && entry.birth_ticks === oldLsp.native.birth_ticks,
      );
      assert.ok(
        target,
        "the injected fault must target the exact previously witnessed native life",
      );
      assert.equal(target.executableSha256, nativeSha256);
      const last = actualProbe(target);
      assert.equal(last.current?.birth_ticks, target.birth_ticks);
      assert.notEqual(last.current?.state, "Z");
      assert.equal(fs.realpathSync(`/proc/${target.pid}/exe`), native);
      observation.faultControl = {
        purpose: "injected backend fault, never successful shutdown cleanup",
        signal: "SIGKILL",
        target,
        proxy,
        beforeSignal: last,
      };
      // No await separates this final PID/birth/image verification from the deliberate fault.
      const delivered = process.kill(target.pid, "SIGKILL");
      Object.assign(observation.faultControl as object, { signalDelivered: delivered });
      record("old-native-explicit-fault", observation.faultControl);
    }
    fs.writeFileSync(configPath, JSON.stringify(config(gates[1].proxy)));
    session.notify("workspace/didChangeWatchedFiles", {
      changes: [{ uri: pathToFileURL(configPath).href, type: 2 }],
    });
    record("configuration-retirement-notified");
    let newQueryId = 0;
    observation.newOwnerQuery = {
      method: "textDocument/hover",
      params,
      result: await session.request("textDocument/hover", params, 10000, (id) => {
        newQueryId = id;
        return [];
      }),
    };
    assert.deepEqual((observation.newOwnerQuery as { result: unknown }).result, expected);
    assert.deepEqual(
      responses.find((response) => (response as { id?: number }).id === newQueryId),
      {
        jsonrpc: "2.0",
        id: newQueryId,
        result: expected,
      },
    );
    const allNewGraph = census(gates[1]);
    observation.newFullOwnerGraph = allNewGraph;
    const newGraph = graph(gates[1], allNewGraph);
    observation.newOwnerGraph = newGraph;
    assert.deepEqual(newGraph.map((entry) => entry.role).sort(), ["editor-lsp", "project-api"]);
    const all = [...allOldGraph, ...allNewGraph].flatMap((entry) => [entry.proxy, entry.native]);
    assert.equal(all.length, 16);
    assert.equal(new Set(all.map((entry) => entry.pid)).size, 16);
    const probes = [...physicalByOwner.values()].flatMap((starts) =>
      starts
        .filter((start) => start.invocation === "version-probe")
        .flatMap((start) => [start.proxy, start.native]),
    );
    all.push(...probes);
    assert.equal(new Set(all.map((entry) => entry.pid)).size, all.length);
    const unchangedBirthCensus = () => {
      for (const gate of gates)
        assert.deepEqual(
          physicalStarts(gate, {
            vizePid: session.processId,
            node,
            nodeSha256,
            native,
            nativeSha256,
          }),
          physicalByOwner.get(gate.role),
          "no additional, missing or rewritten spawn can escape the complete birth census",
        );
    };
    if (mode === "held") {
      const heldLive = children(oldLsp.proxy.pid, native).find(
        (child) =>
          child.pid === oldLsp.native.pid && child.birth_ticks === oldLsp.native.birth_ticks,
      );
      observation.oldStillInflightAfterNewOwner = heldLive;
      assert.ok(
        heldLive,
        "the retired old native owner must remain genuinely in flight after new-owner admission",
      );
      assert.ok(!fs.existsSync(path.join(gates[0].root, "release")));
      const deadline = Date.now() + 5_000;
      record("shutdown-started");
      const shutdown = session.request("shutdown", undefined, 5000).then(
        (response) => {
          record("shutdown-response", response);
          return { kind: "response" as const, response };
        },
        (error: unknown) => {
          record("shutdown-error", String(error));
          return { kind: "error" as const, error };
        },
      );
      const probes: unknown[] = [];
      observation.newOwnerClosureProbes = probes;
      const newIdentities = allNewGraph.flatMap((entry) => [entry.proxy, entry.native]);
      const newClosure = (async () => {
        while (polling) {
          const observed = newIdentities.map(actualProbe);
          probes.push({ at: Date.now(), observed });
          if (observed.every((entry) => isReaped(entry, entry.current)))
            return { kind: "reaped" as const };
          assert.ok(
            Date.now() < deadline,
            "new-owner closure shares the original five-second shutdown deadline",
          );
          await pause(25);
        }
        return { kind: "aborted" as const };
      })();
      const frontier = await Promise.race([newClosure, shutdown]);
      observation.newOwnerClosureFrontier = frontier;
      polling = false;
      await newClosure;
      assert.equal(
        frontier.kind,
        "reaped",
        "shutdown ACK must not bypass the still-held retired owner",
      );
      record("new-owner-actually-reaped");
      observation.vizeAtHeldFrontier = {
        wholeStat: fs.readFileSync(`/proc/${session.processId}/stat`, "utf8"),
        executable: fs.realpathSync(`/proc/${session.processId}/exe`),
      };
      observation.oldStillInflightAfterNewClosure = children(oldLsp.proxy.pid, native);
      assert.ok(
        (observation.oldStillInflightAfterNewClosure as Witness[]).some(
          (child) =>
            child.pid === oldLsp.native.pid && child.birth_ticks === oldLsp.native.birth_ticks,
        ),
      );
      const pending = await Promise.race([
        shutdown,
        pause(100).then(() => ({ kind: "pending" as const })),
      ]);
      observation.heldShutdownFrontier = pending;
      assert.equal(
        pending.kind,
        "pending",
        "new-owner closure cannot hide an omitted retired-owner join",
      );
      assert.ok(Date.now() < deadline);
      record("old-native-response-release");
      fs.writeFileSync(path.join(gates[0].root, "release"), "");
      const terminal = await shutdown;
      observation.shutdownTerminal = terminal;
      assert.deepEqual(terminal, { kind: "response", response: null });
    } else {
      record("recovered-owner-shutdown-started");
      let shutdownId = 0;
      observation.shutdownTerminal = await session.request("shutdown", undefined, 5000, (id) => {
        shutdownId = id;
        return [];
      });
      assert.equal(observation.shutdownTerminal, null);
      assert.deepEqual(
        responses.find((response) => (response as { id?: number }).id === shutdownId),
        {
          jsonrpc: "2.0",
          id: shutdownId,
          result: null,
        },
      );
    }
    unchangedBirthCensus();
    const atAck = all.map(actualProbe);
    observation.allOwnerIdentitiesAtShutdownAck = atAck;
    assert.ok(
      atAck.every((entry) => isReaped(entry, entry.current)),
      "a null shutdown ACK requires every exact early/final API/editor proxy/native life already reaped; same-birth zombies fail",
    );
    const vizeStatAtAck = fs.readFileSync(`/proc/${session.processId}/stat`, "utf8");
    const vizeImageAtAck = fs.realpathSync(`/proc/${session.processId}/exe`);
    observation.vizeAtShutdownAck = { wholeStat: vizeStatAtAck, executable: vizeImageAtAck };
    assert.notEqual(vizeStatAtAck.slice(vizeStatAtAck.lastIndexOf(") ") + 2).split(" ")[0], "Z");
    assert.equal(vizeImageAtAck, fs.realpathSync(process.env.VIZE_LSP_BIN!));
    record("all-owner-identities-absent-at-ack");
    observation.serverExit = await session.finishInput("exit");
    assert.deepEqual(observation.serverExit, { exitCode: 0, signal: null });
    unchangedBirthCensus();
    observation.allOwnerReaping = await observeReaped(all);
    assert.equal(
      (observation.allOwnerReaping as { reaped: boolean }).reaped,
      true,
      "every old/new proxy and native child must be absent or have a different birth; zombies fail",
    );
    for (const gate of gates) {
      const starts = physicalByOwner.get(gate.role)!;
      const terminalBytes = fs.readFileSync(
        path.join(gate.root, "physical-terminals.jsonl"),
        "utf8",
      );
      assert.ok(terminalBytes.endsWith("\n"));
      const physicalTerminals = terminalBytes
        .trimEnd()
        .split("\n")
        .map((line) => object(JSON.parse(line), "actual native terminal"));
      assert.equal(physicalTerminals.length, starts.length);
      const terminalPids = new Set<number>();
      for (const terminal of physicalTerminals) {
        const start = starts.find((entry) => entry.native.pid === terminal.nativePid);
        assert.ok(start, "every terminal must match an independently witnessed native birth");
        assert.ok(!terminalPids.has(start.native.pid));
        terminalPids.add(start.native.pid);
        const killed = mode === "killed" && start.native.pid === oldLsp.native.pid;
        assert.deepEqual(terminal, {
          proxyPid: start.proxy.pid,
          nativePid: start.native.pid,
          code: killed ? null : 0,
          signal: killed ? "SIGKILL" : null,
        });
      }
      const nativePids = fs
        .readFileSync(path.join(gate.root, "native.pids"), "utf8")
        .trimEnd()
        .split("\n")
        .map((pid) => {
          assert.match(pid, /^\d+$/u);
          return Number(pid);
        });
      assert.deepEqual(
        nativePids.toSorted((a, b) => a - b),
        starts.map((start) => start.native.pid).sort((a, b) => a - b),
      );
      const exits = fs
        .readFileSync(path.join(gate.root, "native.exits"), "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
      assert.equal(exits.length, starts.length);
      if (mode === "killed" && gate.role === "old") {
        const killed = exits.filter((exit) => exit.code === null && exit.signal === "SIGKILL");
        assert.equal(killed.length, 1, "retain exactly the explicitly injected old native fault");
        for (const exit of exits.filter((exit) => exit.signal !== "SIGKILL"))
          assert.deepEqual(
            exit,
            { code: 0, signal: null },
            "the other old API owner and probes must exit cleanly",
          );
      } else
        for (const exit of exits)
          assert.deepEqual(
            exit,
            { code: 0, signal: null },
            "the actual native exits cleanly; a kill is never success",
          );
    }
    const journal = custodyJournal(session.stderrText);
    observation.typedRetirementJournal = journal;
    assertCustodyJournal(
      journal,
      { old: allOldGraph, new: allNewGraph },
      { old: oldGraph, new: newGraph },
      mode,
      mode === "killed" ? oldLsp.native.pid : null,
    );
    observation.warmLivePhaseOracleControl = assertWarmLivePhaseControl(
      journal,
      { old: allOldGraph, new: allNewGraph },
      { old: oldGraph, new: newGraph },
      mode,
      mode === "killed" ? oldLsp.native.pid : null,
    );
    success = true;
  } catch (error) {
    observation.failure = error instanceof Error ? error.stack : String(error);
    throw error;
  } finally {
    polling = false;
    observation.responses = responses;
    observation.stderr = session.stderrText;
    if (!observation.typedRetirementJournal) {
      try {
        observation.typedRetirementJournal = custodyJournal(session.stderrText);
      } catch (error) {
        observation.typedJournalParseFailure = error instanceof Error ? error.stack : String(error);
      }
    }
    observation.gateEvidence = gates.map((gate) => ({
      ...gate,
      files: Object.fromEntries(
        fs
          .readdirSync(gate.root)
          .map((name) => [name, fs.readFileSync(path.join(gate.root, name)).toString("base64")]),
      ),
    }));
    const capture = process.env.VIZE_NATIVE_RETIREMENT_CAPTURE_DIR;
    if (capture) {
      fs.mkdirSync(capture, { recursive: true });
      fs.writeFileSync(
        path.join(
          capture,
          mode === "held" ? "retired-owner-inflight.json" : "retired-owner-killed.json",
        ),
        `${JSON.stringify(observation, null, 2)}\n`,
      );
    }
    // Preserve the original failure and held-byte frontier before emergency cleanup.
    if (!success) {
      fs.writeFileSync(path.join(gates[0].root, "release"), "");
      await session.finishInput("exit").catch(() => session.kill().catch(() => undefined));
    }
    fs.rmSync(workspace, { recursive: true, force: true });
  }
}
