// Only these reviewed editor/API source pairs can register LSP inputs instead
// of batch inputs. This grants no runtime, SDK or semantic acceptance.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const SHARED_EDITOR_INITIAL_API_PROTOCOL_ID = "lsp/shared-editor-initial-snapshot-10730";
const LSP_CASE_ID = "lsp/regression/shared-api-snapshot-reopen-10730";
const registration = {
  id: "lsp/shared-editor-initial-snapshot-10730",
  state: "active",
  adapter: "shared-editor-initial-snapshot-v1",
  qualification: "requires-real-same-source-7.0.2-whole-wire",
  sources: [
    {
      path: "crates/vize_canon/src/lsp_client/initial_diagnostics.rs",
      before: "5318a27ee65231a0bf146ec028f79c129f5eb4588e6f5ba7132d809487bfd1a8",
      after: "d011c4e2885c4a9252d43982abf369fce2a953ee08dee38b780beb0a3ec72627",
    },
    {
      path: "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
      before: "02601e1d2bb3eee642b663fd3336156f32791a35cf5999e689c69956626357fb",
      after: "be7c5924613e2f284b8d9e88d30d5512077de2cbfd8ca1e8e77cca15f7c0a971",
    },
    {
      path: "crates/vize_canon/src/lsp_client/editor_lsp/inlay_hint.rs",
      before: "0c477b1f2dbab225f4d63935c687f4eb5ae88017966c857692c739dc463448bd",
      after: "6f43f25b70046abcde4664b09a5d6c974793e690628cc5ddda8aff673acac07c",
    },
  ],
  lspCase: {
    path: "tests/_fixtures/differential/lsp/shared-api-snapshot-reopen-10730/case.json",
    sha256: "ae8dd53c5060b16fe59863097bc6916c168e15e5cee7b77139ea5e2022a9a713",
  },
  wireContract: {
    path: "tests/_fixtures/differential/lsp/shared-api-snapshot-reopen-10730/wire-contract.json",
    sha256: "787ac66952fc6b6a8a74b0cbf96aaf41709fb65831a9e874c991d8cb0995f97a",
  },
  producer: {
    workflow: {
      path: ".github/workflows/lsp-shared-api-reopen.yml",
      sha256: "04c66c27dbd17e47306850cacfd623e4dca48531c4845cadc8ffae67bc83cd76",
    },
    entry: {
      path: "tests/tooling/support/native-runtime-selection/lsp-shared-api-reopen.mjs",
      sha256: "17930e2ea156a5775403888bf20a70e91b12048291294a7d343fd8eaaeb3a2fa",
    },
    driver: {
      path: "tests/tooling/support/native-runtime-selection/lsp.mjs",
      sha256: "e074a5caeb6cca1ede78091f57e61e59ab4a05d82ab241f47f634ed33dfab11c",
    },
    custody: {
      path: "tests/tooling/support/native-runtime-selection/lsp-source-custody.mjs",
      sha256: "abe1920ba0e6badab8b81a4fb1e1eeb16704f9a0bbb83ccdf60f55ddb5b1327b",
    },
    lifecycle: {
      path: "tests/tooling/support/native-runtime-selection/lsp-native-lifecycle.mjs",
      sha256: "95395e34acb935bd98775e635cdc51331060feb5dc37cd97fa248405fefa8eba",
    },
    campaign: {
      path: "tests/tooling/support/native-runtime-selection/lsp-shared-api-campaign.mjs",
      sha256: "915fc8b0662d4989ee5ffacf9004142fa38127e54e0773e49a70834aa3229b6c",
    },
    wire: {
      path: "tests/tooling/support/native-runtime-selection/lsp-wire.mjs",
      sha256: "19abd5edafb2cde0be60b90097f28e4f71df644373a613d1e355246645ff37f3",
    },
    strace: {
      path: "tests/tooling/support/native-runtime-selection/lsp-strace.mjs",
      sha256: "afca55e47efef1a1f34e1847b6a7872291afbffdf67dae5781332c1295b6e9e9",
    },
    fixtureTest: {
      path: "tests/tooling/lsp-shared-api-campaign.test.mjs",
      sha256: "3a668098059a393e107f745c0d714d621c2982078abd70d8c0e0c0c43320fa7d",
    },
    sdk: {
      package: "typescript",
      version: "7.0.2",
      platform: "linux-x64",
      sha256: "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
    },
    straceTerminal: {
      path: "tests/tooling/support/native-runtime-selection/lsp-strace-terminal.mjs",
      sha256: "c951dacdae6db634bb7a8b2ed006095a64838718946b957414e756ad202d47b7",
    },
    registryTest: {
      path: "tests/tooling/shared-editor-initial-api-fixtures.test.mjs",
      sha256: "c3de4d5662c05f677d25e268611e86e94d444396bda4b8b1fbb84e09c1900768",
    },
  },
} as const;

