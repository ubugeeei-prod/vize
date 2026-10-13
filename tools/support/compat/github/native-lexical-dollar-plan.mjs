export const helperPaths = [
  "native-lexical-dollar.sh",
  "native-lexical-dollar-metadata.mjs",
  "native-lexical-dollar-receipt.mjs",
  "native-lexical-composition-custody.mjs",
].map((name) => `tools/support/compat/github/${name}`);
export const cargoArgs = [
  "cargo",
  "test",
  "--locked",
  "--profile",
  "ci-opt",
  "-p",
  "vize_canon",
  "--lib",
  "--no-run",
  "--message-format=json",
  "--config",
  'profile.ci-opt.inherits="release"',
  "--config",
  'profile.ci-opt.lto="thin"',
  "--config",
  "profile.ci-opt.codegen-units=16",
];
export const originalReceiptNames = [
  "receipts.json",
  "imported-runtime-props.runtime.json",
  "editor-runtime.json",
];
export const unobserved = [
  "diagnosing native PID",
  "native normalized options",
  "engine loaded source-file vector",
];
