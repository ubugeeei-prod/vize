import { runEntrypoint } from "../../../support/command-runtime.ts";
import {
  assertTarPackage,
  relativeArchivePath,
} from "../../../support/compat/editor-e2e/archive.ts";

const files = [
  "emacs/LICENSE",
  "emacs/README.md",
  "emacs/test/vize-test.el",
  "emacs/vize.el",
] as const;

runEntrypoint(() => {
  const result = assertTarPackage({
    defaultArchive: "emacs-vize-extension.tar.gz",
    label: "Emacs archive",
    root: "emacs",
    minimumBytes: 2_000,
    maximumBytes: 200_000,
    required: files,
    allowed: ["emacs/", "emacs/LICENSE", "emacs/README.md", "emacs/test/", ...files.slice(2)],
    forbidden: [
      ".git",
      ".github/",
      "node_modules/",
      "target/",
      ".DS_Store",
      ".elc",
      ".tar.gz",
      "~",
    ],
    expectations: {
      "emacs/vize.el": [
        "lexical-binding: t",
        `defcustom vize-eglot-command '("vize" "lsp")`,
        "defcustom vize-eglot-profile nil",
        "recommended . (:editor t :ecosystem t :lint t :typecheck t)",
        "define-derived-mode vize-vue-mode",
        "define-derived-mode vize-art-vue-mode",
        ":initializationOptions options",
        "eglot-server-programs",
        "provide 'vize",
      ],
      "emacs/test/vize-test.el": [
        "ert-deftest vize-eglot-default-program",
        ":initializationOptions",
        "ert-deftest vize-eglot-off-program",
      ],
    },
  });
  console.log(
    `Emacs package smoke passed: ${relativeArchivePath(result.archive)} (${result.entries.length} entries)`,
  );
});
