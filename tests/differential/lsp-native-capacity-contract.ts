//! Closed source-session contract for the original Issue 11008 admission.
export const NATIVE_CAPACITY_ID = "lsp/source-session/native-diagnostic-capacity-11008";
export const NATIVE_CAPACITY_ROOT =
  "tests/_fixtures/differential/lsp/native-diagnostic-capacity-11008";
export const NATIVE_CAPACITY_UNSUPPORTED_REASON =
  "The single-document adapter cannot execute the pinned stopped-backend burst and reopen lifecycle; the dedicated source executors retain their own full wire and source custody.";
export const NATIVE_CAPACITY_SESSION = {
  kind: "native-diagnostic-capacity-v1",
  state: "source-inspected",
  executor: "tests/tooling/lsp-diagnostic-capacity.test.ts",
  reopenExecutor: "tests/tooling/lsp-diagnostic-capacity-reopen.test.ts",
  sourceDocuments: 130,
  bulkDocuments: 129,
  plannedClientMessageCount: 395,
  projectInput: "protocol/project-input.json.gz",
  reopenInput: "protocol/reopen-input.json.gz",
} as const;
export type NativeCapacitySourceSession = typeof NATIVE_CAPACITY_SESSION;
export const NATIVE_CAPACITY_INITIALIZATION = {
  editor: true,
  lint: true,
  typecheck: true,
  typecheckTimeoutMs: 120000,
} as const;
export const NATIVE_CAPACITY_PLAN = {
  schema: "vize.lsp.native-diagnostic-capacity-session",
  version: 1,
  genericAdapter: "unsupported-transition",
  genericNativeParityChecked: 0,
  bulkDocuments: 129,
  sourceDocuments: 130,
  normalizedInputSha256: "7bb95a6f151e978c96ab0e94434533a5834ca54bd9ad977c94e4487a3b3db2fd",
  normalizedReopenSha256: "e4afa12839595e7fcb6d43e3d1f05d2bd99042af1f089a2551eed3994b179c70",
  initializationOptions: {
    editor: true,
    lint: true,
    typecheck: true,
    typecheckTimeoutMs: 120000,
  },
  client: {
    messages: 395,
    requests: {
      initialize: 1,
      "textDocument/documentColor": 1,
      shutdown: 1,
    },
    notifications: {
      initialized: 1,
      "textDocument/didOpen": 130,
      "textDocument/didChange": 260,
      exit: 1,
    },
    responses: 0,
  },
  operations: [
    "initialize",
    "seed-version1-native-error",
    "hold-seed-version2-owned-native-request",
    "open129-version1-invalid-with-sync-acceptance",
    "foreground-color-barrier",
    "resume-owned-native-backend",
    "observe-all-version1-native-errors",
    "bulk-repair129-version2",
    "sequential129-version3-native-recovery",
    "seed-version3-native-repair-and-checked-idle",
    "shutdown-and-exit",
  ],
  before: {
    revision: "49dc7684234a6ae723e1201c3d2bcb7c0039dbac",
    cliSha256: "c5241bc57d274bce1dd89a6b4d652bc0f828ede444f44caa22079ca29c3d7a79",
    coverage: [63, 64, 129],
    rawDiagnostics: 649,
    zeroReverseDependentReceipts: 261,
  },
  after: {
    revision: "c1dc7965dc51f91345db41a85f6cd5ca8866432c",
    cliSha256: "3e32f916f73cff80ddbdffcece6bc79238331eeb0660346f67f116dc6f136876",
    coverage: [129, 129, 129],
    rawDiagnostics: 780,
    zeroReverseDependentReceipts: 393,
  },
  reopen: {
    revision: "c1dc7965dc51f91345db41a85f6cd5ca8866432c",
    versions: [1, 2, 3],
    coverage: [1, 1, 1],
    rawDiagnostics: 10,
    stalePublications: 0,
    before: "not-measured",
  },
  seedVersion2AppendedText: "\n",
} as const;
export const NATIVE_CAPACITY_FILES = {
  "Seed.vue.txt": "Seed.vue",
  "project-input.json.gz": "protocol/project-input.json.gz",
  "reopen-input.json.gz": "protocol/reopen-input.json.gz",
  "before-result.json.gz": "custody/before-result.json.gz",
  "after-result.json.gz": "custody/after-result.json.gz",
  "reopen-result.json.gz": "custody/reopen-result.json.gz",
  "before-client.bin.gz": "custody/before-client.bin.gz",
  "before-server.bin.gz": "custody/before-server.bin.gz",
  "after-client.bin.gz": "custody/after-client.bin.gz",
  "after-server.bin.gz": "custody/after-server.bin.gz",
  "reopen-client.bin.gz": "custody/reopen-client.bin.gz",
  "reopen-server.bin.gz": "custody/reopen-server.bin.gz",
  "executor-before.ts.txt": "custody/executor-before.ts",
  "executor-capacity.ts.txt": "custody/executor-capacity.ts",
  "executor-reopen.ts.txt": "custody/executor-reopen.ts",
} as const;
export const NATIVE_CAPACITY_PINS = {
  "before-result.json.gz": {
    sha256: "60f40a7f19f2e940e329a8ee8fe84eae6ecbebd710aca31bb71add1e7a542014",
    rawSha256: "113619b1c73c3332f24d97bf3670c0c9006f69282bd3fb8813cb3fed366923eb",
    rawBytes: 7291697,
  },
  "after-result.json.gz": {
    sha256: "f91fa54beb8aaf16227fab0b4b44427b207602639e6d1daac04786655a1c96f3",
    rawSha256: "82c82d9fde067c68c008ac7eacf9a80841ff8cab5e0fc1654cdb8f693dfc7352",
    rawBytes: 9610099,
  },
  "reopen-result.json.gz": {
    sha256: "053ad4c6c3934222e02c8d9173aa916d36d3b47d6ebfb568cec94707550518e6",
    rawSha256: "485429f329569d8ab083c98b702c83c64bc312aed57a0dfd13edbd110f36a519",
    rawBytes: 287633,
  },
  "project-input.json.gz": {
    sha256: "84b06a27c7a2d4e9eeef231097437ad18fea66ba462376ba3709e3a42f6e7d8b",
    rawSha256: "ad5be53febce2b2b7163f15ecd0181ff0c9ca5b3584f528ee3b5dec4f98e9c6f",
    rawBytes: 112933,
  },
  "reopen-input.json.gz": {
    sha256: "0d56bf9f624298b1cd3915ebaaeab76aee148a796f0131b027dacdca8afcad68",
    rawSha256: "8e8f507183ffbc4e9f242958af904b4581e12312f4a1dced376ec8b2342b0b9e",
    rawBytes: 2707,
  },
  "before-client.bin.gz": {
    sha256: "012abe94dc878ee4f1020eda1480de9e37104ffb14ca97a93a67de7ed8a6b17a",
    rawSha256: "a8267cf8993655ca2ce6664a74a3f53cfae8b3aafce5046e69f1a8c64c511672",
    rawBytes: 235846,
  },
  "before-server.bin.gz": {
    sha256: "2d5d5a05ed5875082c74043e397140146419b5bf1b3bdf4aeaa313083ef1a82b",
    rawSha256: "ba5cbf01cadfeb487b0e3553429b2421f22ebd9df788c0f52442309670644b28",
    rawBytes: 302310,
  },
  "after-client.bin.gz": {
    sha256: "08d5d2fb1f84614e316fe881d5b525ef85e0e5e17bcf0952d379c9d95835e519",
    rawSha256: "cb7a65f8aab5070dcdbb47f0f3b2650bfa559e3e6227aa0ace7aa6daeaadd943",
    rawBytes: 235846,
  },
  "after-server.bin.gz": {
    sha256: "10031ddfd00fedbcc5ad3a90959028a9e54ace4210a4470d659ae500a4471f29",
    rawSha256: "7017495de2521d661cea3eacdfabe8ea2e0b960143baa8f8ec03df77c88ffc7a",
    rawBytes: 392172,
  },
  "reopen-client.bin.gz": {
    sha256: "08d6715fa7af6fd837687f0d84c46d2d2738a83868ea2a434e37c01b420f0688",
    rawSha256: "52cd0ab028a266e1572bdc8b748d5d060e6b4bd613ef3632c87a688c25eb87bd",
    rawBytes: 4817,
  },
  "reopen-server.bin.gz": {
    sha256: "4f07b5bde79f95225e15a3c8adef94f0a8d1ec66737a76cd6d541a8790f0f4f3",
    rawSha256: "8e75e6937b8e2ed2375d5a11c59c45f7a0f18ff822eaaea4d62f492e857d29ac",
    rawBytes: 9546,
  },
  "Seed.vue.txt": {
    sha256: "3edc49697b204c61b68bb07c3070027a67b3f990895336e4607817b83a6bd0f4",
    rawSha256: "3edc49697b204c61b68bb07c3070027a67b3f990895336e4607817b83a6bd0f4",
    rawBytes: 337,
  },
  "executor-before.ts.txt": {
    sha256: "45166518700caa99b21a43cb26cce9c9dbdbb12bc6e11af3ccba52b410a5b310",
    rawSha256: "45166518700caa99b21a43cb26cce9c9dbdbb12bc6e11af3ccba52b410a5b310",
    rawBytes: 12212,
  },
  "executor-capacity.ts.txt": {
    sha256: "2880db6ce9f287fa57ea880427f78f12fc411e8701931e6d2e32c658ed6d6395",
    rawSha256: "2880db6ce9f287fa57ea880427f78f12fc411e8701931e6d2e32c658ed6d6395",
    rawBytes: 12244,
  },
  "executor-reopen.ts.txt": {
    sha256: "53ad19fb21e6b1f3c2e749fa69cca9bf5e8e0f429bbd6d8debf53b5a4d43f8de",
    rawSha256: "53ad19fb21e6b1f3c2e749fa69cca9bf5e8e0f429bbd6d8debf53b5a4d43f8de",
    rawBytes: 10282,
  },
} as const;
export const NATIVE_CAPACITY_PLAN_SHA256 =
  "e8ff5c760b9daeccd879ec536fb077905eda5bae266db19cc5921bbeddbf5cb5";
