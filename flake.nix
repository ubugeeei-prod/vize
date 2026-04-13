{
  description = "Vize development environment and CLI flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";

    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
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
        vitePlusVersion = "0.1.11";
        vp = pkgs.writeShellApplication {
          name = "vp";
          runtimeInputs = [
            pkgs.nodejs_24
            pkgs.pnpm
          ];
          text = ''
            exec pnpm dlx vite-plus@${vitePlusVersion} "$@"
          '';
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
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
        commonNativeBuildInputs =
          [ pkgs.pkg-config ]
          ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];

        vize = rustPlatform.buildRustPackage {
          pname = "vize";
          version = "0.39.0";
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
            vp
            rustToolchain
            pkgs.rust-analyzer
            pkgs.wasm-pack
            pkgs.wasm-bindgen-cli
            pkgs.cargo-insta
            pkgs.git
            pkgs.jq
          ] ++ commonNativeBuildInputs;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            export PATH="$PWD/node_modules/.bin:$PATH"
            export PLAYWRIGHT_BROWSERS_PATH="$PWD/.cache/ms-playwright"

            echo "Vize dev shell ready."
            echo "Nix provides the vp CLI."
            echo "Run: vp env install && vp install"
          '';
        };
      in
      {
        apps = {
          default = flake-utils.lib.mkApp { drv = vize; };
          vp = flake-utils.lib.mkApp { drv = vp; };
          vize = flake-utils.lib.mkApp { drv = vize; };
        };

        checks = {
          package = vize;
          shell = devShell.inputDerivation;
        };

        devShells.default = devShell;
        formatter = pkgs.nixfmt-rfc-style;

        packages = {
          default = vize;
          vp = vp;
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
