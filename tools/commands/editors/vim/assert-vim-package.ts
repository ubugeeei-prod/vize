import { runEntrypoint } from "../../../support/command-runtime.ts";
import {
  assertTarPackage,
  relativeArchivePath,
} from "../../../support/compat/editor-e2e/archive.ts";

const required = [
  "vim/LICENSE",
  "vim/README.md",
  "vim/autoload/vize.vim",
  "vim/ftdetect/vize.vim",
  "vim/plugin/vize.vim",
  "vim/test/vize_e2e_expected.vim",
  "vim/test/vize_e2e_spec.vim",
  "vim/test/config_project_spec.vim",
  "vim/test/vize_spec.vim",
] as const;

runEntrypoint(() => {
  const result = assertTarPackage({
    defaultArchive: "vim-vize-extension.tar.gz",
    label: "Vim archive",
    root: "vim",
    minimumBytes: 2_000,
    maximumBytes: 200_000,
    required,
    allowed: [
      "vim/",
      "vim/LICENSE",
      "vim/README.md",
      "vim/autoload/",
      "vim/ftdetect/",
      "vim/plugin/",
      "vim/test/",
      ...required.slice(2),
    ],
    forbidden: [".git", ".github/", "node_modules/", "target/", ".DS_Store", ".tar.gz", "~"],
    expectations: {
      "vim/autoload/vize.vim": [
        "'cmd': ['vize', 'lsp']",
        "'allowlist': ['vue', 'art-vue']",
        "initialization_options",
        "function! vize#vim_lsp_config",
        "lsp#register_server",
      ],
      "vim/ftdetect/vize.vim": [
        "*.vue setlocal filetype=vue",
        "*.art.vue call <SID>detect_art_vue()",
        "setlocal filetype=art-vue",
        "if empty(&l:syntax)",
      ],
      "vim/test/vize_spec.vim": ["vize#vim_lsp_config", "assert_equal(['vize', 'lsp']"],
    },
  });
  console.log(
    `Vim package smoke passed: ${relativeArchivePath(result.archive)} (${result.entries.length} entries)`,
  );
});
