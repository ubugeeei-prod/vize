import assert from "node:assert/strict";
import { closeSync, lstatSync, openSync, readFileSync, readSync, realpathSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileImage } from "../computed_event_custody.ts";
import type { Image } from "./model.ts";
import type { Sink } from "./evidence.ts";
import { absoluteUnix } from "./pnpm-shim-render.ts";
import { sha256 } from "./value.ts";

// This is a closed, reviewed trusted-setup proxy program, not a claim about
// its generator or native writer. Only its final literal target varies.
// It is never used as input to fit the independently derived package shims.
const managerPrelude =
  '#!/bin/sh\n# Resolve $0 through symlinks so basedir is the shim\'s real directory.\n# Cap hops at the kernel\'s ELOOP limit so a cycle cannot hang the shim.\nlink="$0"\nhops=0\nwhile [ -L "$link" ] && [ "$hops" -lt 40 ]; do\n  hops=$((hops+1))\n  target=$(readlink "$link")\n  case "$target" in\n    /*) link="$target" ;;\n    *)  link="$(dirname "$link")/$target" ;;\n  esac\ndone\nbasedir=$(dirname "$(echo "$link" | sed -e \'s,\\\\,/,g\')")\nbasedir_win="$basedir"\nexe=""\nmsys=""\n\ncase `uname -a` in\n  *CYGWIN*|*MINGW*|*MSYS*)\n    if command -v cygpath > /dev/null 2>&1; then\n      basedir_win=`cygpath -w "$basedir"`\n    fi\n    exe=".exe"\n    msys="true"\n  ;;\n  *WSL2*)\n    if command -v wslpath > /dev/null 2>&1; then\n      basedir_win="$(wslpath -w "$basedir" 2> /dev/null)"\n      if [ $? -ne 0 ] || [ -z "$basedir_win" ]; then\n        basedir_win="$basedir"\n      else\n        exe=".exe"\n      fi\n    fi\n  ;;\nesac\n';

