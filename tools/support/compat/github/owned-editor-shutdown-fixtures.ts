// This finite source transition owns LSP/worker shutdown inputs, not batch parity.
// Registration itself grants no source, runtime, SDK or semantic acceptance.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const OWNED_EDITOR_SHUTDOWN_PROTOCOL_ID = "lsp/owned-outer-shutdown-10760";
const LSP_CASE_ID = "lsp/regression/owned-outer-shutdown-10760";
const PACK = "tests/_fixtures/differential/lsp/owned-outer-shutdown-10760";
const registration = {
  id: "lsp/owned-outer-shutdown-10760",
  state: "active",
  adapter: "owned-editor-shutdown-v1",
  qualification: "requires-real-source-7.0.2-close-ordering",
  sources: [
    {
      path: "crates/vize_canon/src/corsa_bridge/bridge.rs",
      before: "91e321e78c98f33db894ad3ec073ff28551b26f739cc3d45a74fe7f292febcfb",
      after: "77770ba233d56638cf5f9e05907ade6b4c63461a1ea5cf633fa6b10c94f92bf7",
    },
    {
      path: "crates/vize_canon/src/corsa_bridge/bridge/shutdown_tests.rs",
      before: null,
      after: "7769a78c038b0c8e601071dda3afad148093ecf83f947fe10a7675faabb69aed",
    },
    {
      path: "crates/vize_maestro/src/server.rs",
      before: "c9f6bc1ea5051be5c3e3757fa2520bae5008911f1b38af5ab302835cbafe871a",
      after: "4570fd025c4be5015b1955b27342481d493b66b097fdcbb3cd02f0472a162b59",
    },
    {
      path: "crates/vize_maestro/src/server/handlers.rs",
      before: "d50e55fdddfdac71f227c821e8859245a0aa87a240fe31170957a1bc45c54032",
      after: "8b3fa0c391a207d8ec89ebab0766f407bafd033daa79193573981ddf10d226c2",
    },
    {
      path: "crates/vize_maestro/src/server/project_contexts.rs",
      before: "bab88463abc9bb505dfa5daf4c6759bab738de74a57e8571b1d44511f2ec3ae2",
      after: "fefadea99c1a52de2aac6835015135a5118e454a2fffefc40aee14bf15d6c96f",
    },
    {
      path: "crates/vize_maestro/src/server/shutdown.rs",
      before: null,
      after: "06f15781fbcd30d14fa478b39e9a9ac8c054fd81b3299d65fd6877ede8e4d944",
    },
    {
      path: "crates/vize_maestro/src/server/shutdown/tests.rs",
      before: null,
      after: "13aee1317acf85e104fc58141ce37fde35fb4de4f76b228627ad138fa142acb0",
    },
    {
      path: "crates/vize_maestro/src/server/state/corsa.rs",
      before: "ad997cdc7e5809b1ac167d39eed6d9f3a0c3f321b8ed15f30a989880918dcfb1",
      after: "c1a268a8e7bac3a32677fd2ba879b47ccf6a4eb19fe62fc123c9c5506fe6cd52",
    },
    {
      path: "crates/vize_maestro/src/server/state/corsa/tests.rs",
      before: "1a8f1474c8e9fbfa8b3ff09056022b9f2644a7024e81e5e5d710ee979f45762e",
      after: "f498a56d147ea548e6565339bb0378a8e8a84d5c9b2f3fc093d36af62364a3ae",
    },
    {
      path: "crates/vize_maestro/src/server/state/corsa/tests/shutdown.rs",
      before: null,
      after: "14ee3751bce05bf8282810709442824639e7271e0f650539f6aa67eb5826ff5b",
    },
    {
      path: "crates/vize_maestro/src/server/state/project_contexts.rs",
      before: "89d047e51bab6b350787e081d0900eb4d5dbb0188185cf819b417772717dd20d",
      after: "7eada8dfbae0c728c27e5a0db9fcd35642172b93f22683486961209ffd7bc592",
    },
    {
      path: "crates/vize_maestro/src/server/state/project_contexts/registry.rs",
      before: "e08854a6dd938523718ebd10cd0aee8422796ae551ea1dfd35b73ec38d9e3003",
      after: "f1f11832ecb9dd74d770ce1e1b086e312c03f8e01d1e475c14bff58c8738d3a5",
    },
    {
      path: "crates/vize_maestro/src/server/state/project_contexts/shutdown.rs",
      before: null,
      after: "fb464b280b73c913219b85af3ef295ec39f72fa7f447b714dfe0bd3262d2e3a0",
    },
    {
      path: "crates/vize_maestro/src/server/state/project_contexts/shutdown/tests.rs",
      before: null,
      after: "42ab0a801f5c1f78fae16f65481daada89b775da35151e5b33b0f49f5faa860a",
    },
  ],
  ownership: {
    path: "tests/_fixtures/runtime/lsp-owned-shutdown-10760/ownership.json",
    sha256: "2c48cb912e633f8598f5f4e85813fc4286b096142431e8d3dd26619ea10e5acb",
  },
  rustSelectors: [
    {
      source: "crates/vize_canon/src/corsa_bridge/bridge/shutdown_tests.rs",
      sha256: "7769a78c038b0c8e601071dda3afad148093ecf83f947fe10a7675faabb69aed",
      selector:
        "corsa_bridge::bridge::shutdown_tests::pinned_shutdown_ownership_cases_use_the_actual_worker_lane",
    },
    {
      source: "crates/vize_maestro/src/server/state/project_contexts/shutdown/tests.rs",
      sha256: "42ab0a801f5c1f78fae16f65481daada89b775da35151e5b33b0f49f5faa860a",
      selector:
        "server::state::project_contexts::shutdown::tests::pinned_owner_graphs_capture_the_complete_registry",
    },
  ],
  lspCase: {
    path: "tests/_fixtures/differential/lsp/owned-outer-shutdown-10760/case.json",
    sha256: "df6d4037468dd8a4f7846e411aced216858e1f976311597d8fd58bca028e6e91",
  },
  wireContract: {
    path: "tests/_fixtures/differential/lsp/owned-outer-shutdown-10760/wire-contract.json",
    sha256: "b366f034bec8a5011601a6053ae7ef04d0c3fa07ef91113817e6e08a9946797b",
  },
  producer: {
    files: [
      {
        path: "tests/tooling/support/native-runtime-selection/providers.mjs",
        sha256: "666b2b35d4665fa695eabdf473497ef617ec3871332d2dfc66dbd3f2ed250879",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp-source-custody.mjs",
        sha256: "abe1920ba0e6badab8b81a4fb1e1eeb16704f9a0bbb83ccdf60f55ddb5b1327b",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp-wire.mjs",
        sha256: "19abd5edafb2cde0be60b90097f28e4f71df644373a613d1e355246645ff37f3",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp-strace.mjs",
        sha256: "b189dfc2de82bfd0bd9fce6e5584c02e38c9984ee6c2df2800d8caf2df0fcbaf",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp-strace-lines.mjs",
        sha256: "75433713a12562fd38fa9d475e15006d43d19771bee7e5524f11eabe454c6ca6",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp-strace-terminal.mjs",
        sha256: "006e6ba72071b008f4c9b08a816fc00d243bca657340f6fb6cf5c293edd58d8c",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp-native-lifecycle.mjs",
        sha256: "95395e34acb935bd98775e635cdc51331060feb5dc37cd97fa248405fefa8eba",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp-shared-api-campaign.mjs",
        sha256: "915fc8b0662d4989ee5ffacf9004142fa38127e54e0773e49a70834aa3229b6c",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp.mjs",
        sha256: "083c80ad8a662422f6c534d243e75a5652c0f4756cc7024d13755777612290fd",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp-owned-outer-shutdown.mjs",
        sha256: "7d829a0b1164dbc020149bd30f175cc94f77b035e55ae0deda028fb2b3399898",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp-owned-outer-shutdown-campaign.mjs",
        sha256: "0b63d2cc097ee503e79d833e7ebb09d41eaf871d86fbe6ddb4e245a646a82a14",
      },
      {
        path: "tests/tooling/support/native-runtime-selection/lsp-owned-outer-shutdown-order.mjs",
        sha256: "7684c3c9854ce98895a422ee0c7651184afc10ac74682e44f968cb37134d4eb1",
      },
      {
        path: ".github/workflows/lsp-owned-outer-shutdown.yml",
        sha256: "1c4cea1334b8957bbe08a8bdeb8ee1c7a849cfe4979ab87b48c568b5b42f0aef",
      },
      {
        path: "tests/tooling/owned-editor-shutdown-order.test.mjs",
        sha256: "136d502e28c9f6babfe08420b7c4a105bc86cd46a208876bd17edd16f414b6ec",
      },
      {
        path: "tests/tooling/owned-editor-shutdown-fixtures.test.mjs",
        sha256: "8018463e5955e80cd5a6133670ec7f7cac11f5d7df5d7e3179e92826c4060131",
      },
      {
        path: "tests/tooling/native-runtime-selection-lsp-strace.test.mjs",
        sha256: "952310756b287146a0c6a8c9c07be52a54142008041ef8e3d90ff1391ff49716",
      },
      {
        path: "tests/_fixtures/runtime/lsp-owned-shutdown-10760/reviewed-before.json",
        sha256: "5b9d6232e3ea3b37a47110c7a96fb68dc0aaf8e4b0d30d6d270ad6bd7eeb8d2d",
      },
      {
        path: "tests/tooling/owned-editor-shutdown-report.test.mjs",
        sha256: "2a657982e7ff35cd1e6de9ca7b8545815fe577cdb4b7d91172b4e75e598556c3",
      },
    ],
    sdk: {
      package: "typescript",
      version: "7.0.2",
      platform: "linux-x64",
      sha256: "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
    },
    validationComposition: {
      requiredSnapshotFix: "ed44709453b1561a0420507b6d79284f3e507357",
      qualificationOnly: true,
      productInheritance: false,
    },
  },
} as const;

