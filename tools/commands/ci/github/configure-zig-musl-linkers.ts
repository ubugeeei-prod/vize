import fs from "node:fs";
import path from "node:path";

import { isEntrypoint, runEntrypoint } from "../../../support/command-runtime.ts";

function requiredEnvironment(name: string): string {
  const value = process.env[name];
  if (value === undefined) throw new Error(`${name} is required`);
  return value;
}

function writeExecutable(target: string, contents: string): void {
  fs.writeFileSync(target, contents);
  fs.chmodSync(target, 0o755);
}

function configureTarget(
  linkerDirectory: string,
  zigAr: string,
  rustTarget: string,
  zigTarget: string,
): string {
  const cc = path.join(linkerDirectory, `zig-cc-${zigTarget}`);
  writeExecutable(
    cc,
    `#!/usr/bin/env bash
set -euo pipefail
args=()
skip_next=0
for arg in "$@"; do
  if (( skip_next )); then
    skip_next=0
    continue
  fi
  case "$arg" in
    --target=*)
      ;;
    --target)
      skip_next=1
      ;;
    *)
      args+=("$arg")
      ;;
  esac
done
exec zig cc -target ${zigTarget} "\${args[@]}"
`,
  );
  const targetVariable = rustTarget.toLowerCase();
  return [
    `CARGO_TARGET_${rustTarget}_LINKER=rust-lld`,
    `CC_${targetVariable}=${cc}`,
    `AR_${targetVariable}=${zigAr}`,
  ].join("\n");
}

export function configureZigMuslLinkers(arguments_: readonly string[]): number {
  if (arguments_.length > 0) {
    throw new Error(
      `Usage: node tools/commands/ci/github/configure-zig-musl-linkers.ts; unexpected argument ${arguments_[0]}`,
    );
  }
  const linkerDirectory = path.join(requiredEnvironment("RUNNER_TEMP"), "vize-zig-linkers");
  fs.mkdirSync(linkerDirectory, { recursive: true });
  const zigAr = path.join(linkerDirectory, "zig-ar");
  writeExecutable(zigAr, '#!/usr/bin/env bash\nexec zig ar "$@"\n');
  const variables = [
    configureTarget(linkerDirectory, zigAr, "X86_64_UNKNOWN_LINUX_MUSL", "x86_64-linux-musl"),
    configureTarget(linkerDirectory, zigAr, "AARCH64_UNKNOWN_LINUX_MUSL", "aarch64-linux-musl"),
  ].join("\n");
  fs.appendFileSync(requiredEnvironment("GITHUB_ENV"), `${variables}\n`);
  return 0;
}

if (isEntrypoint(import.meta.url)) {
  runEntrypoint(() => configureZigMuslLinkers(process.argv.slice(2)));
}
