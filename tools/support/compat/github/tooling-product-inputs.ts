import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Audited product ownership for the original instance-global CLI campaign.
 *
 * Its source adapter copies npm/cli declarations and executes the source CLI.
 * The checkout's Node API declaration is a different producer. A bare "npm"
 * command in that campaign is not a read of every file in the npm/ directory.
 * Imports and explicit reads of the Node producer invalidate this independence.
 */
export type ToolingSourceCone = {
  complete: boolean;
  files: readonly string[];
  references: readonly string[];
  identity: string | undefined;
};
export type ToolingProductSelection = {
  required: boolean;
  paths: readonly string[];
};
const cliCampaign = "tests/tooling/content-mapper-instance-globals-10617.test.ts";
const nativeDeclaration = "npm/native/index.d.ts";
// Dependency audit of all 16 original consumer/helper modules, not a cached
// runtime verdict. A future body or import-set change restores qualification.
export const auditedCliCone = "6bccd518dc13e0d2a31a9f87dfa9e25e34aabcac067fa95eafa83529ce0c5a96";

export function toolingSourceConeIdentity(
  cwd: string,
  files: readonly string[],
): string | undefined {
  try {
    const names = [...new Set(files.map((file) => file.replaceAll("\\", "/")))].sort();
    if (names.some((file) => file.startsWith("/") || file.split("/").includes("..")))
      return undefined;
    const entries = names.map((file) => [file, readFileSync(join(cwd, file)).toString("base64")]);
    return createHash("sha256").update(JSON.stringify(entries)).digest("hex");
  } catch {
    return undefined;
  }
}
const readsNativeProducer = (reference: string): boolean => {
  const normalized = reference.replaceAll("\\", "/").replace(/^\.\//u, "");
  return (
    /^@vizejs\/native(?:[/-]|$)/u.test(normalized) ||
    normalized === nativeDeclaration ||
    normalized.startsWith("npm/native/") ||
    normalized === "npm/native" ||
    normalized.includes("/npm/native/") ||
    normalized.endsWith("/npm/native")
  );
};
const changesCliProducer = (file: string): boolean =>
  /^(?:Cargo\.(?:toml|lock)|rust-toolchain\.toml)$/u.test(file) ||
  /^(?:\.cargo|\.github|davinci|vendor)\//u.test(file) ||
  /^crates\/(?!vize_vitrine(?:\/|$))/u.test(file) ||
  /^npm\/cli(?:\/|$)/u.test(file) ||
  /^tests\/fixtures\/typechecker\/undeclared-instance-globals-10617(?:\/|$)/u.test(file) ||
  /^tools\/support\/compat\/github\/(?:source-cli[^/]*|plan-tooling-tests\.mjs|tooling-product-inputs\.ts)$/u.test(
    file,
  );

/** Return unmodified inputs unless the consumer/producer separation is known. */
export function toolingProductSelection(
  test: string,
  paths: readonly string[],
  readCone: () => ToolingSourceCone,
): ToolingProductSelection {
  if (test !== cliCampaign) return { required: false, paths };
  if (paths.some(changesCliProducer)) return { required: true, paths };
  if (!paths.includes(nativeDeclaration)) return { required: false, paths };
  const cone = readCone();
  // Unknown or newly coupled consumers must run, rather than inheriting an audit.
  const sourceCliAdapter =
    cone.references.includes("npm/cli") && cone.references.includes("content-mapper");
  if (
    !cone.complete ||
    cone.identity !== auditedCliCone ||
    !sourceCliAdapter ||
    cone.files.some(readsNativeProducer) ||
    cone.references.some(readsNativeProducer)
  )
    return { required: true, paths };
  return { required: false, paths: paths.filter((file) => file !== nativeDeclaration) };
}
