// Final C12 source tuple. Current-head qualification remains required before dispatch.
export const CANDIDATE_SOURCE = "1f316f04cbad600e3b6075ec383ef65fa270593b";
export const BASE_SOURCE = "83433431f96dd514f2de4c835a3b25ab6c67f416";
export const CANDIDATE_TREE = "bb35d2ec846929fcffcbfd6c29193030c6df1699";
export const PROVIDER_SHA256 = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
export const SOURCE_PINS: Readonly<Record<string, string>> = {
  ".cargo/config.toml": "9087d9834c7170cfa25b764557d51b652081f674af8ee03f83e3a947eabb13e5",
  "Cargo.lock": "e9a8659e79a919541b6cf29340886dabad787a18fa3051bd77873fc3a76194a4",
  "Cargo.toml": "6400cafce2abb1c16bb3c43dfc6498d0a0c695c88849bd5a6bdd4063b91377de",
  "crates/vize/Cargo.toml": "69d2a340d7690e1a340df270c9beb8cb459afaf43cc42bfaab848f4eb01d93a0",
  "crates/vize/tests/lsp_corsa_responsiveness_cli.rs":
    "ccaba05109caf53590d4b91d4de9bfcb7d951711aba28d626afe13343224ea57",
  "crates/vize_canon/Cargo.toml":
    "04887c025691d2ba0e027489f44bf69883af3c99739f51cdaa4b7e41bd769cfc",
  "crates/vize_canon/src/corsa_bridge.rs":
    "126a5e8aaf1e0c4a218495f1c53303fe3b413dedbc1c753fcada1a281b1f56bb",
  "crates/vize_canon/src/corsa_bridge/bridge.rs":
    "4b8ce1b623a8312fb4fa3b570be1e86ac844d53749fa8b10a8c447e815592c93",
  "crates/vize_canon/src/corsa_bridge/bridge/startup.rs":
    "88bcc649664c72e702c85bcd3cc9b1d77056cc5452969c00df7a77f9e9396a72",
  "crates/vize_canon/src/corsa_bridge/bridge/startup/tests.rs":
    "96d8be78cdb35fb2a5ddb6f0fd5faab960142538761b158c4c52f4d8f871aeda",
  "crates/vize_canon/src/corsa_bridge/editor_session.rs":
    "8c6f9938833a95bec4ea432536df91ce7773f998677d91037a1f933ad56983ab",
  "crates/vize_canon/src/corsa_bridge/editor_session/retained_tests.rs":
    "27ab638f1cd7c000cbb658ddf86d1ed356f0da74e1b654d3b7daf19f503287bb",
  "crates/vize_canon/src/corsa_bridge/session.rs":
    "57f0d130d2353a315636392d35471ef30d105f212617ae1cb3f0125e9e51c523",
  "crates/vize_canon/src/corsa_bridge/vue_document.rs":
    "5398eab8dd20be99759055781cac3d144ee2ca178d28759511d3a039d049cfc6",
  "crates/vize_canon/src/corsa_bridge/vue_document/initial_project.rs":
    "18990a40129b98a2ac1a1367333b880ce62b7c867dae6e4f3beb9738ca247ce5",
  "crates/vize_canon/src/corsa_bridge/vue_document/initial_project/native_tests.rs":
    "40e5758bff4470172be582d36d03ccb7e1e8ac0115731213e7acaeed59af4a4a",
  "crates/vize_canon/src/corsa_bridge/vue_document/initial_project/tests.rs":
    "1f9e80a7c8da4c8734e0b46747daeaef846367a4fef42bf9d787621bd3f2765b",
  "crates/vize_canon/src/corsa_bridge/worker.rs":
    "8d155a1792e4e56cad7ca8939dc79705732a494f78e9dfca8c53344ac0b0c06b",
  "crates/vize_canon/src/lib.rs":
    "aac29d38afa0cf15af6463fa6dea0ffe5c3b6771a86bad44c59f7920465895b0",
  "crates/vize_canon/src/lsp_client/lifecycle/workspace.rs":
    "753276aec69a0ef195f62745c82a9c93b6f5ac6a061eb8e57a83fc6c23e87d9a",
  "crates/vize_canon/src/lsp_client/lifecycle/workspace/initial_project_tests.rs":
    "82a60799502575f4901ea9e80d4d95c83f64520c5dc0bed553ddc25745f61608",
  "crates/vize_canon/src/lsp_client/workspace_project.rs":
    "08774437fae78288baa58ab855f3aee329188e69839f45d090ea6461790c19ea",
  "crates/vize_canon/src/lsp_client/workspace_project/initial.rs":
    "3cda28ebb701720d218af6c5b80a724670575d57faae78f2a83fd650ffa58b20",
  "crates/vize_maestro/Cargo.toml":
    "be62177cefb2e99a97aec549d84372020f6380cbb9ec7e543d3bed7c891d53d2",
  "crates/vize_maestro/src/ide/context.rs":
    "e38a2c655100546581aa241b27637cb9bef6b87b4632171efb5b0f65f5a0845e",
  "crates/vize_maestro/src/ide/context/initial_project.rs":
    "de1e12e939c7fe21caccdf4e2dd47f7bcac13746109ae7759f0b31533fbf7d6c",
  "crates/vize_maestro/src/ide/context/initial_project/tests.rs":
    "02340710ab1a9eac67e967d076978e75dcef27a971341956a2d3a8d8b4f518ae",
  "crates/vize_maestro/src/ide/corsa_support/canonical/open.rs":
    "3fea161c2862a3c364997fc072d81cec8cb76d1729d80f1c4ac4f1083e136a59",
  "crates/vize_maestro/src/server.rs":
    "035f90a568afacca9a5eaadec57c2940dbd6627ccb6001bc59d9c9f1c95d65e9",
  "crates/vize_maestro/src/server/diagnostic_publishing/retry_tests.rs":
    "a830d784385037afabc4e952265ec2fe019ad6d0b4ee3de8c616d90a7b7a0aef",
  "crates/vize_maestro/src/server/handlers/typed.rs":
    "8b7f11857234b5e2b1ad41bd8777993e60303589a5d64eb1b393ef307a27449e",
  "crates/vize_maestro/src/server/initial_diagnostics/preparation.rs":
    "69111593f4289cc3ef8ae828d44da51c8219d1a4522e37ddd44ee87f018a189a",
  "crates/vize_maestro/src/server/state/corsa.rs":
    "ec4e4738cd47e5ed3ad7498042d1cc2fdcc5651eaae814960a2f7f36313806fd",
  "crates/vize_maestro/src/server/state/corsa/initial_project.rs":
    "fd85fab341bab57ff59d8efeb7feda3acaa9dee504f7ae17c1618df7b854a59d",
  "crates/vize_maestro/src/server/state/corsa/initial_project/tests.rs":
    "5cc90da77a818a911958026fcd7fc520b15f2fe32159fdddbbc8b796661ee8e3",
  "crates/vize_maestro/src/server/state/project_contexts/shutdown.rs":
    "0d1b4e4b8d5f036eb74645a6443829d749703e01a8b919cb1c7a5f6daababfb6",
  "crates/vize_maestro/src/server/state/project_contexts/shutdown/tests.rs":
    "ea0170ebc8b21b1ba4fde96eed9b25ef9964b54d4a64c52b9aaf84eb2390486c",
  "editors/vscode/test/css-hover-foreground-source.ts":
    "77de355fd5c2120a35a48c7c9358990c66d222d873a1cd14138dea136dd075d0",
  "editors/vscode/test/css-hover-initial-project-source.ts":
    "b493bb88de7d307294f6826a3809eee8c16ad04f69ed431db2db60a0559f760e",
  "npm/cli/content-mapper-helpers.d.cts":
    "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
  "npm/cli/content-mapper-helpers.d.mts":
    "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
  "npm/cli/content-mapper-helpers.d.ts":
    "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
  "npm/cli/generated/content-mapper-types.d.ts":
    "7457fd86aafb4b2a5b1edbddedf538a45a072517645dd0ece2fd4c074a0f632e",
  "npm/cli/virtual-ts-globals.d.ts":
    "d647737ea035f166c94c131013af3265a3a33783dbad307b2eb3c6d558fc476a",
  "npm/cli/virtual-ts-patterns.d.ts":
    "e42c2c431553563663e28f75b4e68f8728d928a2867c857c76ef8cf30fe0f545",
  "npm/cli/virtual-ts-slots.d.ts":
    "3b2f3387f8e720bdffd4ed864b74af6fa19895a89ba6f2f64151fd8dd98bdf9f",
  "npm/cli/virtual-ts.d.cts": "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
  "npm/cli/virtual-ts.d.mts": "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
  "npm/cli/virtual-ts.d.ts": "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
  "package.json": "703095a422cfc630a5d29ef5f4a5230b1c853c6013b8e8080809643cc5b4c6a0",
  "pnpm-lock.yaml": "bc0ff0b8a3b617d02c86d27540ba3046fe55ef5d41c2e9179dec155caaf5c38c",
  "pnpm-workspace.yaml": "e183899daa4d3b95c151deba9b2457d7202f07f81f79f80879a35aa3eced32fd",
  "rust-toolchain.toml": "cf13b173bea8eb5e0dfca16b32c015341369c3706301cf329415e96567ac9c71",
  "tests/_fixtures/ci/native-phase-membership-original.json":
    "c7c12caf0399493ab456974022ee3399626c0b3601a09a4afc340f54b6a1add9",
  "tests/_fixtures/differential/typechecker/original-diagnosing-process-7698/input.json":
    "3b9e44621b6399d53ff7281d53e710c76ebab2b5328f08deba3ed3d171eba906",
  "tests/davinci_test_support/src/original_diagnosing_process.rs":
    "209e5a6e15d8d47cc70cf6afc6c1b36b8248c4394b7983fa74119a114b5dad27",
  "tests/tooling/fixtures/css-hover-initial-project/foreground-before.rs.txt":
    "752179a706971407290c7144720bbcbc63261031bf6b5378bdb8cc9427b601ac",
  "tests/tooling/fixtures/css-hover-initial-project/initial-project.rs.txt":
    "8b7f11857234b5e2b1ad41bd8777993e60303589a5d64eb1b393ef307a27449e",
  "tests/tooling/vscode-css-hover-foreground-source.test.ts":
    "cd09384ef54ce621a8d0f630b91c50cc7003c35ee5105ca5528327eab7cf0bb5",
  "tests/tooling/vscode-css-hover-initial-project-source.test.ts":
    "9385ed4045d78d9a079c01e4fea34db2d05154dc7919400ccd48756ad3e1d8a4",
};
export const SDK = {
  corsa: {
    name: "corsa",
    version: "2.0.0-beta.4",
    source: "registry+https://github.com/rust-lang/crates.io-index",
    checksum: "a88d1e7cd3fa6919d9f2ab4b51de40645547606317ac3f04dfccc1ded0ce9dcc",
  },
  corsa_client: {
    name: "corsa_client",
    version: "2.0.0-beta.4",
    source: "registry+https://github.com/rust-lang/crates.io-index",
    checksum: "48683d933fc2fd2b2bf9f8563103c2bbcb249f0ec6a6ace1831644ca0a84d780",
  },
  corsa_core: {
    name: "corsa_core",
    version: "2.0.0-beta.4",
    source: "registry+https://github.com/rust-lang/crates.io-index",
    checksum: "c64b958bb6e54a0f6a13cde2f5df28ce95234b6bb90c289fd2a3b6027af951c7",
  },
  corsa_jsonrpc: {
    name: "corsa_jsonrpc",
    version: "2.0.0-beta.4",
    source: "registry+https://github.com/rust-lang/crates.io-index",
    checksum: "5603d3a1ca6c9f0caadbf8e89c853b740f31f44e818a9c1a01f09c2b93da5c8f",
  },
  corsa_lsp: {
    name: "corsa_lsp",
    version: "2.0.0-beta.4",
    source: "registry+https://github.com/rust-lang/crates.io-index",
    checksum: "b6949c82fac11db1376c361c7e70a6ac6019dedce1d379110385229eb643ea25",
  },
  corsa_orchestrator: {
    name: "corsa_orchestrator",
    version: "2.0.0-beta.4",
    source: "registry+https://github.com/rust-lang/crates.io-index",
    checksum: "56a610d7d8fec2e576d461751bd7ccb7bf2348a030418b5dc6756473353e2c0c",
  },
  corsa_runtime: {
    name: "corsa_runtime",
    version: "2.0.0-beta.4",
    source: "registry+https://github.com/rust-lang/crates.io-index",
    checksum: "71b6fd0a075158abbb1c513e754eb73b368066bbccf5967c9a2be5d6305df637",
  },
} as const;
export const TOOLING_PATHS = [
  ".github/workflows/lsp-current-baseline.yml",
  ".github/actions/lsp-initial-canon-functional/controller.ts",
  ".github/actions/lsp-initial-canon-functional/laws.ts",
  ".github/actions/lsp-initial-canon-functional/controls.test.ts",
  ".github/actions/lsp-initial-canon-functional/README.md",
  ".github/actions/lsp-initial-canon-functional/source-pins.ts",
] as const;
