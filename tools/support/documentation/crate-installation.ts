/** Source-owned installation availability for Rust library documentation. */
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { parse } from "@iarna/toml";
import { publishedCrates } from "../release/publish_crates/main.ts";

const root = resolve(import.meta.dirname, "../../..");

/** Derive a crate command from its manifest and the canonical release inventory. */
export function crateDocumentationInstallation(crateName: string) {
  if (!/^[a-z][a-z0-9_]*$/u.test(crateName)) throw new Error(`Invalid crate name: ${crateName}`);
  const sourcePath = `crates/${crateName}`;
  const manifest = parse(readFileSync(resolve(root, sourcePath, "Cargo.toml"), "utf8"));
  const metadata = manifest.package as
    | { readonly name?: unknown; readonly publish?: unknown }
    | undefined;
  if (
    typeof metadata !== "object" ||
    metadata === null ||
    Array.isArray(metadata) ||
    metadata.name !== crateName
  )
    throw new Error(`Expected crate manifest: ${crateName}`);
  const policy = metadata.publish;
  if (
    policy !== undefined &&
    typeof policy !== "boolean" &&
    !(Array.isArray(policy) && policy.every((registry) => typeof registry === "string"))
  )
    throw new Error(`Unsupported crate publication policy: ${crateName}`);
  const published = publishedCrates.includes(crateName);
  if (published && (policy === false || (Array.isArray(policy) && !policy.includes("crates-io"))))
    throw new Error(`Release inventory contradicts crate publication policy: ${crateName}`);
  return published
    ? ({ ecosystem: "cargo", distribution: "registry", package: `cargo add ${crateName}` } as const)
    : ({
        ecosystem: "cargo",
        distribution: "workspace",
        package: `cargo add ${crateName} --path ../vize/${sourcePath}`,
        checkout: "../vize",
        sourcePath,
      } as const);
}
