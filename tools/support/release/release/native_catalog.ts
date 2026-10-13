import { replaceFirst } from "../command_runtime.ts";

/**
 * Rewrites every native binary catalog pin under the `native-binaries:` block
 * of `pnpm-workspace.yaml` to the new release version. It also keeps pnpm's
 * minimum-release-age allowlist aligned with those newly-published native
 * package versions.
 * The catalog block is recognised by the `  native-binaries:` header (two-space
 * indent) and runs until the next top-level or catalog-section header. Inside
 * that block the function replaces the version literal on every
 * `    "@vizejs/native-...": "<version>"` line. The block header itself plus
 * any comments are preserved untouched.
 */
export function replaceNativeBinaryWorkspaceCatalog(
  content: string,
  currentVersion: string,
  newVersion: string,
): string {
  const updated: string[] = [];
  let inCatalog = false;
  const header = "  native-binaries:";
  const pinPrefix = '    "@vizejs/native-';
  const minimumAgePrefix = '  - "@vizejs/native-';
  for (const line of content.split("\n")) {
    if (line.startsWith(minimumAgePrefix) && line.includes(currentVersion)) {
      updated.push(
        line.includes(newVersion)
          ? line
          : replaceFirst(line, currentVersion, `${currentVersion} || ${newVersion}`),
      );
      continue;
    }
    if (line === header) {
      inCatalog = true;
      updated.push(line);
      continue;
    }
    if (inCatalog) {
      const leavesBlock = line !== "" && !line.startsWith("    ") && !line.startsWith("  #");
      if (leavesBlock) {
        inCatalog = false;
      } else if (line.startsWith(pinPrefix) && line.includes(currentVersion)) {
        updated.push(replaceFirst(line, currentVersion, newVersion));
        continue;
      }
    }
    updated.push(line);
  }
  return updated.join("\n");
}

/**
 * Rewrites every native binary catalog pin under the `native-binaries:` block
 * inside `pnpm-lock.yaml`.
 * The catalogs section of pnpm-lock.yaml mirrors `pnpm-workspace.yaml` and each
 * native package appears as a two-line entry:
 * ```
 *     '@vizejs/native-darwin-arm64':
 *       specifier: 0.107.0
 *       version: 0.107.0
 * ```
 * We walk the file, detect entry into the `native-binaries:` block (under
 * `catalogs:` at the four-space indent), and rewrite the `specifier:` /
 * `version:` lines for every native package. The block ends when we either hit
 * a top-level YAML key or another catalog header at the same indent.
 */
export function replaceNativeBinaryLockfileCatalog(
  content: string,
  currentVersion: string,
  newVersion: string,
): string {
  const updated: string[] = [];
  let inBlock = false;
  const header = "  native-binaries:";
  for (const line of content.split("\n")) {
    if (line === header) {
      inBlock = true;
      updated.push(line);
      continue;
    }
    if (inBlock) {
      const leavesBlock = line !== "" && !line.startsWith("    ");
      if (leavesBlock) {
        inBlock = false;
      } else if (
        (line.startsWith("      specifier: ") || line.startsWith("      version: ")) &&
        line.includes(currentVersion)
      ) {
        updated.push(replaceFirst(line, currentVersion, newVersion));
        continue;
      }
    }
    updated.push(line);
  }
  return updated.join("\n");
}
