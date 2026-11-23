import { readToolingProgram } from "./program-path.ts";
export const assertPublishManifestIsSanitized = readToolingProgram(
  "commands/publish-crates/check-manifest.cjs",
);
