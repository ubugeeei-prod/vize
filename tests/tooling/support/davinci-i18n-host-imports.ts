/** Exact imports at the existing locale/catalog host callers. */
export const i18nHostImports: Record<string, readonly string[]> = {
  "crates/vize/src/commands/explain/tests.rs": ["use vize_carton::i18n::Locale;"],
  "crates/vize/src/commands/explain/catalog.rs": [
    "use vize_carton::i18n::{Locale, Translator, translator};",
  ],
  "crates/vize/src/commands/explain/page.rs": ["use vize_carton::i18n::Locale;"],
  "crates/vize/src/commands/lint/rich.rs": [
    "use vize_carton::i18n::{Locale, Translator, translator};",
    "use vize_carton::i18n::Locale;",
  ],
  "crates/vize/src/commands/lint/cross_file/component.rs": [
    "use vize_carton::i18n::{Locale, t, t_fmt};",
  ],
  "crates/vize/tests/diagnostic_render.rs": ["use vize_carton::i18n::{Locale, translator};"],
  "crates/vize/tests/diagnostic_render/cases/markup.rs": ["use vize_carton::i18n::Locale;"],
  "crates/vize/tests/diagnostic_render/cases.rs": ["use vize_carton::i18n::Locale;"],
  "crates/vize_relief/src/errors/diagnostic.rs": ["use vize_carton::i18n::{Locale, translator};"],
  "crates/vize_patina/src/lib.rs": ["pub use vize_carton::i18n::Locale;"],
  "crates/vize_patina/src/context.rs": ["use vize_carton::i18n::{Locale, t, t_fmt};"],
  "crates/vize_patina/src/linter/config.rs": ["use vize_carton::i18n::Locale;"],
  "crates/vize_patina/src/linter/engine/sfc/facade/tests/content_model.rs": [
    "use vize_carton::i18n::Locale;",
  ],
  "crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for/tests.rs": [
    "use vize_carton::i18n::Translator;",
  ],
  "crates/vize_patina/tests/native_syntax_img_alt.rs": [
    "use vize_carton::i18n::{Locale, translator};",
  ],
  "crates/vize_patina/tests/native_syntax_img_alt/support.rs": [
    "use vize_carton::i18n::{Locale, translator};",
  ],
  "crates/vize_canon/src/lib.rs": ["pub use vize_carton::i18n::Locale;"],
  "crates/vize_canon/src/diagnostic.rs": ["use vize_carton::i18n::{Locale, t, t_fmt};"],
  "crates/vize_vitrine/src/wasm/lint.rs": [
    "use vize_carton::i18n::{Locale as TranslationLocale, t_fmt};",
  ],
};

export function withoutI18nHostImports(source: string, file: string): string {
  let storage = source;
  for (const declaration of i18nHostImports[file] ?? []) {
    const escaped = declaration.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    storage = storage.replace(new RegExp(`^\\s*${escaped}$`, "gmu"), "host_i18n_import");
  }
  return storage;
}