export interface RuntimePathReceipt {
  root: string;
  inheritedPathSha256: string;
  inheritedPathBytes: number;
  managerEntry: Image;
  managerExecutable: Image;
  managerWriterImageOwner: "UNKNOWN";
  actualWriterRuntimeAttribution: false;
  bundledNodeGyp: string | null;
  sourceExtraPaths: string[];
  execPaths: string[];
  scriptPaths: string[];
  selected: Array<{ route: "exec" | "script"; command: "node" | "sh"; image: Image }>;
}
function nativeMagic(path: string): void {
  const fd = openSync(path, "r"),
    bytes = Buffer.alloc(4);
  try {
    assert.equal(readSync(fd, bytes, 0, 4, 0), 4);
  } finally {
    closeSync(fd);
  }
  assert(
    ["7f454c46", "cffaedfe", "feedfacf", "cafebabe", "bebafeca"].includes(bytes.toString("hex")),
    "trusted-setup executable route is not a supported native image",
  );
}
function setupExecutable(entry: Image, sink: Sink): { unresolved: string; image: Image } {
  assert.deepEqual(fileImage(entry.path), entry);
  let path = entry.path;
  if (entry.bytes <= 4200) {
    const raw = readFileSync(entry.path);
    const source = new TextDecoder("utf8", { fatal: true }).decode(raw);
    const match =
      /\nexec "\$basedir\/([^"\n]+)"  "\$@"\nexit \$\?\n# cmd-shim-target=(\/[^\n]+)\n$/u.exec(
        source,
      );
    sink("trusted-setup-executable-route-before-assertion", {
      entry,
      terminalLiteralObserved: match !== null,
      wholeProgramSha256: sha256(raw),
      nativeWriterImageOwner: "UNKNOWN",
      actualWriterRuntimeAttribution: false,
    });
    assert(match, "trusted-setup target route is UNKNOWN");
    const relative = match[1]!,
      marker = absoluteUnix(match[2]!);
    assert(!/[\\\0\r\n"$`:]/u.test(relative));
    assert.equal(
      source,
      managerPrelude +
        '\nexec "$basedir/' +
        relative +
        '"  "$@"\nexit $?\n# cmd-shim-target=' +
        marker +
        "\n",
    );
    path = resolve(dirname(entry.path), relative);
    assert.equal(path, marker, "setup proxy marker differs from its actual literal exec target");
  }
  absoluteUnix(path);
  const stat = lstatSync(path);
  assert((stat.isFile() || stat.isSymbolicLink()) && (stat.mode & 0o111) !== 0);
  const canonical = realpathSync(path);
  nativeMagic(canonical);
  const image = fileImage(canonical);
  sink("observed-trusted-setup-native-target", {
    unresolved: path,
    image,
    bodyBytesExcluded: true,
    writerImageOwner: "UNKNOWN",
    actualWriterRuntimeAttribution: false,
  });
  return { unresolved: path, image };
}
function bundledPath(executable: { unresolved: string; image: Image }, sink: Sink): string | null {
  // c7 executor/bundled_node_gyp.rs: unresolved parent first, then canonical.
  // The actual installed path is an input observation, not writer provenance.
  for (const parent of [dirname(executable.unresolved), dirname(executable.image.path)]) {
    const directory = join(parent, "dist/node-gyp-bin"),
      path = join(directory, "node-gyp");
    const stat = lstatSync(path, { throwIfNoEntry: false });
    let regular = false;
    if (stat) regular = lstatSync(realpathSync(path)).isFile();
    sink("source-bundled-node-gyp-path", {
      path,
      present: stat !== undefined,
      regular,
      realPath: regular ? realpathSync(path) : null,
    });
    if (regular) return directory;
  }
  return null;
}
export function captureRuntimePaths(
  root: string,
  manager: Image,
  node: Image,
  env: NodeJS.ProcessEnv,
  sourceExtraPaths: string[],
  sink: Sink,
): RuntimePathReceipt {
  absoluteUnix(root);
  // c7's dependency ancestor bin walk contributes none for these source-owned
  // RUNNER_TEMP/consumer-KIND roots; do not invent arbitrary ancestor prefixes.
  assert(!root.split("/").includes("node_modules"));
  const inheritedRaw = env.PATH;
  assert(typeof inheritedRaw === "string" && inheritedRaw.length > 0);
  const inherited = inheritedRaw.split(":").map(absoluteUnix);
  const own = join(root, "node_modules/.bin");
  for (const name of ["node", "node.exe", "sh"]) {
    const path = join(own, name),
      stat = lstatSync(path, { throwIfNoEntry: false });
    sink("mandatory-local-runtime-before-assertion", { path, present: stat !== undefined });
    assert.equal(stat, undefined, "local runtime shadow is not authenticated");
  }
  const executable = setupExecutable(manager, sink),
    gyp = bundledPath(executable, sink);
  assert(sourceExtraPaths.every((path) => path === own));
  // c7 exec preserves prefix duplicates. Script extend_path preserves its
  // prefix too, filtering duplicate entries only from the inherited suffix.
  const execPrefix = [own, ...sourceExtraPaths];
  const scriptPrefix = [own, ...(gyp ? [gyp] : []), ...sourceExtraPaths];
  const execPaths = [...execPrefix, ...inherited];
  const scriptPaths = [...scriptPrefix, ...inherited.filter((p) => !scriptPrefix.includes(p))];
  const shell = fileImage("/bin/sh");
  assert.deepEqual(fileImage(node.path), node);
  const selected: RuntimePathReceipt["selected"] = [];
  for (const [route, paths] of [
    ["exec", execPaths],
    ["script", scriptPaths],
  ] as const)
    for (const [command, expected] of [
      ["node", node],
      ["sh", shell],
    ] as const) {
      let image: Image | null = null;
      for (const directory of paths) {
        const path = join(directory, command),
          stat = lstatSync(path, { throwIfNoEntry: false });
        if (!stat) continue;
        const canonical = realpathSync(path),
          actualStat = lstatSync(canonical);
        if (!actualStat.isFile() || (actualStat.mode & 0o111) === 0) continue;
        image = fileImage(canonical);
        sink("selected-source-runtime-before-assertion", { route, command, path, image, expected });
        assert.deepEqual(
          image,
          expected,
          "source-selected Node/shell runtime is not the authenticated image",
        );
        break;
      }
      assert(image, "source-selected runtime is UNKNOWN");
      selected.push({ route, command, image });
    }
  return {
    root,
    inheritedPathSha256: sha256(inheritedRaw),
    inheritedPathBytes: Buffer.byteLength(inheritedRaw),
    managerEntry: manager,
    managerExecutable: executable.image,
    managerWriterImageOwner: "UNKNOWN",
    actualWriterRuntimeAttribution: false,
    bundledNodeGyp: gyp,
    sourceExtraPaths,
    execPaths,
    scriptPaths,
    selected,
  };
}
