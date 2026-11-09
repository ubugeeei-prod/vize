import { runEntrypoint } from "../../../support/command-runtime.ts";
import {
  assertTarPackage,
  relativeArchivePath,
} from "../../../support/compat/editor-e2e/archive.ts";

const required = [
  "nvim/LICENSE",
  "nvim/README.md",
  "nvim/ftdetect/vize.lua",
  "nvim/lua/vize/config.lua",
  "nvim/lua/vize/init.lua",
  "nvim/plugin/vize.lua",
  "nvim/test/component_contract_hover.lua",
  "nvim/test/ref_surface_hover.lua",
  "nvim/test/vize_e2e_expected.lua",
  "nvim/test/vize_e2e_spec.lua",
  "nvim/test/vize_spec.lua",
] as const;

runEntrypoint(() => {
  const result = assertTarPackage({
    defaultArchive: "nvim-vize-extension.tar.gz",
    label: "Neovim archive",
    root: "nvim",
    minimumBytes: 2_000,
    maximumBytes: 200_000,
    required,
    allowed: [
      "nvim/",
      "nvim/LICENSE",
      "nvim/README.md",
      "nvim/ftdetect/",
      "nvim/lua/",
      "nvim/lua/vize/",
      "nvim/plugin/",
      "nvim/test/",
      ...required.slice(2),
    ],
    forbidden: [".git", ".github/", "node_modules/", "target/", ".DS_Store", ".tar.gz", "~"],
    expectations: {
      "nvim/lua/vize/config.lua": [
        'cmd = { "vize", "lsp" }',
        'filetypes = { "vue", "art-vue" }',
        "root_markers = {",
        '"vite.config.ts"',
        '"vite.config.js"',
        '"tsconfig.json"',
        "lint = true",
        "recommended = {",
        "init_options = vim.empty_dict()",
        'assert_list("cmd"',
      ],
      "nvim/lua/vize/init.lua": ['vim.lsp.config("vize", resolved)', 'vim.lsp.enable("vize")'],
      "nvim/ftdetect/vize.lua": [
        'pattern = "*.vue"',
        'pattern = "*.art.vue"',
        'filetype = "art-vue"',
        'vim.treesitter.language.add("art-vue")',
        'vim.treesitter.language.register("vue", "art-vue")',
      ],
      "nvim/test/vize_spec.lua": ["config.normalize", "vim.lsp.config.vize"],
      "nvim/test/vize_e2e_spec.lua": [
        "vim.lsp.start",
        "textDocument/completion",
        "textDocument/hover",
        "textDocument/codeAction",
        "textDocument/formatting",
        "textDocument/semanticTokens/full",
        "textDocument/rename",
      ],
    },
  });
  console.log(
    `Neovim package smoke passed: ${relativeArchivePath(result.archive)} (${result.entries.length} entries)`,
  );
});
