{
  perSystem =
    {
      lib,
      pkgs,
      system,
      ...
    }:
    let
      renderScript = import ./render-script.nix;
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
        installPhase = builtins.readFile ./scripts/install-blacksmith.sh;
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
      clearTestboxEnvironment = builtins.readFile ./scripts/clear-testbox.sh;
      activateTestboxEnvironment = renderScript ./scripts/set-testbox.sh {
        BLACKSMITH = "${blacksmith}";
      };

      testboxEnvironmentCheck = pkgs.runCommand "vize-testbox-environment" { } (
        renderScript ./scripts/check-testbox.sh {
          CLEAR_TESTBOX = clearTestboxEnvironment;
          ACTIVATE_TESTBOX = activateTestboxEnvironment;
          BLACKSMITH = "${blacksmith}";
        }
      );
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
