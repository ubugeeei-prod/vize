export const coreRoot = "davinci/vize_l0/src/";
export const hostRoot = "crates/vize_carton/src/";
const modules = [
  "i18n_compiler",
  "i18n_compiler_directive",
  "i18n_compiler_template",
  "i18n_croquis",
  "i18n_croquis_last",
  "i18n_croquis_more",
  "i18n_croquis_rest",
  "i18n_explain",
  "i18n_l3",
  "i18n_render",
  "i18n_rules_ecosystem",
  "i18n_rules_markup",
  "i18n_rules_script",
  "i18n_rules_script_more",
  "i18n_supplemental",
  "i18n_supplemental_extra",
  "i18n_supplemental_extra2",
  "i18n_supplemental_html",
];
export const files = [
  "i18n.rs",
  ...modules.map((name) => `${name}.rs`),
  ...["catalog.rs", "load.rs", "tests.rs", "en.json", "ja.json", "zh.json"].map(
    (name) => `i18n/${name}`,
  ),
];
export const declarations = "pub mod i18n;\n" + modules.map((name) => `mod ${name};\n`).join("");
export const compiler = coreRoot + "compiler_error.rs";
export const coreTests = coreRoot + "compiler_error/tests.rs";
export const hostTests = hostRoot + "i18n/compiler_error_tests.rs";
export const testMarker =
  "#[test]\nfn all_existing_code_and_locale_wrappers_keep_the_catalog_text()";
export const wrappers = `    /// The catalogued headline for this code in \`locale\`.
    #[must_use]
    pub fn localized_message(self, locale: Locale) -> CompactString {
        self.localized_message_with(&translator().for_locale(locale))
    }

    /// The catalogued remedy for this code in \`locale\`.
    #[must_use]
    pub fn localized_help(self, locale: Locale) -> CompactString {
        self.localized_help_with(&translator().for_locale(locale))
    }

`;
export const hostScope = `#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "host translation parsing retains its existing std string and unknown-key contract"
)]
`;
export const calls: [string, string, string][] = [
  ...["message", "help"].map((kind): [string, string, string] => [
    "crates/vize/src/commands/explain/page.rs",
    `code.localized_${kind}(catalog.locale())`,
    `code.localized_${kind}_with(&catalog.messages())`,
  ]),
  ...["self.code", "code"].flatMap((receiver) =>
    (receiver === "code" ? ["locale", "Locale::En"] : ["locale"]).flatMap((locale) =>
      ["message", "help"].map((kind): [string, string, string] => [
        "crates/vize_relief/src/errors/diagnostic.rs",
        `${receiver}.localized_${kind}(${locale})`,
        `${receiver}.localized_${kind}_with(&translator().for_locale(${locale}))`,
      ]),
    ),
  ),
];
const directConsumers = [
  "crates/vize/src/commands/explain/tests.rs",
  "crates/vize/src/commands/explain/catalog.rs",
  "crates/vize/src/commands/explain/page.rs",
  "crates/vize/src/commands/lint/rich.rs",
  "crates/vize/src/commands/lint/cross_file/component.rs",
  "crates/vize/tests/diagnostic_render.rs",
  "crates/vize/tests/diagnostic_render/cases/markup.rs",
  "crates/vize/tests/diagnostic_render/cases.rs",
  "crates/vize_relief/src/errors/diagnostic.rs",
  "crates/vize_patina/src/lib.rs",
  "crates/vize_patina/src/linter/engine/sfc/facade/tests/content_model.rs",
  "crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for/tests.rs",
  "crates/vize_canon/src/lib.rs",
  "crates/vize_vitrine/src/wasm/lint.rs",
];
export const changes: [string, string, string][] = directConsumers.map((path) => [
  path,
  "vize_l0::i18n",
  "vize_carton::i18n",
]);
changes.push(
  ["crates/vize/src/render/catalog.rs", "`vize_l0::i18n::Translator`", "the host translator"],
  ["crates/vize_patina/src/context.rs", "    i18n::{Locale, t, t_fmt},\n", ""],
  [
    "crates/vize_patina/src/context.rs",
    "use vize_croquis::Croquis;",
    "use vize_carton::i18n::{Locale, t, t_fmt};\nuse vize_croquis::Croquis;",
  ],
  [
    "crates/vize_patina/src/linter/config.rs",
    "use vize_l0::{FxHashMap, FxHashSet, String, i18n::Locale};",
    "use vize_carton::i18n::Locale;\nuse vize_l0::{FxHashMap, FxHashSet, String};",
  ],
  [
    "crates/vize_relief/Cargo.toml",
    "[dependencies]\n",
    "[dependencies]\nvize_carton.workspace = true\n",
  ],
  [
    "crates/vize_vitrine/Cargo.toml",
    "[dependencies]\n",
    "[dependencies]\nvize_carton.workspace = true\n",
  ],
  [
    "crates/vize_carton/Cargo.toml",
    "[dependencies]\n",
    "[dependencies]\nonce_cell.workspace = true\nrustc-hash.workspace = true\n",
  ],
);

export const edges: [string, string[]][] = [
  ["vize_carton", ["once_cell", "rustc-hash"]],
  ["vize_relief", ["vize_carton"]],
  ["vize_vitrine", ["vize_carton"]],
];
