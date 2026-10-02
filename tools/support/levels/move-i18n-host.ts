import {
  coreRoot,
  hostRoot,
  files,
  declarations,
  compiler,
  coreTests,
  hostTests,
  testMarker,
  wrappers,
  hostScope,
  changes,
  edges,
} from "./i18n-host-contract.ts";
import { preflight, lockPackage } from "./i18n-host-preflight.ts";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const root = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
const read = (path: string) => readFileSync(join(root, path), "utf8");
const exists = (path: string) => existsSync(join(root, path));
const write = (path: string, text: string) => writeFileSync(join(root, path), text);
const requireState = (condition: boolean, message: string) => {
  if (!condition) throw new Error(message);
};
function move() {
  const old = files.every((file) => exists(coreRoot + file) && !exists(hostRoot + file));
  const next = files.every((file) => !exists(coreRoot + file) && exists(hostRoot + file));
  requireState(old || next, "i18n source/target collision or partial move");
  requireState(
    read(coreRoot + "diag/catalog.rs").includes("pub trait MessageLookup"),
    "the portable message provider must exist before moving its host consumer",
  );
  if (old)
    for (const file of files) {
      mkdirSync(dirname(join(root, hostRoot + file)), { recursive: true });
      renameSync(join(root, coreRoot + file), join(root, hostRoot + file));
    }
}

function replace(path: string, before: string, after: string) {
  const text = read(path);
  if (text.includes(before)) write(path, text.replaceAll(before, after));
  else requireState(text.includes(after), `unexpected integration state: ${path}`);
}

function integrate() {
  requireState(
    files.every((file) => !exists(coreRoot + file) && exists(hostRoot + file)),
    "complete the pure i18n move first",
  );
  requireState(
    read("tests/tooling/support/davinci-i18n-host-imports.ts").includes("host_i18n_import") &&
      read("tests/tooling/support/davinci-host-imports.ts").includes("withoutI18nHostImports"),
    "the reviewed host import companion must accompany replay",
  );
  const updatedLock = preflight(read, exists);

  write(coreRoot + "lib.rs", read(coreRoot + "lib.rs").replace(declarations, ""));
  if (!read(hostRoot + "lib.rs").includes(declarations))
    replace(hostRoot + "lib.rs", "pub mod config;\n", "pub mod config;\n" + declarations);
  if (!read(hostRoot + "lib.rs").includes(hostScope))
    replace(hostRoot + "lib.rs", "pub mod i18n;", hostScope + "pub mod i18n;");
  replace(
    hostRoot + "lib.rs",
    "Canonical implementations live in `vize_l0`.",
    "Portable storage identities are re-exported from `vize_l0`.",
  );
  replace(
    hostRoot + "i18n/tests.rs",
    "use crate::diag::MessageLookup;",
    "use vize_l0::diag::MessageLookup;",
  );
  replace(
    hostRoot + "i18n.rs",
    "use crate::diag::MessageLookup;",
    "use vize_l0::diag::MessageLookup;",
  );
  replace(
    hostRoot + "i18n.rs",
    "use vize_l0::i18n::{Locale, Translator};",
    "use vize_carton::i18n::{Locale, Translator};",
  );
  replace(hostRoot + "i18n/catalog.rs", "use alloc::borrow::Cow;", "use std::borrow::Cow;");
  replace(hostRoot + "i18n/tests.rs", "use alloc::borrow::Cow;", "use std::borrow::Cow;");
  replace(
    hostRoot + "i18n/catalog.rs",
    "use crate::diag::MessageLookup;",
    "use vize_l0::diag::MessageLookup;",
  );
  write(
    compiler,
    read(compiler).replace("use crate::i18n::{Locale, translator};\n", "").replace(wrappers, ""),
  );
  replace(
    coreRoot + "compiler_error/codes.rs",
    "The catalog (`vize_l0::i18n`)",
    "The host translation catalog",
  );
  if (read(coreTests).includes(testMarker)) {
    const text = read(coreTests),
      at = text.indexOf(testMarker);
    write(
      hostTests,
      "use vize_l0::compiler_error::ErrorCode;\nuse super::{Locale, translator};\n\n" +
        text
          .slice(at)
          .replaceAll("crate::cstr!", "vize_l0::cstr!")
          .replace(
            "            assert_eq!(code.localized_message(locale).as_str(), message);\n",
            "",
          )
          .replace("            assert_eq!(code.localized_help(locale).as_str(), help);\n", ""),
    );
    write(coreTests, text.slice(0, at).replace("use crate::i18n::{Locale, translator};\n", ""));
  }
  if (!read(hostRoot + "i18n.rs").includes("mod compiler_error_tests;"))
    write(
      hostRoot + "i18n.rs",
      read(hostRoot + "i18n.rs") + "\n#[cfg(test)]\nmod compiler_error_tests;\n",
    );
  for (const [path, before, after] of changes) {
    if (read(path).includes(after) && after.includes(before)) continue;
    replace(path, before, after);
  }
  const relief = "crates/vize_relief/src/errors/diagnostic.rs";
  replace(relief, "use vize_carton::i18n::Locale;", "use vize_carton::i18n::{Locale, translator};");
  write(
    relief,
    read(relief)
      .replaceAll(
        "self.code.localized_message(locale)",
        "self.code.localized_message_with(&translator().for_locale(locale))",
      )
      .replaceAll(
        "self.code.localized_help(locale)",
        "self.code.localized_help_with(&translator().for_locale(locale))",
      )
      .replaceAll(
        "code.localized_message(Locale::En)",
        "code.localized_message_with(&translator().for_locale(Locale::En))",
      )
      .replaceAll(
        "code.localized_help(Locale::En)",
        "code.localized_help_with(&translator().for_locale(Locale::En))",
      )
      .replaceAll(
        "code.localized_message(locale)",
        "code.localized_message_with(&translator().for_locale(locale))",
      )
      .replaceAll(
        "code.localized_help(locale)",
        "code.localized_help_with(&translator().for_locale(locale))",
      ),
  );
  const catalog = "crates/vize/src/commands/explain/catalog.rs";
  const binding =
    "    pub(crate) fn messages(&self) -> impl vize_l0::diag::MessageLookup + '_ {\n        self.translator.for_locale(self.locale)\n    }\n\n";
  if (!read(catalog).includes("pub(crate) fn messages("))
    replace(
      catalog,
      "    pub(crate) const fn locale(",
      binding + "    pub(crate) const fn locale(",
    );
  const page = "crates/vize/src/commands/explain/page.rs";
  write(
    page,
    read(page)
      .replaceAll(
        "code.localized_message(catalog.locale())",
        "code.localized_message_with(&catalog.messages())",
      )
      .replaceAll(
        "code.localized_help(catalog.locale())",
        "code.localized_help_with(&catalog.messages())",
      ),
  );
  const wasm = "crates/vize_vitrine/src/wasm/lint.rs";
  write(
    wasm,
    read(wasm)
      .replaceAll("L0Locale", "TranslationLocale")
      .replaceAll("l0_locale", "translation_locale")
      .replace("Convert to L0 locale for i18n.", "Bind the host translation locale."),
  );
  const reader = "tests/tooling/davinci-diagnostic-catalog-sources.ts";
  write(
    reader,
    read(reader)
      .replaceAll(
        'read("davinci", "vize_l0", "src", file)',
        'read("crates", "vize_carton", "src", file)',
      )
      .replaceAll(
        'read("davinci", "vize_l0", "src", "i18n",',
        'read("crates", "vize_carton", "src", "i18n",',
      ),
  );
  write("Cargo.lock", updatedLock);
  check();
}