const digest = (bytes: Buffer): string => createHash("sha256").update(bytes).digest("hex");
function rows(value: unknown): unknown[] {
  if (value === undefined) return [];
  assert(Array.isArray(value), "closed initial snapshot protocol rows required");
  assert(value.length <= 1, "duplicate initial snapshot protocol row");
  for (const row of value)
    assert.deepEqual(row, registration, "unknown or changed initial snapshot registration");
  return value;
}
function pinned(
  readFile: ReadFile,
  side: "base" | "head",
  ref: { path: string; sha256: string },
): Buffer {
  const bytes = readFile(side, ref.path);
  assert(bytes !== null, `missing ${side} source/input: ${ref.path}`);
  assert.equal(digest(bytes), ref.sha256, `changed ${side} source/input: ${ref.path}`);
  return bytes;
}

/** Missing/existing registration lends no classification to another edit. */
export function newSharedEditorInitialApiCases(
  before: unknown,
  after: unknown,
  readFile: ReadFile,
): { ids: string[]; sources: Set<string> } {
  const old = rows(before),
    next = rows(after);
  const applicable = registration.sources.every((source) => {
    const beforeBytes = readFile("base", source.path),
      afterBytes = readFile("head", source.path);
    return (
      beforeBytes !== null &&
      afterBytes !== null &&
      digest(beforeBytes) === source.before &&
      digest(afterBytes) === source.after
    );
  });
  if (!applicable) {
    assert(next.length === 0 || old.length !== 0, "unreviewed initial snapshot source pair");
    return { ids: [], sources: new Set() };
  }
  assert.equal(old.length, 0, "the exact initial snapshot fix requires a new registration");
  assert.equal(
    next.length,
    1,
    "the exact initial snapshot fix requires its closed LSP registration",
  );
  for (const source of registration.sources) {
    pinned(readFile, "base", { path: source.path, sha256: source.before });
    pinned(readFile, "head", { path: source.path, sha256: source.after });
  }
  const original = readFile("base", registration.lspCase.path);
  assert.equal(original, null, "this registration requires its newly authored LSP case");
  const manifestBytes = readFile("head", "tests/_fixtures/differential/lsp/manifest.json");
  assert(manifestBytes !== null, "ordinary language LSP manifest required");
  const manifest = JSON.parse(manifestBytes.toString("utf8")) as {
    cases?: Array<{
      id?: unknown;
      case?: unknown;
      inputs?: { root?: unknown; files?: unknown };
      requestCount?: unknown;
    }>;
  };
  assert(Array.isArray(manifest.cases));
  const selected = manifest.cases.filter((row) => row.id === LSP_CASE_ID);
  assert.equal(selected.length, 1, "one complete ordinary LSP source case required");
  const planned = selected[0];
  assert.deepEqual(planned.case, {
    path: "shared-api-snapshot-reopen-10730/case.json",
    sha256: registration.lspCase.sha256,
  });
  assert.equal(planned.requestCount, 2);
  const data = JSON.parse(pinned(readFile, "head", registration.lspCase).toString("utf8")) as {
    id?: unknown;
    entry?: unknown;
    files?: Array<{ source: string; runtimePath: string; sha256: string }>;
    provenance?: { wireContract?: unknown };
  };
  assert.equal(data.id, LSP_CASE_ID);
  assert.equal(data.entry, "src/App.vue");
  assert(Array.isArray(data.files) && data.files.length === 4);
  assert.deepEqual(planned.inputs, {
    root: "shared-api-snapshot-reopen-10730",
    files: data.files.map((file) => ({ path: file.source, sha256: file.sha256 })),
  });
  for (const file of data.files)
    pinned(readFile, "head", {
      path: `tests/_fixtures/differential/lsp/shared-api-snapshot-reopen-10730/${file.source}`,
      sha256: file.sha256,
    });
  assert.deepEqual(data.provenance?.wireContract, {
    path: "wire-contract.json",
    sha256: registration.wireContract.sha256,
  });
  const wire = JSON.parse(pinned(readFile, "head", registration.wireContract).toString("utf8")) as {
    id?: unknown;
    basic?: unknown;
    transitions?: unknown;
  };
  assert.equal(wire.id, SHARED_EDITOR_INITIAL_API_PROTOCOL_ID);
  const { sdk: _sdk, ...producer } = registration.producer;
  for (const ref of Object.values(producer)) pinned(readFile, "head", ref);
  return {
    ids: [SHARED_EDITOR_INITIAL_API_PROTOCOL_ID],
    sources: new Set(registration.sources.map((source) => source.path)),
  };
}
