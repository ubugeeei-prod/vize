{
  perSystem =
    {
      lib,
      pkgs,
      system,
      ...
    }:
    let
      # Blacksmith CLI (Testbox). Kept pinned to an exact release with
      # versioned artifact URLs — the vendor also publishes a moving `latest`
      # channel, and a fixed-output derivation pointed at it would silently
      # change what the shell installs.
      blacksmithVersion = "0.4.46";
      blacksmithArtifacts = {
        aarch64-darwin = {
          url = "https://clireleases.blacksmith.sh/cli/v${blacksmithVersion}/darwin/arm64/blacksmith";
          hash = "sha256-4DBbwkNpVHIzozeV2+hcJHlTG+jvlfWJzQd4C5JmQX4=";
        };
        x86_64-darwin = {
          url = "https://clireleases.blacksmith.sh/cli/v${blacksmithVersion}/darwin/amd64/blacksmith";
          hash = "sha256-XGVjxInIwYZXqyGhM8vzL5GkhWha3ZXkJ6mPDXgL3Cg=";
        };
        x86_64-linux = {
          url = "https://clireleases.blacksmith.sh/cli/v${blacksmithVersion}/linux/amd64/blacksmith";
          hash = "sha256-ABJrjw+yHuHcjPrEZNmRsCpn229Od87Hxja38i0CNVM=";
        };
        aarch64-linux = {
          url = "https://clireleases.blacksmith.sh/cli/v${blacksmithVersion}/linux/arm64/blacksmith";
          hash = "sha256-1RJ9+sW0pIteoODzb/8I6Qh3JYyyd+VoQcsh97PQmAI=";
        };
      };
      artifact = blacksmithArtifacts.${system} or (throw "blacksmith CLI: unsupported system ${system}");

      blacksmith = pkgs.stdenvNoCC.mkDerivation {
        pname = "blacksmith";
        version = blacksmithVersion;
        src = pkgs.fetchurl { inherit (artifact) url hash; };
        dontUnpack = true;
        nativeBuildInputs = lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.autoPatchelfHook ];
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
      };

      # The default shell must never look like the Testbox shell, so it clears
      # the markers rather than merely omitting them: `nix develop` entered
      # from inside `nix develop .#testbox` would otherwise inherit both.
      clearTestboxEnvironment = ''
        unset VIZE_TESTBOX_SHELL VIZE_BLACKSMITH_BIN
      '';
      activateTestboxEnvironment = ''
        export VIZE_TESTBOX_SHELL=1
        export VIZE_BLACKSMITH_BIN="${blacksmith}/bin/blacksmith"
      '';

      testboxEnvironmentCheck = pkgs.runCommand "vize-testbox-environment" { } ''
        export VIZE_TESTBOX_SHELL=inherited
        export VIZE_BLACKSMITH_BIN=/host/blacksmith
        ${clearTestboxEnvironment}
        test -z "''${VIZE_TESTBOX_SHELL:-}"
        test -z "''${VIZE_BLACKSMITH_BIN:-}"
        ${activateTestboxEnvironment}
        test "$VIZE_TESTBOX_SHELL" = 1
        test "$VIZE_BLACKSMITH_BIN" = "${blacksmith}/bin/blacksmith"
        test -x "$VIZE_BLACKSMITH_BIN"
        touch "$out"
      '';
    in
    {
      packages.blacksmith = blacksmith;

      # The two shell fragments travel with the CLI they guard, so the dev
      # shells cannot activate the markers without the pinned binary behind
      # them.
      _module.args = {
        inherit activateTestboxEnvironment clearTestboxEnvironment;
      };

      checks.testbox-environment = testboxEnvironmentCheck;
    };
}
