{
  perSystem =
    {
      activateTestboxEnvironment,
      clearTestboxEnvironment,
      config,
      lib,
      pkgs,
      rustDevToolchain,
      ...
    }:
    let
      renderScript = import ./render-script.nix;
      nodejs = pkgs.nodejs_24;
      pnpm = pkgs.pnpm;
      # A Nix-provided Darwin stdenv has no Xcode behind it. Answer the two
      # queries the toolchain actually makes and defer the rest to the host
      # `xcrun` when the machine has one.
      workspaceXcrun = pkgs.writeShellApplication {
        name = "xcrun";
        text = builtins.readFile ./scripts/xcrun.sh;
      };

      darwinEnvironment = lib.optionalString pkgs.stdenv.hostPlatform.isDarwin (
        renderScript ./scripts/activate-darwin.sh {
          LIBICONV = "${pkgs.libiconv}";
        }
      );

      devShell = pkgs.mkShell {
        packages = [
          nodejs
          pnpm
          config.packages.vp
          rustDevToolchain
          pkgs.git
          pkgs.rust-analyzer
          pkgs.wasm-pack
          pkgs.wasm-bindgen-cli
          pkgs.binaryen
          pkgs.cargo-insta
          pkgs.jq
          pkgs.pkg-config
        ]
        ++ lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
          workspaceXcrun
          pkgs.libiconv
        ];

        RUST_SRC_PATH = "${rustDevToolchain}/lib/rustlib/src/rust/library";

        shellHook = renderScript ./scripts/activate-workspace.sh {
          CLEAR_TESTBOX = clearTestboxEnvironment;
          VP = "${config.packages.vp}";
          DARWIN = darwinEnvironment;
        };
      };

      testboxDevShell = devShell.overrideAttrs (previous: {
        nativeBuildInputs = (previous.nativeBuildInputs or [ ]) ++ [
          config.packages.blacksmith
          pkgs.gh
          pkgs.rsync
        ];
        shellHook =
          (previous.shellHook or "")
          + (renderScript ./scripts/activate-testbox.sh {
            ACTIVATE_TESTBOX = activateTestboxEnvironment;
            BLACKSMITH_VERSION = config.packages.blacksmith.version;
          });
      });

      # `nix develop .#parallel-frontend`: opt-in for local check/clippy/test
      # loops only. rustc's parallel frontend is nightly-only
      # (`-Zunstable-options --jobs-frontend=N`); the default shell, CI, release
      # builds and measurements keep the pinned `rust-toolchain.toml` compiler
      # (docs/davinci/decisions/2026-11-17-rust-parallel-frontend.md). The flag
      # is appended so the Darwin link flags survive, and the target directory
      # is per thread count. Run the final clippy and tests in the default shell.
      parallelFrontendToolchain = pkgs.rust-bin.nightly."2026-11-16".default.override {
        extensions = [
          "clippy"
          "rust-src"
          "rustfmt"
        ];
      };

      parallelFrontendDevShell = devShell.overrideAttrs (previous: {
        nativeBuildInputs = [
          parallelFrontendToolchain
        ]
        ++ lib.remove rustDevToolchain (previous.nativeBuildInputs or [ ]);
        RUST_SRC_PATH = "${parallelFrontendToolchain}/lib/rustlib/src/rust/library";
        shellHook =
          (previous.shellHook or "") + (builtins.readFile ./scripts/activate-parallel-frontend.sh);
      });
    in
    {
      devShells = {
        default = devShell;
        testbox = testboxDevShell;
        parallel-frontend = parallelFrontendDevShell;
      };

      checks = {
        shell = devShell.inputDerivation;
      };
    };
}
