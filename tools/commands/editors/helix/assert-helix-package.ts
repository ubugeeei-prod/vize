import { runEntrypoint } from "../../../support/command-runtime.ts";
import {
  assertTarPackage,
  relativeArchivePath,
} from "../../../support/compat/editor-e2e/archive.ts";

runEntrypoint(() => {
  const required = ["helix/LICENSE", "helix/README.md", "helix/languages.toml"];
  const result = assertTarPackage({
    defaultArchive: "helix-vize-extension.tar.gz",
    label: "Helix archive",
    root: "helix",
    minimumBytes: 1_000,
    maximumBytes: 100_000,
    required,
    allowed: ["helix/", ...required],
    forbidden: [".git", ".github/", "node_modules/", "target/", ".DS_Store", ".tar.gz", "~"],
    expectations: {
      "helix/languages.toml": [
        "[language-server.vize]",
        'command = "vize"',
        'args = ["lsp"]',
        'name = "vue"',
        'scope = "source.vue"',
        'file-types = ["vue"]',
        'name = "art-vue"',
        'language-id = "art-vue"',
        'scope = "source.art-vue"',
        'file-types = [{ glob = "*.art.vue" }]',
        '"vite.config.ts"',
        '"vite.config.js"',
        '"tsconfig.json"',
        '"package.json"',
        '".git"',
        'language-servers = ["vize"]',
      ],
    },
  });
  console.log(
    `Helix package smoke passed: ${relativeArchivePath(result.archive)} (${result.entries.length} entries)`,
  );
});
