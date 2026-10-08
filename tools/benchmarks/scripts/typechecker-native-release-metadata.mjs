/** Exact field rewrite mirrored from tools/support/release/pr_pin_metadata.rs. */
export function rewriteVersionMetadata(path, content, old, next) {
  const packageJson =
    path.endsWith("/package.json") &&
    (path.startsWith("npm/") ||
      ["editors/vscode/package.json", "editors/vscode-art/package.json"].includes(path));
  const readme = path === "README.md" || (path.startsWith("npm/") && path.endsWith("/README.md"));
  const cargoLock =
    ["Cargo.lock", "editors/zed/Cargo.lock", "examples/volt-target/Cargo.lock"].includes(path) ||
    (path.startsWith("davinci/vize_extension_host/tests/guests/") && path.endsWith("/Cargo.lock"));
  let section = "",
    pkg = "",
    replaced = false,
    benchmark = false,
    catalog = false;
  return content
    .split("\n")
    .map((line) => {
      const trimmed = line.trim();
      if (trimmed.startsWith("[") && trimmed.endsWith("]")) section = trimmed;
      if (cargoLock && trimmed === "[[package]]") pkg = "";
      if (cargoLock && /^name = ".*"$/u.test(trimmed)) pkg = trimmed.slice(8, -1);
      if (readme) {
        if (line.includes("<!-- benchmark:readme:start -->")) benchmark = true;
        const result = benchmark ? line : line.replaceAll(old, next);
        if (line.includes("<!-- benchmark:readme:end -->")) benchmark = false;
        return result;
      }
      if (
        path === "Cargo.toml" &&
        section === "[workspace.package]" &&
        trimmed === `version = "${old}"`
      )
        return line.replace(old, next);
      if (
        path === "Cargo.toml" &&
        section === "[workspace.dependencies]" &&
        (trimmed.startsWith("vize_") || line.includes('package = "vize_')) &&
        line.includes(`version = "=${old}"`)
      )
        return line.replaceAll(`version = "=${old}"`, `version = "=${next}"`);
      if (packageJson && !replaced && trimmed.startsWith('"version"') && line.includes(old)) {
        replaced = true;
        return line.replace(old, next);
      }
      if (
        ["editors/zed/Cargo.toml", "editors/zed/extension.toml"].includes(path) &&
        !replaced &&
        trimmed.startsWith('version = "') &&
        trimmed.endsWith('"') &&
        line.includes(old)
      ) {
        replaced = true;
        return line.replace(old, next);
      }
      if (
        cargoLock &&
        (pkg === "vize" ||
          pkg.startsWith("vize_") ||
          ["davinci_harness", "davinci_test_support", "vize-zed-extension"].includes(pkg)) &&
        trimmed === `version = "${old}"`
      )
        return line.replace(old, next);
      if (
        path === "pnpm-workspace.yaml" &&
        line.startsWith('  - "@vizejs/native-') &&
        line.includes(old)
      )
        return line.includes(next) ? line : line.replace(old, `${old} || ${next}`);
      if (["pnpm-workspace.yaml", "pnpm-lock.yaml"].includes(path)) {
        if (line === "  native-binaries:") catalog = true;
        else if (
          catalog &&
          line !== "" &&
          !line.startsWith("    ") &&
          !(path === "pnpm-workspace.yaml" && line.startsWith("  #"))
        )
          catalog = false;
        if (
          catalog &&
          line.includes(old) &&
          ((path === "pnpm-workspace.yaml" && line.startsWith('    "@vizejs/native-')) ||
            (path === "pnpm-lock.yaml" &&
              (line.startsWith("      specifier: ") || line.startsWith("      version: "))))
        )
          return line.replace(old, next);
      }
      return line;
    })
    .join("\n");
}

export function metadataCandidate(path) {
  return (
    [
      "Cargo.toml",
      "editors/zed/extension.toml",
      "editors/zed/Cargo.toml",
      "pnpm-workspace.yaml",
      "pnpm-lock.yaml",
    ].includes(path) ||
    ["Cargo.lock", "package.json", "README.md"].some((name) => path.endsWith(name))
  );
}
