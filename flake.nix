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
        };

        lib = pkgs.lib;
        workspaceCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        workspaceVersion = workspaceCargo.workspace.package.version;
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
            pkgs.rust-analyzer
            pkgs.wasm-pack
            pkgs.wasm-bindgen-cli
            pkgs.binaryen
            pkgs.cargo-insta
            pkgs.git
            pkgs.jq
          ]
          ++ commonNativeBuildInputs;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            export VIZE_WORKSPACE_ROOT="$PWD"
            export PATH="$VIZE_WORKSPACE_ROOT/node_modules/.bin:$PATH"
            export PLAYWRIGHT_BROWSERS_PATH="$PWD/.cache/ms-playwright"
            export WASM_PACK_CACHE="$PWD/.cache/wasm-pack"

            echo "Vize dev shell ready."
            echo "Nix provides Node, pnpm, Rust, wasm-pack, wasm-bindgen, and binaryen."
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
        };
      }
    )
    // {
      overlays.default = final: prev: {
        vize = self.packages.${prev.system}.default;
      };
    };
}
