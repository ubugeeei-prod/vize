#!/usr/bin/env bash
set -euo pipefail

linker_dir="$RUNNER_TEMP/vize-zig-linkers"
zig_ar="$linker_dir/zig-ar"

mkdir -p "$linker_dir"
printf '#!/usr/bin/env bash\nexec zig ar "$@"\n' > "$zig_ar"
chmod +x "$zig_ar"

write_cc() {
  local rust_target="$1"
  local zig_target="$2"
  local cc="$linker_dir/zig-cc-$zig_target"
  local env_target

  env_target="$(tr '[:upper:]' '[:lower:]' <<< "$rust_target")"
  printf '#!/usr/bin/env bash\nexec zig cc -target %s "$@"\n' "$zig_target" > "$cc"
  chmod +x "$cc"

  {
    printf 'CARGO_TARGET_%s_LINKER=%s\n' "$rust_target" "$cc"
    printf 'CC_%s=%s\n' "$env_target" "$cc"
    printf 'AR_%s=%s\n' "$env_target" "$zig_ar"
  } >> "$GITHUB_ENV"
}

write_cc X86_64_UNKNOWN_LINUX_MUSL x86_64-linux-musl
write_cc AARCH64_UNKNOWN_LINUX_MUSL aarch64-linux-musl
