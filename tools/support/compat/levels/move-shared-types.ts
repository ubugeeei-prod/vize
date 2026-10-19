import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import path from "node:path";

export function moveSharedTypes(root, mode) {
  const read = (file) => readFileSync(path.join(root, file), "utf8");
  const write = (file, source) => writeFileSync(path.join(root, file), source);
  const carton = "davinci/vize_l0/src/",
    substrate = "davinci/vize_davinci/src/";
  if (mode === "--shared-moves-only") {
    mkdirSync(path.join(root, carton, "compiler_error"), { recursive: true });
    for (const [before, after] of [
      [substrate + "stage.rs", carton + "stage.rs"],
      ["crates/vize_relief/src/errors/codes.rs", carton + "compiler_error/codes.rs"],
    ]) {
      if (!existsSync(path.join(root, after)))
        renameSync(path.join(root, before), path.join(root, after));
    }
    return;
  }
  if (existsSync(path.join(root, carton + "compiler_error.rs"))) return;
  const take = (source, start, end) => {
    const first = source.indexOf(start),
      last = source.indexOf(end, first);
    if (first < 0 || last < first) throw new Error(`shared type boundary changed: ${start}`);
    return [source.slice(first, last), source.slice(0, first) + source.slice(last)];
  };
  const errorsPath = "crates/vize_relief/src/errors.rs";
  const [codes, errors] = take(
    read(errorsPath),
    "/// Error codes for compiler errors",
    "/// Result type for compiler operations",
  );
  write(
    errorsPath,
    errors.replace("mod codes;\n", "pub use vize_l0::compiler_error::ErrorCode;\n"),
  );
  const diagnosticPath = "crates/vize_relief/src/errors/diagnostic.rs";
  const [methods, diagnostic] = take(
    read(diagnosticPath),
    "impl ErrorCode {",
    "impl CompilerError {",
  );
  write(
    diagnosticPath,
    diagnostic
      .replace("Exemption, PartKind, Stage", "Exemption, PartKind")
      .replace("use vize_l0::i18n::{Locale, translator};", "use vize_l0::i18n::Locale;")
      .replace("use vize_l0::{CompactString, Span, cstr};", "use vize_l0::{CompactString, Span};"),
  );
  const recoveryPath = "crates/vize_relief/src/errors/recovery.rs";
  const [recoveryMethods, recovery] = take(
    read(recoveryPath),
    "/// Parse error codes",
    "impl CompilerError {",
  );
  write(
    recoveryPath,
    recovery.replace(
      "use super::{CompilerError, ErrorCode};",
      "use super::CompilerError;\npub use vize_l0::compiler_error::RECOVERED_PARSE_CODES;",
    ),
  );
  write(
    carton + "compiler_error.rs",
    `//! Stable compiler codes shared with the preserved parser.
use crate::stage::Stage;
use crate::i18n::{Locale, translator};
use crate::{CompactString, cstr};
mod codes;
\n${codes}${methods}${recoveryMethods}`,
  );
  const corePath = "crates/vize_relief/src/relief/core.rs";
  const [namespace, core] = take(
    read(corePath),
    "/// Namespace for elements",
    "/// Constant type levels",
  );
  write(
    corePath,
    core.replace("use vize_l0::Span;", "pub use vize_l0::Namespace;\nuse vize_l0::Span;"),
  );
  write(
    carton + "namespace.rs",
    "//! Namespace shared by surface parsing and legacy ASTs.\nuse serde::{Deserialize, Serialize};\n\n" +
      namespace,
  );
  write(
    carton + "lib.rs",
    read(carton + "lib.rs").replace(
      "// Shared modules\n",
      "// Shared modules\npub mod compiler_error;\npub mod stage;\nmod namespace;\npub use compiler_error::ErrorCode;\npub use namespace::Namespace;\n",
    ),
  );
  write(
    substrate + "stage.rs",
    "//! Canonical stage identities shared with the foundation.\npub use vize_l0::stage::{ARTIFACT_STAGES, CONVERSIONS, ConversionCrate, L0, L1, L1_TO_L2, L2, L2_TO_L3, L3, LAYERS, LayerCrate, Stage, StageCrate, pipeline_display_id, pipeline_wire_id};\n",
  );
  const files = [
    "event.rs",
    "build.rs",
    "build/tag.rs",
    "parse.rs",
    "markup/lex/compat/dynamic_arg.rs",
    "markup/lex/compat/adapter.rs",
    "markup/lex/compat/empty_delimiter_tests.rs",
    "markup/lex/compat/tests.rs",
    "markup/lex/compat/types.rs",
    "markup/lex/compat/dynamic_arg/tests.rs",
    "markup/lex/compat/states.rs",
  ];
  for (const file of [
    ...files.map((file) => "davinci/vize_l1/src/" + file),
    "davinci/vize_l1/tests/surface_fidelity.rs",
  ]) {
    write(
      file,
      read(file)
        .replaceAll("vize_relief::ErrorCode", "vize_l0::ErrorCode")
        .replaceAll("vize_relief::Namespace", "vize_l0::Namespace"),
    );
  }
  const manifest = "davinci/vize_l1/Cargo.toml";
  write(manifest, read(manifest).replace("vize_relief = { workspace = true }\n", ""));
}
