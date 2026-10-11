import { createHash } from "node:crypto";
import * as fs from "node:fs";
import { createRequire } from "node:module";
import * as os from "node:os";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

interface CheckPacketInput {
  name: string;
  stdout: string;
  parsed: unknown;
  cwd: string;
  command: string;
  argv: readonly string[];
}

/** Retain the original answer before assertions, only in the dedicated qualifier. */
export function retainAppCheckPacket(input: CheckPacketInput): void {
  const destination = process.env.VIZE_APP_CHECK_CAPTURE_DIR;
  if (!destination) return;
  const directory = path.join(destination, input.name);
  fs.mkdirSync(path.join(directory, "bytes"), { recursive: true });
  fs.writeFileSync(path.join(directory, "stdout.json"), input.stdout);
  fs.writeFileSync(path.join(directory, "parsed.json"), JSON.stringify(input.parsed, null, 2));
  const files: Array<Record<string, unknown>> = [];
  const missing: Array<Record<string, unknown>> = [];
  const captured = new Set<string>();
  function retain(file: string, role: string): void {
    const absolute = path.resolve(file);
    const key = `${role}\0${absolute}`;
    if (captured.has(key)) return;
    captured.add(key);
    try {
      const identity = fs.lstatSync(absolute);
      const physical = fs.realpathSync(absolute);
      const bytes = fs.readFileSync(absolute);
      const sha256 = createHash("sha256").update(bytes).digest("hex");
      fs.writeFileSync(path.join(directory, "bytes", sha256), bytes);
      files.push({
        role,
        path: absolute,
        physical,
        sha256,
        size: bytes.length,
        symlink: identity.isSymbolicLink() ? fs.readlinkSync(absolute) : null,
        inode: identity.ino,
        device: identity.dev,
        modifiedMs: identity.mtimeMs,
        retainedBytes: `bytes/${sha256}`,
      });
    } catch (error) {
      missing.push({ role, path: absolute, error: String(error) });
    }
  }
  const testsRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  for (const owner of [testsRoot, path.dirname(testsRoot)]) {
    for (const name of ["package.json", "pnpm-lock.yaml"]) {
      const file = path.join(owner, name);
      if (fs.existsSync(file)) retain(file, "qualifier-dependency-contract");
    }
  }
  const require = createRequire(import.meta.url);
  const ts = require("typescript") as typeof import("typescript");
  const configs = new Set<string>();
  function retainConfig(file: string, role: string): void {
    const absolute = path.resolve(file);
    if (configs.has(absolute)) return;
    configs.add(absolute);
    retain(absolute, role);
    if (!fs.existsSync(absolute)) return;
    const result = ts.readConfigFile(absolute, (file) => ts.sys.readFile(file));
    if (result.error) {
      missing.push({
        role: "config-parse",
        path: absolute,
        error: {
          code: result.error.code,
          category: result.error.category,
          start: result.error.start,
          length: result.error.length,
          message: ts.flattenDiagnosticMessageText(result.error.messageText, "\n"),
        },
      });
      return;
    }
    const extensions: unknown = result.config?.extends;
    for (const target of Array.isArray(extensions) ? extensions : [extensions]) {
      if (typeof target !== "string") continue;
      try {
        let extended: string;
        if (target.startsWith(".") || path.isAbsolute(target)) {
          extended = path.resolve(path.dirname(absolute), target);
          if (!fs.existsSync(extended)) extended += ".json";
        } else {
          extended = createRequire(absolute).resolve(target);
        }
        retainConfig(extended, "authored-extends-config");
      } catch (error) {
        missing.push({
          role: "extends-resolution",
          config: absolute,
          target,
          error: String(error),
        });
      }
    }
  }
  const report = input.parsed as {
    programs?: Array<{ root: string; tsconfig?: string; files?: string[] }>;
    files?: Array<{ file: string }>;
  };
  const observations: Array<Record<string, unknown>> = [];
  for (const program of report.programs ?? []) {
    const reportedRoot = path.resolve(input.cwd, program.root);
    const selected = program.tsconfig ? path.resolve(input.cwd, program.tsconfig) : null;
    if (selected) retainConfig(selected, "reported-selected-config");
    let physicalRoot: string;
    try {
      physicalRoot = fs.realpathSync(reportedRoot);
    } catch (error) {
      missing.push({ role: "reported-program-root", path: reportedRoot, error: String(error) });
      continue;
    }
    // Literal source identity.rs schema; this observes its physical namespace.
    // The native process's selected argv/API is not inferred from this path.
    const key = createHash("sha256")
      .update("vize-canon-project-key:v1\0")
      .update(physicalRoot)
      .digest("hex");
    const cache = path.resolve(process.env.XDG_CACHE_HOME ?? path.join(os.homedir(), ".cache"));
    const temporary = path.resolve(os.tmpdir());
    const inside = (base: string): boolean =>
      base === physicalRoot || base.startsWith(`${physicalRoot}${path.sep}`);
    const base = !inside(cache)
      ? cache
      : !inside(temporary)
        ? temporary
        : path.join(path.dirname(physicalRoot), ".vize-canon-cache");
    const virtualRoot = path.join(base, "vize", "canon", "projects", key);
    let generated = path.join(virtualRoot, "tsconfig.json");
    if (selected && fs.existsSync(selected)) {
      const relative = path.relative(physicalRoot, path.dirname(fs.realpathSync(selected)));
      if (
        !relative.startsWith(`..${path.sep}`) &&
        relative !== ".." &&
        !path.isAbsolute(relative)
      ) {
        generated = path.join(virtualRoot, relative, "tsconfig.json");
      }
    }
    retainConfig(generated, "physical-generated-config-candidate");
    observations.push({
      reportedRoot,
      physicalRoot,
      reportedSelectedConfig: selected,
      virtualRoot,
      generatedConfigCandidate: generated,
      nativeSelectedConfigProved: false,
    });
    for (const file of program.files ?? [])
      retain(path.resolve(input.cwd, file), "reported-program-input");
    for (const name of ["package.json", "pnpm-lock.yaml", "package-lock.json", "yarn.lock"]) {
      const file = path.join(physicalRoot, name);
      if (fs.existsSync(file)) retain(file, "program-package-contract");
    }
    const manifest = path.join(physicalRoot, "package.json");
    if (fs.existsSync(manifest)) {
      const owner = JSON.parse(fs.readFileSync(manifest, "utf8"));
      for (const name of Object.keys({
        ...owner.dependencies,
        ...owner.devDependencies,
        ...owner.peerDependencies,
      })) {
        const installed = path.join(physicalRoot, "node_modules", name, "package.json");
        if (fs.existsSync(installed)) retain(installed, "installed-package-manifest-observation");
      }
    }
  }
  for (const file of report.files ?? [])
    retain(path.resolve(input.cwd, file.file), "reported-authored-source");
  fs.writeFileSync(
    path.join(directory, "receipt.json"),
    JSON.stringify(
      {
        name: input.name,
        command: input.command,
        authoredVizeArgv: input.argv,
        cwd: input.cwd,
        wholePrograms: report.programs,
        observations,
        files,
        missing,
        authorityUnproved: [
          "native exec-time selected config and normalized options",
          "native dependency-read identities beyond reported program inputs",
        ],
      },
      null,
      2,
    ),
  );
}
