/**
 * Install a built workspace package into a project the way npm lays out a
 * published one: a real directory holding the manifest's `files`, with its
 * dependencies beside it. A dependency the project already installed keeps
 * the project's copy; any other dependency links the workspace package's
 * resolution. A symlink to the source package instead would resolve the
 * package's own imports (helper `.d.ts` imports of `vue`, a module's
 * `@nuxt/kit`) from the source tree.
 */

import {
  cpSync,
  existsSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { dirname, join } from "node:path";

/** A symlinked scope (`node_modules/@vue -> store`) becomes a real dir of links, so adding to it never writes into the store. */
function ensureRealDirectory(dir: string): void {
  if (existsSync(dir) && lstatSync(dir).isSymbolicLink()) {
    const source = realpathSync(dir);
    rmSync(dir);
    mkdirSync(dir);
    for (const entry of readdirSync(source)) symlinkSync(join(source, entry), join(dir, entry));
  }
  mkdirSync(dir, { recursive: true });
}

/** Link `source` at `target` unless the project installed its own copy there. */
export function linkPackage(source: string, target: string): void {
  const present = lstatSync(target, { throwIfNoEntry: false });
  if (present && !present.isSymbolicLink()) return;
  if (present) rmSync(target);
  ensureRealDirectory(dirname(target));
  symlinkSync(source, target, "dir");
}

/** Replace `modules/<name>` with the package's published files and link its missing dependencies. */
export function installPackageFiles(sourceDir: string, modules: string, name: string): void {
  const target = join(modules, name);
  const manifest = JSON.parse(readFileSync(join(sourceDir, "package.json"), "utf8")) as {
    files: string[];
    dependencies?: Record<string, string>;
  };
  for (const entry of manifest.files) {
    if (!existsSync(join(sourceDir, entry))) {
      throw new Error(`${name} is not built (${join(sourceDir, entry)} missing)`);
    }
  }
  ensureRealDirectory(dirname(target));
  rmSync(target, { recursive: true, force: true });
  for (const entry of ["package.json", ...manifest.files]) {
    cpSync(join(sourceDir, entry), join(target, entry), { recursive: true });
  }
  for (const dependency of Object.keys(manifest.dependencies ?? {})) {
    linkPackage(
      realpathSync(join(sourceDir, "node_modules", dependency)),
      join(modules, dependency),
    );
  }
}
