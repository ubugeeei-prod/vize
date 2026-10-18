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
      nodejs = pkgs.nodejs_24;
      pnpm = pkgs.pnpm;

      # A Nix-provided Darwin stdenv has no Xcode behind it. Answer the two
      # queries the toolchain actually makes and defer the rest to the host
      # `xcrun` when the machine has one.
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

        shellHook = ''
          ${clearTestboxEnvironment}
          export VIZE_WORKSPACE_ROOT="$PWD"
          export PATH="${config.packages.vp}/bin:$VIZE_WORKSPACE_ROOT/node_modules/.bin:$PATH"
          export PLAYWRIGHT_BROWSERS_PATH="$PWD/.cache/ms-playwright"
          export WASM_PACK_CACHE="$PWD/.cache/wasm-pack"
          ${lib.optionalString pkgs.stdenv.hostPlatform.isDarwin ''
            export VIZE_DARWIN_LIBICONV_LIB="${pkgs.libiconv}/lib"
            export LIBRARY_PATH="$VIZE_DARWIN_LIBICONV_LIB''${LIBRARY_PATH:+:$LIBRARY_PATH}"
            export RUSTFLAGS="-L native=$VIZE_DARWIN_LIBICONV_LIB''${RUSTFLAGS:+ $RUSTFLAGS}"
          ''}

          echo "Vize dev shell ready."
          echo "Nix provides Node, pnpm, Rust, wasm-pack, wasm-bindgen, and binaryen."
          echo "Run: vp install --frozen-lockfile"
          echo "Local defaults: vp check / vp fmt / vp dev / vp build / vp test / vp lint"
        '';
      };

      testboxDevShell = devShell.overrideAttrs (previous: {
        nativeBuildInputs = (previous.nativeBuildInputs or [ ]) ++ [
          config.packages.blacksmith
          pkgs.gh
          pkgs.rsync
        ];
        shellHook = (previous.shellHook or "") + ''
          ${activateTestboxEnvironment}
          echo "Blacksmith Testbox tools ready (CLI ${config.packages.blacksmith.version})."
          echo "Pinned CLI: $VIZE_BLACKSMITH_BIN"
          echo 'First use: "$VIZE_BLACKSMITH_BIN" auth login'
          echo "After pushing the current HEAD, see CONTRIBUTING.md for warmup and safe ID capture."
          echo "Then: vp build:testbox / vp test:testbox / vp lint:testbox; stop with vp testbox:stop"
        '';
      });
    in
    {
      devShells = {
        default = devShell;
        testbox = testboxDevShell;
      };

      checks.shell = devShell.inputDerivation;
    };
}
