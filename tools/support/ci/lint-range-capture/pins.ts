// Historical verified baseline; current reporter bytes are pinned below.
export const reporterBaseline = "ddef7f37fc2e3e1692168e16a52bdb3f21464faf";
export const originalSource = "ae874862fb175f1e3a6df27cbd412b270f664d60";
export const originalRun = 37098199031;
export const historicalInvalidCount = 224; // Comparison only; never an expected fresh count.
export const outputDirectory = "eslint-invalid-range-capture";
export const pins = [
  {
    path: "tools/commands/fixtures/lint-divergence-report.rs",
    gitEntry:
      "100644 blob 61f24873ee65df0d19636aece01a1ddfb761b840\ttools/commands/fixtures/lint-divergence-report.rs",
    sha256: "28e6791248e7c6f68a48af0ae00b99f6178ba61f035af0d16b462bdb0a23792d",
    bytes: 69449,
    sameOriginalBytes: false,
  },
  {
    path: "tools/benchmarks/scripts/package.json",
    gitEntry:
      "100644 blob 8f697801ce6dc05615579751941c82fc671e4bd0\ttools/benchmarks/scripts/package.json",
    sha256: "f0903bb317e21f8d684e9253a9ad50389104c6b01b9168f98097099a1b566b91",
    bytes: 999,
    sameOriginalBytes: true,
  },
  {
    path: "pnpm-lock.yaml",
    gitEntry: "100644 blob 76754c988cbb8fc983861b087ab74c71e497f5a8\tpnpm-lock.yaml",
    sha256: "c16e5e1370026b63cfb4603ad06cf6c0425e6dc4578149efa53eca1e47a5578e",
    bytes: 921981,
    sameOriginalBytes: true,
  },
  {
    path: "pnpm-workspace.yaml",
    gitEntry: "100644 blob 5bd4a1c3e0dcd90c7958430ccfaf6c722c5d83db\tpnpm-workspace.yaml",
    sha256: "8d56e626002a50be4c9526333af15dc1a3dcd882b05ac22c42bc0633958b664e",
    bytes: 16812,
    sameOriginalBytes: true,
  },
  {
    path: "package.json",
    gitEntry: "100644 blob 1d17aec6920d0c405c40a62b670839f8c2f3b92a\tpackage.json",
    sha256: "d8fcbdb8b01ce5837a2ff1691a8f45649cb6dc936b49b4883f8cf6d04ea722e2",
    bytes: 1736,
    sameOriginalBytes: true,
  },
  {
    path: "tests/_fixtures/patina-eslint-vue-rule-map.json",
    gitEntry:
      "100644 blob dfb2bd2e802e71ed977784b5cd67b71170880d30\ttests/_fixtures/patina-eslint-vue-rule-map.json",
    sha256: "504e8038af7b2b90a76124b66749702cf988101dd54270d100147617dcd8732c",
    bytes: 42711,
    sameOriginalBytes: true,
  },
  {
    path: "tests/_fixtures/patina-lint-documented-divergences.json",
    gitEntry:
      "100644 blob fe51488c7066f6687ef680d6bfaa4f7768ef205c\ttests/_fixtures/patina-lint-documented-divergences.json",
    sha256: "37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570",
    bytes: 3,
    sameOriginalBytes: true,
  },
  {
    path: "tools/support/common.rs",
    gitEntry: "100644 blob 671e8016f6e44c5fe8928fb8f9fe5964ab38d2e5\ttools/support/common.rs",
    sha256: "c23e55a197a410a899082875ffc34e4478573d855f3463ec8ebc6780644eb497",
    bytes: 8265,
    sameOriginalBytes: true,
  },
  {
    path: ".gitmodules",
    gitEntry: "100644 blob bc71f6bc676f2de703adeee5f039e904f92604aa\t.gitmodules",
    sha256: "afc78b70104cfcbbbf6acd79b1a7c27729e5c49b59a1942b65d240c85ba007bb",
    bytes: 21974,
    sameOriginalBytes: true,
  },
  {
    path: ".github/actions/setup-rust-script/action.yml",
    gitEntry:
      "100644 blob a5b9971f0ba1f87598760b97d5cf660628c64939\t.github/actions/setup-rust-script/action.yml",
    sha256: "32bdb4455eede5b12ad962fa28930797d47fe1b10cfcacd72703218184bc2c50",
    bytes: 1539,
    sameOriginalBytes: true,
  },
] as const;
export const versions = {
  eslint: "10.4.1",
  "eslint-plugin-vue": "10.9.2",
  "vue-eslint-parser": "10.4.1",
  "@typescript-eslint/parser": "8.65.0",
} as const;
export const lockClosure = {
  snapshots: 97,
  sha256: "6ad071998c4a62e366a0c07c998773fe98131b205210bb6496610a5d5ae800cf",
  authority: "Prepared original importer inventory; whole installed lock is pinned above.",
};
