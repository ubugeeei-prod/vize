import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import type { VizePublicRegistryInstallAuthority } from "../installed-alias-replay/authority-schema.ts";
import { exactFile } from "../installed-alias-replay/custody.ts";
import type { InstalledAliasSession, Packet } from "../installed-alias-replay/protocol.ts";
import {
  auditBoundRecord,
  boundFiles,
  boundGuardSource,
  boundTsconfig,
  boundVizeConfig,
  loadFrozenBoundCases,
  observeBoundCase,
  prepareBoundExpectation,
  type BoundRecord,
} from "./bound.ts";
import {
  auditEventSlotRecord,
  eventSlotTsconfig,
  eventSlotVizeConfig,
  loadEventSlotCases,
  observeEventSlotCase,
  prepareEventSlotExpectation,
  type EventSlotRecord,
} from "./events-slots.ts";
import {
  auditLibraryRecord,
  libraryTsconfig,
  libraryVizeConfig,
  loadFrozenLibraryCases,
  observeLibraryCase,
  prepareLibraryExpectation,
  type LibraryRecord,
} from "./library.ts";
import { prepareEventGuard, type EventGuardExpectation } from "./runner-guard.ts";

export type ControllerRecord = EventSlotRecord | LibraryRecord | BoundRecord;
export type PreparedScope = {
  files: readonly { file: string; text: string }[];
  tsconfig: string;
  vizeConfig: unknown;
  lint: boolean;
  shutdownId: number;
  expected: unknown;
  guard?: EventGuardExpectation;
  pendingBatchWitness?: string;
  observe: (
    session: InstalledAliasSession,
    progress: (record: ControllerRecord) => void,
  ) => Promise<ControllerRecord>;
  audit: (record: ControllerRecord, notifications: readonly Packet[]) => unknown[];
};
export type ReplayCase = {
  id: string;
  family: "event-slot" | "library" | "bound";
  context: string;
  prepare: (projectRoot: string) => PreparedScope;
};

/** Same original ancestor search, constrained to the authenticated public Corsa archive. */
export function publicDomLibrary(authority: VizePublicRegistryInstallAuthority): string {
  const packageRoot = path.join(
    authority.installRoot,
    "node_modules",
    authority.bundledCorsa.packageName,
  );
  assert.equal(fs.realpathSync(packageRoot), packageRoot);
  const manifest = JSON.parse(fs.readFileSync(authority.payloadManifestPath, "utf8"));
  const payload = manifest.packages.find(
    (entry: { name: string }) => entry.name === authority.bundledCorsa.packageName,
  );
  assert.ok(payload);
  let directory = path.dirname(authority.bundledCorsa.path);
  for (;;) {
    assert.ok(directory === packageRoot || directory.startsWith(`${packageRoot}${path.sep}`));
    for (const candidate of [
      path.join(directory, "lib.dom.d.ts"),
      path.join(directory, "lib/lib.dom.d.ts"),
    ]) {
      if (!fs.existsSync(candidate)) continue;
      const owned = payload.files.find((entry: { path: string }) => entry.path === candidate);
      assert.ok(owned, "DOM library must be an authenticated public archive member");
      return exactFile(candidate, owned.sha256, packageRoot);
    }
    if (directory === packageRoot) break;
    directory = path.dirname(directory);
  }
  throw new Error("the reviewed public Corsa archive must expose its actual bundled DOM library");
}

/** Every finite immutable oracle is loaded before the first installed provider launch. */
export function loadReplayCases(sourceRoot: string, libraryPath: string): readonly ReplayCase[] {
  const eventCases: ReplayCase[] = loadEventSlotCases(sourceRoot).map((oracle) => ({
    id: oracle.id,
    family: "event-slot",
    context: oracle.context,
    prepare: (root) => {
      const guard = prepareEventGuard(root);
      return {
        files: oracle.files,
        tsconfig: eventSlotTsconfig,
        vizeConfig: eventSlotVizeConfig,
        lint: false,
        shutdownId: 4,
        expected: prepareEventSlotExpectation(oracle, root),
        guard,
        observe: (session, progress) =>
          observeEventSlotCase(session, oracle, root, progress, [guard.invalid, guard.repaired]),
        audit: (record, notifications) =>
          auditEventSlotRecord(record as EventSlotRecord, notifications),
      };
    },
  }));
  const libraryRoot = path.join(
    sourceRoot,
    "tests/_fixtures/differential/lsp/installed-event-slot-replay/library",
  );
  const libraryCases: ReplayCase[] = loadFrozenLibraryCases(libraryRoot).map((oracle, index) => ({
    id: `library-${String(index + 1).padStart(2, "0")}-${oracle.kind}-${oracle.newline === "\n" ? "lf" : "crlf"}`,
    family: "library",
    context: oracle.context,
    prepare: (root) => ({
      files: [
        { file: "src/Toggle.vue", text: oracle.inputs.toggleDisk ?? oracle.inputs.toggle },
        { file: "src/App.vue", text: oracle.inputs.app },
      ],
      tsconfig: libraryTsconfig,
      vizeConfig: libraryVizeConfig,
      lint: true,
      shutdownId: oracle.kind === "ambiguous" ? 6 : oracle.kind === "partial" ? 3 : 4,
      expected: prepareLibraryExpectation(oracle, root),
      ...(oracle.kind === "partial"
        ? {
            pendingBatchWitness:
              "The separate observed-only public batch invocation is pending. No independently authored whole batch stdout golden exists; these LSP results grant no batch qualification.",
          }
        : {}),
      observe: (session, progress) =>
        observeLibraryCase(session, oracle, root, progress, { libraryPath }),
      audit: (record, notifications) => auditLibraryRecord(record as LibraryRecord, notifications),
    }),
  }));
  const boundRoot = path.join(
    sourceRoot,
    "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter",
  );
  const boundCases: ReplayCase[] = loadFrozenBoundCases(boundRoot).map((oracle, index) => ({
    id: `bound-${String(index + 1).padStart(2, "0")}-${oracle.form}-${oracle.newline === "\n" ? "lf" : "crlf"}`,
    family: "bound",
    context: `bound ${oracle.form}, newline=${JSON.stringify(oracle.newline)}`,
    prepare: (root) => ({
      files: [
        ...boundFiles.map((file) => ({ file: `src/${file}`, text: oracle.sources[file] })),
        { file: "src/NativeGuard.vue", text: boundGuardSource },
      ],
      tsconfig: boundTsconfig,
      vizeConfig: boundVizeConfig,
      lint: false,
      shutdownId: 101,
      expected: prepareBoundExpectation(oracle, root),
      observe: (session, progress) => observeBoundCase(session, oracle, root, progress),
      audit: (record, notifications) => auditBoundRecord(record as BoundRecord, notifications),
    }),
  }));
  const cases = [...eventCases, ...libraryCases, ...boundCases];
  assert.deepEqual([eventCases.length, libraryCases.length, boundCases.length], [18, 22, 20]);
  assert.equal(new Set(cases.map((item) => item.id)).size, 60);
  for (const item of cases) assert.match(item.id, /^[A-Za-z0-9_-]+$/u);
  return cases;
}
