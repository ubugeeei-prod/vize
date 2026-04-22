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
          config.allowUnfreePredicate = pkg: (pkg.pname or null) == "moonbit";
        };

        lib = pkgs.lib;
        workspaceCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        workspaceVersion = workspaceCargo.workspace.package.version;
        moonbitArtifacts = {
          aarch64-darwin = {
            version = "latest-2026-04-20";
            url = "https://cli.moonbitlang.com/binaries/latest/moonbit-darwin-aarch64.tar.gz";
            hash = "sha256-YMfegcuZa6lZ5oSbJDxh6nXIVgvX0vg1kjpP8IyKdek=";
          };
          x86_64-linux = {
            version = "latest-2026-04-20";
            url = "https://cli.moonbitlang.com/binaries/latest/moonbit-linux-x86_64.tar.gz";
            hash = "sha256-Q0DItKe3+CA/Y1wnPNZ7CplCxg+4UpEDZk6KDDfQliQ=";
          };
          aarch64-linux = {
            version = "latest-2026-04-20";
            url = "https://cli.moonbitlang.com/binaries/latest/moonbit-linux-aarch64.tar.gz";
            hash = "sha256-ASUvvvTxPUIpXRZL684qhYYTV8Wzch2WZ9y4e0CS9bw=";
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
              dontUnpack = true;
              dontConfigure = true;
              dontBuild = true;
              installPhase = ''
                mkdir -p $out
                tar -xzf $src -C $out
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
        nodejs = pkgs.nodejs_24;
        pnpm = pkgs.pnpm;
        workspaceVp = pkgs.writeShellApplication {
          name = "vp";
          runtimeInputs = [
            nodejs
            pnpm
          ];
          text = ''
            workspace_root="''${VIZE_WORKSPACE_ROOT:-$PWD}"
            if [ -x "$workspace_root/node_modules/.bin/vp" ]; then
              exec "$workspace_root/node_modules/.bin/vp" "$@"
            fi

            if [ -x "$PWD/node_modules/.bin/vp" ]; then
              exec "$PWD/node_modules/.bin/vp" "$@"
            fi

            cat >&2 <<'EOF'
            Local vite-plus is not installed.
            Run this inside the Nix shell:

              pnpm install --frozen-lockfile

            The flake intentionally avoids `pnpm dlx` so builds only use the locked workspace dependencies.
            EOF
            exit 127
          '';
        };
        rustToolchain = pkgs.rust-bin.stable."1.94.1".default.override {
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
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

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
          ++ lib.optionals (moonbit != null) [ moonbit ]
          ++ commonNativeBuildInputs;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            export VIZE_WORKSPACE_ROOT="$PWD"
            export PATH="$VIZE_WORKSPACE_ROOT/node_modules/.bin:$PATH"
            export PLAYWRIGHT_BROWSERS_PATH="$PWD/.cache/ms-playwright"
            export WASM_PACK_CACHE="$PWD/.cache/wasm-pack"
            ${lib.optionalString (moonbit != null) ''
              export MOON_HOME="${moonbit}"
              if [ -n "''${HOME:-}" ]; then
                PATH=":$PATH:"
                PATH="''${PATH//:$HOME/.moon/bin:/:}"
                PATH="''${PATH#:}"
                PATH="''${PATH%:}"
              fi
              export PATH="${moonbit}/bin:$PATH"
            ''}

            echo "Vize dev shell ready."
            echo "Nix provides Node, pnpm, Rust, wasm-pack, wasm-bindgen, binaryen, and MoonBit."
            ${lib.optionalString (moonbit == null) ''echo "MoonBit native toolchain is not available for ${system}; install it separately if needed."''}
            echo "Run: pnpm install --frozen-lockfile"
            echo "Then: vp build"
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
