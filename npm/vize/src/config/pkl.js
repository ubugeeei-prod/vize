import * as fs from "node:fs";
import * as path from "node:path";
import { execFileSync } from "node:child_process";
import { randomUUID } from "node:crypto";
import { fileURLToPath, pathToFileURL } from "node:url";

const PACKAGE_ROOT = path.resolve(fileURLToPath(new URL(".", import.meta.url)), "../..");

const DOCUMENTED_PKL_SCHEMA_IMPORT_RE =
  /^(\s*(?:amends|import)\s+)(["'])node_modules\/vize\/pkl\/(VizeConfig\.pkl|vize\.pkl)\2/gm;

/**
 * Evaluate a `vize.config.pkl` file and return its JSON representation.
 *
 * The npm-facing loader keeps PKL execution in Node because it needs package
 * resolution for `@pkl-community/pkl`, but all structural config normalization
 * happens in Rust after this function returns. A missing PKL runtime returns
 * `null` so config discovery can fall through to lower-priority formats; an
 * evaluation failure throws because a present PKL config should not silently be
 * ignored.
 */
export function loadPklConfigJson(filePath) {
  const pklBin = findPklBinary();
  if (!pklBin) {
    console.warn(
      "[vize] pkl CLI not found. Install @pkl-community/pkl or add pkl to PATH. " +
        "Falling back to the next config format.",
    );
    return null;
  }

  const patchedFilePath = createPklConfigWithBundledSchemaImports(filePath);
  const evalFilePath = patchedFilePath ?? filePath;
  try {
    return execFileSync(pklBin, ["eval", "-f", "json", evalFilePath], {
      cwd: path.dirname(filePath),
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "pipe"],
      timeout: 30_000,
    });
  } catch (error) {
    throw new Error(`Failed to evaluate vize PKL config at ${filePath}: ${getErrorMessage(error)}`);
  } finally {
    if (patchedFilePath) {
      fs.rmSync(patchedFilePath, { force: true });
    }
  }
}

function findPklBinary() {
  try {
    const pklPkgPath = import.meta.resolve?.("@pkl-community/pkl");
    if (pklPkgPath) {
      const pklLibDir = path.dirname(fileURLToPath(pklPkgPath));
      const pklPackageDir = path.dirname(pklLibDir);
      const candidates = [
        path.join(pklLibDir, "main.js"),
        path.join(pklPackageDir, "pkl"),
        path.join(pklPackageDir, "pkl.exe"),
      ];

      for (const candidate of candidates) {
        if (fs.existsSync(candidate)) {
          try {
            execFileSync(candidate, ["--version"], { stdio: "ignore" });
            return candidate;
          } catch {
            // Keep looking: the shim can exist when its runtime is unavailable.
          }
        }
      }
    }
  } catch {
    // Fall back to PATH below.
  }

  try {
    execFileSync("pkl", ["--version"], { stdio: "ignore" });
    return "pkl";
  } catch {
    return null;
  }
}

function createPklConfigWithBundledSchemaImports(filePath) {
  const configDir = path.dirname(filePath);
  const source = fs.readFileSync(filePath, "utf-8");
  let patched = false;

  const content = source.replace(
    DOCUMENTED_PKL_SCHEMA_IMPORT_RE,
    (match, prefix, quote, schemaFile) => {
      const projectSchemaPath = path.join(configDir, "node_modules", "vize", "pkl", schemaFile);
      if (fs.existsSync(projectSchemaPath)) {
        return match;
      }

      const bundledSchemaPath = path.join(PACKAGE_ROOT, "pkl", schemaFile);
      if (!fs.existsSync(bundledSchemaPath)) {
        return match;
      }

      patched = true;
      return `${prefix}${quote}${pathToFileURL(bundledSchemaPath).href}${quote}`;
    },
  );

  if (!patched) {
    return null;
  }

  const tempFile = path.join(
    configDir,
    `.vize-config-${process.pid}-${Date.now()}-${randomUUID()}.pkl`,
  );
  fs.writeFileSync(tempFile, content, { flag: "wx", mode: 0o600 });
  return tempFile;
}

function getErrorMessage(error) {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}
