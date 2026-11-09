import { runEntrypoint } from "../../../support/command-runtime.ts";
import {
  assertTarPackage,
  relativeArchivePath,
  workspaceVersion,
} from "../../../support/compat/editor-e2e/archive.ts";

const languageFiles = [
  "brackets.scm",
  "config.toml",
  "highlights.scm",
  "indents.scm",
  "injections.scm",
  "outline.scm",
  "overrides.scm",
] as const;
const required = [
  "zed/Cargo.lock",
  "zed/Cargo.toml",
  "zed/LICENSE",
  "zed/README.md",
  "zed/extension.toml",
  ...languageFiles.map((file) => `zed/languages/art-vue/${file}`),
  ...languageFiles.map((file) => `zed/languages/vue/${file}`),
  "zed/src/lib.rs",
  "zed/src/initialization_options.rs",
  "zed/src/initialization_tests.rs",
  "zed/src/unit_tests.rs",
];

runEntrypoint(() => {
  const version = workspaceVersion();
  const result = assertTarPackage({
    defaultArchive: "zed-vize-extension.tar.gz",
    label: "Zed archive",
    root: "zed",
    minimumBytes: 5_000,
    maximumBytes: 2_000_000,
    required,
    allowed: [
      "zed/",
      "zed/Cargo.lock",
      "zed/Cargo.toml",
      "zed/LICENSE",
      "zed/README.md",
      "zed/extension.toml",
      "zed/languages/",
      "zed/languages/art-vue/",
      "zed/languages/vue/",
      ...languageFiles.map((file) => `zed/languages/art-vue/${file}`),
      ...languageFiles.map((file) => `zed/languages/vue/${file}`),
      "zed/src/",
      "zed/src/lib.rs",
      "zed/src/initialization_options.rs",
      "zed/src/initialization_tests.rs",
      "zed/src/unit_tests.rs",
    ],
    forbidden: [".git", ".github/", ".zed/", "node_modules/", "target/", ".DS_Store", "~"],
    expectations: {
      "zed/extension.toml": [
        'id = "vize"',
        'name = "Vize"',
        `version = "${version}"`,
        'repository = "https://github.com/ubugeeei-prod/vize"',
        "[language_servers.vize]",
        'languages = ["Vue.js", "Art Vue"]',
        "[language_servers.vize.language_ids]",
        '"Vue.js" = "vue"',
        '"Art Vue" = "art-vue"',
        "[grammars.vue]",
      ],
      "zed/Cargo.toml": [
        'name = "vize-zed-extension"',
        `version = "${version}"`,
        'edition = "2024"',
        'license = "MIT"',
        "publish = false",
        'crate-type = ["cdylib"]',
        'zed_extension_api = "=0.7.0"',
      ],
      "zed/src/lib.rs": [
        `const SERVER_NAME: &'static str = "vize";`,
        `const SERVER_BINARY: &'static str = "vize";`,
        "worktree.which(Self::SERVER_BINARY)",
        'unwrap_or_else(|| vec!["lsp".to_string()])',
        "language_server_initialization_options",
        "settings.initialization_options",
        "language_server_workspace_configuration",
        "zed::register_extension!(VizeExtension);",
      ],
      "zed/src/initialization_options.rs": ["explicit.unwrap_or_else(|| serde_json::json!({}))"],
      "zed/languages/art-vue/config.toml": [
        'name = "Art Vue"',
        'grammar = "vue"',
        'path_suffixes = ["art.vue"]',
        'prettier_parser_name = "vue"',
      ],
      "zed/languages/vue/config.toml": [
        'name = "Vue.js"',
        'grammar = "vue"',
        'path_suffixes = ["vue"]',
        'prettier_parser_name = "vue"',
      ],
      "zed/languages/art-vue/injections.scm": [
        "directive_attribute",
        "style_element",
        "template_element",
      ],
      "zed/languages/art-vue/highlights.scm": ["@tag.component.type.constructor"],
    },
  });
  const extension = result.read("zed/extension.toml");
  if (extension.includes("[grammars.") && extension.includes("[grammars.art-vue]"))
    throw new Error("Zed extension.toml must not declare dashed grammar ids");
  console.log(
    `Zed package smoke passed: ${relativeArchivePath(result.archive)} (${result.entries.length} entries)`,
  );
});
