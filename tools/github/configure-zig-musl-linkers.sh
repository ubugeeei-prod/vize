#!/usr/bin/env bash
set -euo pipefail

linker_dir="$RUNNER_TEMP/vize-zig-linkers"
zig_ar="$linker_dir/zig-ar"

mkdir -p "$linker_dir"
printf '#!/usr/bin/env bash\nexec zig ar "$@"\n' > "$zig_ar"
chmod +x "$zig_ar"

write_zig_wrapper() {
  local output="$1"
  local zig_target="$2"
  local mode="$3"

  {
    printf '#!/usr/bin/env bash\n'
    printf 'set -euo pipefail\n'
    printf 'args=()\n'
    printf 'skip_next=0\n'
    printf 'for arg in "$@"; do\n'
    printf '  if (( skip_next )); then\n'
    printf '    skip_next=0\n'
    printf '    continue\n'
    printf '  fi\n'
    printf '  case "$arg" in\n'
    printf '    --target=*)\n'
    printf '      ;;\n'
    printf '    --target)\n'
    printf '      skip_next=1\n'
    printf '      ;;\n'
    printf '    *)\n'
    printf '      args+=("$arg")\n'
    printf '      ;;\n'
    printf '  esac\n'
    printf 'done\n'
    if [[ "$mode" == "link" ]]; then
      printf 'exec zig cc -target %s "${args[@]}" -nostdlib -nostartfiles\n' "$zig_target"
    else
      printf 'exec zig cc -target %s "${args[@]}"\n' "$zig_target"
    fi
  } > "$output"
  chmod +x "$output"
}

write_cc() {
  local rust_target="$1"
  local zig_target="$2"
  local cc="$linker_dir/zig-cc-$zig_target"
  local linker="$linker_dir/zig-link-$zig_target"
  local env_target

  env_target="$(tr '[:upper:]' '[:lower:]' <<< "$rust_target")"
  write_zig_wrapper "$cc" "$zig_target" "compile"
  write_zig_wrapper "$linker" "$zig_target" "link"

  {
    printf 'CARGO_TARGET_%s_LINKER=%s\n' "$rust_target" "$linker"
    printf 'CC_%s=%s\n' "$env_target" "$cc"
    printf 'AR_%s=%s\n' "$env_target" "$zig_ar"
  } >> "$GITHUB_ENV"
}

write_cc X86_64_UNKNOWN_LINUX_MUSL x86_64-linux-musl
write_cc AARCH64_UNKNOWN_LINUX_MUSL aarch64-linux-musl
