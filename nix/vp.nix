{
  perSystem =
    { pkgs, ... }:
    let
      nodejs = pkgs.nodejs_24;
      pnpm = pkgs.pnpm;
    in
    {
      packages.vp = pkgs.writeShellApplication {
        name = "vp";
        runtimeInputs = [
          nodejs
          pnpm
        ];
        text = ''
          workspace_root_input="''${VIZE_WORKSPACE_ROOT:-$PWD}"
          if workspace_root="$(cd "$workspace_root_input" 2>/dev/null && pwd -P)"; then
            :
          else
            workspace_root="$(pwd -P)"
          fi
          current_dir="$(pwd -P)"

          resolve_local_vp() {
            if [ -x "$workspace_root/node_modules/.bin/vp" ]; then
              printf '%s\n' "$workspace_root/node_modules/.bin/vp"
              return 0
            fi

            if [ -x "$PWD/node_modules/.bin/vp" ]; then
              printf '%s\n' "$PWD/node_modules/.bin/vp"
              return 0
            fi

            return 1
          }

          if local_vp="$(resolve_local_vp)"; then
            if [ "$current_dir" = "$workspace_root" ] && [ "$#" -eq 1 ]; then
              case "$1" in
                build | check | dev | fmt | lint | test | *:*)
                  exec "$local_vp" run --workspace-root "$1"
                  ;;
              esac
            fi

            exec "$local_vp" "$@"
          fi

          if [ "$#" -ge 1 ] && [ "$1" = "install" ]; then
            shift
            exec pnpm install "$@"
          fi

          cat >&2 <<'EOF'
          Local vite-plus is not installed.
          Run this inside the Nix shell:

            vp install --frozen-lockfile

          The flake intentionally avoids `pnpm dlx` so builds only use the locked workspace dependencies.
          EOF
          exit 127
        '';
      };
    };
}