function check() {
  preflight(read, exists);
  requireState(
    files.every((file) => !exists(coreRoot + file) && exists(hostRoot + file)),
    "i18n must be owned by Carton",
  );
  requireState(
    !read(coreRoot + "lib.rs").includes("mod i18n") &&
      read(hostRoot + "lib.rs").includes(declarations),
    "the core must not export the host catalog",
  );
  requireState(
    !read(compiler).includes("crate::i18n") && !/locale: Locale|translator\(/u.test(read(compiler)),
    "L0 compiler messages must use only the caller-supplied provider",
  );
  requireState(
    read(hostRoot + "i18n/catalog.rs").includes("impl MessageLookup for LocaleMessages"),
    "the host must implement the real portable provider",
  );
  requireState(
    !read(coreTests).includes(testMarker) && exists(hostTests),
    "locale integration laws belong to the host",
  );
  for (const [path, before, after] of changes) {
    if (after)
      requireState(
        read(path).includes(after) ||
          (path.endsWith("diagnostic.rs") &&
            read(path).includes("use vize_carton::i18n::{Locale, translator};")),
        `unmigrated host caller: ${path}`,
      );
    if (!after.includes(before))
      requireState(!read(path).includes(before), `retired caller: ${path}`);
  }
  const lock = read("Cargo.lock");
  for (const [name, additions] of edges)
    for (const dependency of additions)
      requireState(
        lockPackage(lock, name).includes(` "${dependency}",\n`),
        `missing lock edge: ${name} -> ${dependency}`,
      );
  requireState(
    read("crates/vize/src/commands/explain/page.rs").includes(
      "code.localized_message_with(&catalog.messages())",
    ),
    "CLI explain must consume the portable provider",
  );
  requireState(
    read("crates/vize_relief/src/errors/diagnostic.rs")
      .replace(/\s+/gu, "")
      .includes("self.code.localized_message_with(&translator().for_locale(locale))"),
    "Relief must bind the host provider",
  );
}

switch (process.argv[2]) {
  case "moves":
    move();
    break;
  case "integrate":
    integrate();
    break;
  case "check":
    check();
    break;
  default:
    throw new Error("usage: vp node tools/support/levels/move-i18n-host.ts moves|integrate|check");
}
