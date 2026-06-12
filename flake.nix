{
  description = "Vize development environment and CLI flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";

    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    let
      overlays = [ rust-overlay.overlays.default ];
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfreePredicate =
            pkg:
            builtins.elem (pkg.pname or null) [
              "blacksmith"
              "moonbit"
            ];
        };

        lib = pkgs.lib;
        workspaceCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        workspaceVersion = workspaceCargo.workspace.package.version;
        cratesIoIndex = "registry+https://github.com/rust-lang/crates.io-index";
        cratesIoNixIndexUrl = "https://vize.invalid/nix-crates.io-index";
        cratesIoNixIndex = "registry+${cratesIoNixIndexUrl}";
        nixCargoLockContents = builtins.replaceStrings [ cratesIoIndex ] [ cratesIoNixIndex ] (
          builtins.readFile ./Cargo.lock
        );
        moonbitArtifacts = {
          aarch64-darwin = {
            version = "0.1.20260608-2026-06-08";
            url = "https://cli.moonbitlang.com/binaries/latest/moonbit-darwin-aarch64.tar.gz";
            hash = "sha256-YAgKPu+uA4e2Z/UXJJpwt2ezdMANIjLuczet2ff3JIc=";
          };
          x86_64-linux = {
            version = "0.1.20260608-2026-06-08";
            url = "https://cli.moonbitlang.com/binaries/latest/moonbit-linux-x86_64.tar.gz";
            hash = "sha256-PFAX9WkoSALxoLhCLRSZY6AcpYfFFEURZK75HGXfiqg=";
          };
          aarch64-linux = {
            version = "0.1.20260608-2026-06-08";
            url = "https://cli.moonbitlang.com/binaries/latest/moonbit-linux-aarch64.tar.gz";
            hash = "sha256-fw4LyzobPWKaDV/9sattbQzjs2GXdRJ+wLhr/4dtVo4=";
          };
        };
        moonbit =
          if builtins.hasAttr system moonbitArtifacts then
            let
              artifact = moonbitArtifacts.${system};
            in
            pkgs.stdenvNoCC.mkDerivation {
              pname = "moonbit";
              inherit (artifact) version;
              src = pkgs.fetchurl {
                inherit (artifact) url hash;
              };
              coreSrc = pkgs.fetchurl {
                url = "https://cli.moonbitlang.com/cores/core-latest.tar.gz";
                hash = "sha256-LMOHyw9HPsEBp+BnOAAUnIhHgSFkRfBgTtXx4RDPc+o=";
              };
              nativeBuildInputs = lib.optionals pkgs.stdenv.isLinux [ pkgs.patchelf ];
              dontUnpack = true;
              dontConfigure = true;
              dontBuild = true;
              installPhase = ''
                mkdir -p $out
                tar -xzf $src -C $out
                chmod -R a+rX,u+w $out
                find $out/bin -type f -exec chmod a+x {} +
                ${lib.optionalString pkgs.stdenv.isLinux ''
                  for binary in $out/bin/*; do
                    if patchelf --print-interpreter "$binary" >/dev/null 2>&1; then
                      patchelf \
                        --set-interpreter "${pkgs.stdenv.cc.bintools.dynamicLinker}" \
                        --set-rpath "${
                          lib.makeLibraryPath [
                            pkgs.glibc
                            pkgs.stdenv.cc.cc.lib
                            pkgs.zlib
                            # `moonc` started dynamically linking libLLVM.so.18.1
                            # in the latest upstream binary; keep it on the rpath
                            # so the patched ELF can resolve it.
                            pkgs.llvmPackages_18.libllvm.lib
                          ]
                        }" \
                        "$binary"
                    fi
                  done
                ''}
                mkdir -p $out/lib
                tar -xzf $coreSrc -C $out/lib
                PATH=$out/bin:$PATH $out/bin/moon -C $out/lib/core bundle --warn-list -a --all
                PATH=$out/bin:$PATH $out/bin/moon -C $out/lib/core bundle --warn-list -a --target wasm-gc --quiet
              '';
              meta = {
                description = "MoonBit native toolchain";
                homepage = "https://www.moonbitlang.com/download/";
                license = lib.licenses.unfree;
                platforms = builtins.attrNames moonbitArtifacts;
              };
            }
          else
            null;
        # Blacksmith CLI (Testbox). Distributed as a single prebuilt binary
        # from a mutable `latest` channel, so — like the MoonBit block above —
        # each platform pins the current binary's hash. The placeholder hashes
        # below must be filled before this builds: run `nix build` once and
        # paste the `got: sha256-…` value Nix reports for each platform, or
        #   nix store prefetch-file --name blacksmith \
        #     https://clireleases.blacksmith.sh/cli/latest/<os>/<arch>/blacksmith
        #
        # NOTE: the CLI self-updates in the background on every invocation,
        # which cannot write into the read-only Nix store. If the pinned
        # binary errors while trying to self-update, gate the update off (check
        # `blacksmith --help` for the flag/env var) — the store path is the
        # source of truth and is refreshed by bumping the hashes here.
        blacksmithArtifacts = {
          aarch64-darwin = {
            url = "https://clireleases.blacksmith.sh/cli/latest/darwin/arm64/blacksmith";
            hash = "sha256-wTCMj9d5siTpB01ljPFa3NAwN28lJicaJcc7AiqTRWw=";
          };
          x86_64-darwin = {
            url = "https://clireleases.blacksmith.sh/cli/latest/darwin/amd64/blacksmith";
            hash = "sha256-aT3dpCOy00OyuEp/08z9pZxA31lZQEQ1MhRdNzwNgxM=";
          };
          x86_64-linux = {
            url = "https://clireleases.blacksmith.sh/cli/latest/linux/amd64/blacksmith";
            hash = "sha256-zzc5MFZaS2xvqSGyG1uuTFYFLQhoIb7FEzxp1c0prvk=";
          };
          aarch64-linux = {
            url = "https://clireleases.blacksmith.sh/cli/latest/linux/arm64/blacksmith";
            hash = "sha256-6XF/S/sn4s6zktPJbadkXt0yALLIySppZTUeNvoiWss=";
          };
        };
        blacksmith =
          if builtins.hasAttr system blacksmithArtifacts then
            let
              artifact = blacksmithArtifacts.${system};
            in
            pkgs.stdenvNoCC.mkDerivation {
              pname = "blacksmith";
              version = "latest";
              src = pkgs.fetchurl { inherit (artifact) url hash; };
              dontUnpack = true;
              nativeBuildInputs = lib.optionals pkgs.stdenv.isLinux [ pkgs.autoPatchelfHook ];
              installPhase = ''
                runHook preInstall
                install -Dm755 $src $out/bin/blacksmith
                runHook postInstall
              '';
              meta = {
                description = "Blacksmith CLI (Testbox)";
                homepage = "https://docs.blacksmith.sh/blacksmith-testbox/overview";
                license = lib.licenses.unfree;
                platforms = builtins.attrNames blacksmithArtifacts;
              };
            }
          else
            null;
        nodejs = pkgs.nodejs_24;
        pnpm = pkgs.pnpm;
        workspaceVp = pkgs.writeShellApplication {
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
                  check | fmt | dev | build)
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
        workspaceXcrun = pkgs.writeShellApplication {
          name = "xcrun";
          text = ''
            if [ "$#" -eq 3 ] && [ "$1" = "--sdk" ] && [ "$2" = "macosx" ] && [ "$3" = "--show-sdk-path" ]; then
              if [ -n "''${SDKROOT:-}" ]; then
                printf '%s\n' "$SDKROOT"
                exit 0
              fi
            fi

            if [ "$#" -eq 2 ] && [ "$1" = "-f" ]; then
              case "$2" in
                clang | clang++)
                  command -v "$2"
                  exit $?
                  ;;
              esac
            fi

            if [ -x /usr/bin/xcrun ]; then
              exec /usr/bin/xcrun "$@"
            fi

            printf 'unsupported xcrun invocation: %s\n' "$*" >&2
            exit 127
          '';
        };
        rustToolchain = pkgs.rust-bin.stable."1.95.0".default.override {
          extensions = [
            "clippy"
            "rust-src"
            "rustfmt"
          ];
          targets = [ "wasm32-unknown-unknown" ];
        };
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        commonNativeBuildInputs = [
          pkgs.pkg-config
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];

        vize = rustPlatform.buildRustPackage {
          pname = "vize";
          version = workspaceVersion;
          src = lib.cleanSource ./.;

          cargoLock = {
            lockFileContents = nixCargoLockContents;
            allowBuiltinFetchGit = true;
            extraRegistries = {
              # crates.io's API endpoint rejects fetchurl's default request
              # headers. Use a Nix-only registry alias so Cargo does not
              # define crates-io twice, then fetch the same tarballs from
              # the static endpoint.
              "${cratesIoNixIndexUrl}" = "https://static.crates.io/crates";
            };
          };
          postPatch = ''
            substituteInPlace Cargo.lock \
              --replace-fail ${lib.escapeShellArg cratesIoIndex} ${lib.escapeShellArg cratesIoNixIndex}
          '';

          cargoBuildFlags = [
            "-p"
            "vize"
          ];
          cargoTestFlags = [
            "-p"
            "vize"
          ];
          doCheck = false;
          nativeBuildInputs = commonNativeBuildInputs;

          meta = {
            description = "High-performance Vue.js toolchain in Rust";
            homepage = "https://vizejs.dev";
            license = lib.licenses.mit;
            mainProgram = "vize";
            platforms = lib.platforms.all;
          };
        };

        devShell = pkgs.mkShell {
          packages = [
            nodejs
            pnpm
            workspaceVp
            rustToolchain
            pkgs.git
            pkgs.rust-analyzer
            pkgs.wasm-pack
            pkgs.wasm-bindgen-cli
            pkgs.binaryen
            pkgs.cargo-insta
            pkgs.jq
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [ workspaceXcrun ]
          ++ lib.optionals (moonbit != null) [ moonbit ]
          ++ lib.optionals (blacksmith != null) [ blacksmith ]
          ++ commonNativeBuildInputs;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            export VIZE_WORKSPACE_ROOT="$PWD"
            export PATH="${workspaceVp}/bin:$VIZE_WORKSPACE_ROOT/node_modules/.bin:$PATH"
            export PLAYWRIGHT_BROWSERS_PATH="$PWD/.cache/ms-playwright"
            export WASM_PACK_CACHE="$PWD/.cache/wasm-pack"
            ${lib.optionalString pkgs.stdenv.isDarwin ''
              export VIZE_DARWIN_LIBICONV_LIB="${pkgs.libiconv}/lib"
              export LIBRARY_PATH="$VIZE_DARWIN_LIBICONV_LIB''${LIBRARY_PATH:+:$LIBRARY_PATH}"
              export RUSTFLAGS="-L native=$VIZE_DARWIN_LIBICONV_LIB''${RUSTFLAGS:+ $RUSTFLAGS}"
            ''}
            ${lib.optionalString (moonbit != null) ''
              export MOON_HOME="$VIZE_WORKSPACE_ROOT/.cache/moonbit"
              moonbit_version_file="$MOON_HOME/.nix-version"
              if [ ! -x "$MOON_HOME/bin/moon" ] || [ "$(cat "$moonbit_version_file" 2>/dev/null || true)" != "${moonbit.version}" ]; then
                rm -rf "$MOON_HOME/bin" "$MOON_HOME/include" "$MOON_HOME/lib"
                mkdir -p "$MOON_HOME"
                cp -R "${moonbit}/bin" "${moonbit}/include" "${moonbit}/lib" "$MOON_HOME"/
                chmod -R u+rwX "$MOON_HOME/bin" "$MOON_HOME/include" "$MOON_HOME/lib"
                printf '%s\n' "${moonbit.version}" > "$moonbit_version_file"
              fi
              if [ -n "''${HOME:-}" ]; then
                PATH=":$PATH:"
                PATH="''${PATH//:$HOME\/.moon\/bin:/:}"
                PATH="''${PATH#:}"
                PATH="''${PATH%:}"
              fi
              export PATH="$MOON_HOME/bin:$PATH"
            ''}

            echo "Vize dev shell ready."
            echo "Nix provides Node, pnpm, Rust, wasm-pack, wasm-bindgen, binaryen, and MoonBit."
            ${lib.optionalString (moonbit == null)
              ''echo "MoonBit native toolchain is not available for ${system}; install it separately if needed."''
            }
            echo "Run: vp install --frozen-lockfile"
            echo "Then: vp check / vp fmt / vp dev / vp build"
          '';
        };
      in
      {
        apps = {
          default = flake-utils.lib.mkApp { drv = vize; };
          vize = flake-utils.lib.mkApp { drv = vize; };
        };

        checks = {
          package = vize;
          shell = devShell.inputDerivation;
        };

        devShells.default = devShell;
        formatter = pkgs.nixfmt;

        packages = {
          default = vize;
          vize = vize;
        }
        // lib.optionalAttrs (moonbit != null) {
          moonbit = moonbit;
        };
      }
    )
    // {
      overlays.default = final: prev: {
        vize = self.packages.${prev.system}.default;
      };
    };
}