const digest = (bytes: Buffer): string => createHash("sha256").update(bytes).digest("hex");
function rows(value: unknown): unknown[] {
  if (value === undefined) return [];
  assert(Array.isArray(value), "closed owned-shutdown protocol rows required");
  assert(value.length <= 1, "duplicate owned-shutdown protocol row");
  for (const row of value)
    assert.deepEqual(row, registration, "unknown or changed owned-shutdown registration");
  return value;
}
function pinned(readFile: ReadFile, ref: { path: string; sha256: string }): Buffer {
  const bytes = readFile("head", ref.path);
  assert(bytes !== null, `missing owned-shutdown source/input: ${ref.path}`);
  assert.equal(digest(bytes), ref.sha256, `changed owned-shutdown source/input: ${ref.path}`);
  return bytes;
}

/** Only the exact reviewed joint transition may substitute real LSP inputs. */
export function newOwnedEditorShutdownCases(
  previous: unknown,
  current: unknown,
  readFile: ReadFile,
): { ids: string[]; sources: Set<string> } {
  const old = rows(previous),
    next = rows(current);
  const applicable = registration.sources.every((source) => {
    const before = readFile("base", source.path),
      after = readFile("head", source.path);
    return (
      (source.before === null
        ? before === null
        : before !== null && digest(before) === source.before) &&
      after !== null &&
      digest(after) === source.after
    );
  });
  if (!applicable) {
    assert(next.length === 0 || old.length !== 0, "unreviewed owned-shutdown source transition");
    return { ids: [], sources: new Set() };
  }
  assert.equal(old.length, 0, "the exact owned-shutdown fix requires a new registration");
  assert.equal(next.length, 1, "the exact owned-shutdown fix requires its closed LSP registration");
  assert.equal(readFile("base", registration.lspCase.path), null, "new ordinary LSP case required");
  const manifestBytes = readFile("head", "tests/_fixtures/differential/lsp/manifest.json");
  assert(manifestBytes !== null, "ordinary LSP manifest required");
  const manifest = JSON.parse(manifestBytes.toString("utf8")) as {
    cases?: Array<{
      id?: unknown;
      case?: unknown;
      inputs?: unknown;
      requestCount?: unknown;
    }>;
  };
  assert(Array.isArray(manifest.cases));
  const selected = manifest.cases.filter((row) => row.id === LSP_CASE_ID);
  assert.equal(selected.length, 1, "one real ordinary LSP case required");
  const data = JSON.parse(pinned(readFile, registration.lspCase).toString("utf8")) as {
    id?: unknown;
    entry?: unknown;
    files?: Array<{ source: string; runtimePath: string; sha256: string }>;
    expected?: { source: string; sha256: string };
    provenance?: { wireContract?: unknown };
  };
  assert.equal(data.id, LSP_CASE_ID);
  assert.equal(data.entry, "src/App.vue");
  assert(Array.isArray(data.files) && data.files.length === 4);
  assert.deepEqual(
    data.files.map((file) => file.runtimePath),
    ["src/App.vue", "src/support.ts", "tsconfig.json", "vite.config.mjs"],
  );
  assert.deepEqual(selected[0].case, {
    path: "owned-outer-shutdown-10760/case.json",
    sha256: registration.lspCase.sha256,
  });
  assert.equal(selected[0].requestCount, 2);
  assert.deepEqual(selected[0].inputs, {
    root: "owned-outer-shutdown-10760",
    files: data.files.map((file) => ({ path: file.source, sha256: file.sha256 })),
  });
  for (const file of data.files)
    pinned(readFile, { path: `${PACK}/${file.source}`, sha256: file.sha256 });
  assert(data.expected !== undefined, "complete ordinary hover oracle required");
  const expected = JSON.parse(
    pinned(readFile, {
      path: `${PACK}/${data.expected.source}`,
      sha256: data.expected.sha256,
    }).toString("utf8"),
  ) as unknown;
  assert(Array.isArray(expected) && expected.length === 2, "both whole ordinary hovers required");
  assert.deepEqual(data.provenance?.wireContract, {
    path: "wire-contract.json",
    sha256: registration.wireContract.sha256,
  });
  const wire = JSON.parse(pinned(readFile, registration.wireContract).toString("utf8")) as {
    id?: unknown;
    version?: unknown;
    initializeRequest: { path: string; sha256: string };
    initializerSource: { path: string; sha256: string };
  };
  assert.equal(wire.id, OWNED_EDITOR_SHUTDOWN_PROTOCOL_ID);
  assert.equal(wire.version, 1);
  pinned(readFile, {
    path: `${PACK}/${wire.initializeRequest.path}`,
    sha256: wire.initializeRequest.sha256,
  });
  pinned(readFile, wire.initializerSource);
  const ownership = JSON.parse(pinned(readFile, registration.ownership).toString("utf8")) as {
    schema?: unknown;
    version?: unknown;
  };
  assert.equal(ownership.schema, "vize.owned-compiler-shutdown");
  assert.equal(ownership.version, 1);
  for (const selector of registration.rustSelectors) {
    const text = pinned(readFile, { path: selector.source, sha256: selector.sha256 }).toString(
      "utf8",
    );
    const name = selector.selector.split("::").at(-1);
    assert(name !== undefined);
    assert(text.includes(`fn ${name}()`), "actual Rust selector required");
    assert(text.includes("include_str!("), "Rust selector consumes the committed ownership input");
    assert(text.includes("tests/_fixtures/runtime/lsp-owned-shutdown-10760/ownership.json"));
  }
  for (const ref of registration.producer.files) pinned(readFile, ref);
  // The complete Maestro closure is checked above but keeps its ordinary LSP law.
  // These are the only two reviewed Canon paths; no pattern or future edit inherits it.
  return {
    ids: [OWNED_EDITOR_SHUTDOWN_PROTOCOL_ID],
    sources: new Set([
      "crates/vize_canon/src/corsa_bridge/bridge.rs",
      "crates/vize_canon/src/corsa_bridge/bridge/shutdown_tests.rs",
    ]),
  };
}
