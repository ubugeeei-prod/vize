import { readFileSync } from "node:fs";
process.stdout.write(
  JSON.parse(
    readFileSync(
      process.env.VIZE_TYPECHECK_REGRESSION_CAPTURE +
        "/outside_import_preserves_original_type_libraries_and_real_cli_diagnostics/oracle.json",
      "utf8",
    ),
  ).nativeBinary,
);
