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
          config.packages.kache
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
          KACHE = "${config.packages.kache}";
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

    in
    {
      devShells = {
        default = devShell;
        testbox = testboxDevShell;
      };

      checks = {
        shell = devShell.inputDerivation;
      };
    };
}
