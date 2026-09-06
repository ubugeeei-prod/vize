/**
 * The static contribution and configuration surface the extension-host suites
 * assert against.
 *
 * These are the extension's published contract — the command ids it registers
 * and the `vize.<key>.enable` switches it maps onto LSP initialization options
 * — so they live apart from the suites that exercise them. A suite that needs
 * to know what "the recommended profile" means reads it here rather than
 * restating it, which is what kept the profile expectations in sync when the
 * capability list grew.
 */

const extensionId = "ubugeeei.vize";

const recommendedInitializationOptions = {
  editor: true,
  ecosystem: true,
  lint: true,
  typecheck: true,
};

const explicitlyDisabledInitializationOptions = {
  codeActions: false,
  codeLens: false,
  completion: false,
  definition: false,
  documentLinks: false,
  documentSymbols: false,
  ecosystem: false,
  editor: false,
  fileRename: false,
  autoInsert: false,
  foldingRanges: false,
  formatting: false,
  hover: false,
  inlayHints: false,
  legacyVue2: false,
  lint: false,
  optionsApi: false,
  references: false,
  rename: false,
  semanticTokens: false,
  signatureHelp: false,
  typecheck: false,
  workspaceSymbols: false,
};

const lintOnlyInitializationOptions = {
  ...explicitlyDisabledInitializationOptions,
  lint: true,
};

const featureSettingKeys = [
  "lint.enable",
  "diagnostics.enable",
  "typecheck.enable",
  "editor.enable",
  "ecosystem.enable",
  "optionsApi.enable",
  "legacyVue2.enable",
  "completion.enable",
  "hover.enable",
  "definition.enable",
  "references.enable",
  "documentSymbols.enable",
  "workspaceSymbols.enable",
  "codeActions.enable",
  "rename.enable",
  "codeLens.enable",
  "formatting.enable",
  "semanticTokens.enable",
  "signatureHelp.enable",
  "documentLinks.enable",
  "foldingRanges.enable",
  "inlayHints.enable",
  "fileRename.enable",
  "autoInsert.enable",
];

const granularEditorCapabilitySettings = [
  ["completion.enable", "completion"],
  ["signatureHelp.enable", "signatureHelp"],
  ["hover.enable", "hover"],
  ["definition.enable", "definition"],
  ["references.enable", "references"],
  ["documentSymbols.enable", "documentSymbols"],
  ["workspaceSymbols.enable", "workspaceSymbols"],
  ["codeActions.enable", "codeActions"],
  ["rename.enable", "rename"],
  ["codeLens.enable", "codeLens"],
  ["formatting.enable", "formatting"],
  ["semanticTokens.enable", "semanticTokens"],
  ["documentLinks.enable", "documentLinks"],
  ["foldingRanges.enable", "foldingRanges"],
  ["inlayHints.enable", "inlayHints"],
  ["fileRename.enable", "fileRename"],
];

const commandIds = [
  "vize.disable",
  "vize.enableLintOnlyProfile",
  "vize.enableRecommendedProfile",
  "vize.findReferences",
  "vize.restartServer",
  "vize.selectServerPath",
  "vize.showOutput",
  "vize.showStatus",
];

module.exports = {
  commandIds,
  explicitlyDisabledInitializationOptions,
  extensionId,
  featureSettingKeys,
  granularEditorCapabilitySettings,
  lintOnlyInitializationOptions,
  recommendedInitializationOptions,
};
