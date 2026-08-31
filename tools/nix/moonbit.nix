let
  root = ../..;
in
{
  perSystem =
    {
      lib,
      pkgs,
      system,
      ...
    }:
    let
      # Single source of truth for the MoonBit toolchain, shared with
      # `.github/actions/setup-moonbit` so the Nix shell and CI can never
      # install different compilers for the same commit.
      moonbitVersion = builtins.replaceStrings [ "\n" ] [ "" ] (
        builtins.readFile (root + /.moonbit-version)
      );
      moonbitUrlVersion = builtins.replaceStrings [ "+" ] [ "%2B" ] moonbitVersion;

      # Upstream publishes no x86_64-darwin binary, so that system gets a null
      # toolchain and the dev shell says so rather than failing to evaluate.
      moonbitArtifacts = {
        aarch64-darwin = {
          url = "https://cli.moonbitlang.com/binaries/${moonbitUrlVersion}/moonbit-darwin-aarch64.tar.gz";
          hash = "sha256-tHgaHjjIANH9ZWk7GXCy0kKfrvMdiTPSZqH24mk6lu8=";
        };
        x86_64-linux = {
          url = "https://cli.moonbitlang.com/binaries/${moonbitUrlVersion}/moonbit-linux-x86_64.tar.gz";
          hash = "sha256-NvXnzxVFWU4XzT8cC3V/5uhq0CGLyW9Bk2nLuFAuYro=";
        };
        aarch64-linux = {
          url = "https://cli.moonbitlang.com/binaries/${moonbitUrlVersion}/moonbit-linux-aarch64.tar.gz";
          hash = "sha256-4ZGoimZM9JQ3NoRtuJEIrHajpZK0QnpWYiGAwSoYrcU=";
        };
      };

      moonbit =
        if moonbitArtifacts ? ${system} then
          pkgs.stdenvNoCC.mkDerivation {
            pname = "moonbit";
            version = moonbitVersion;
            src = pkgs.fetchurl { inherit (moonbitArtifacts.${system}) url hash; };
            coreSrc = pkgs.fetchurl {
              name = "core-${moonbitVersion}.tar.gz";
              url = "https://cli.moonbitlang.com/cores/core-${moonbitUrlVersion}.tar.gz";
              hash = "sha256-BpItNd2U3Auup2gilf5ghPDYVO7TtJfNa1J/d9iZp/U=";
            };
            nativeBuildInputs = lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.patchelf ];
            dontUnpack = true;
            dontConfigure = true;
            dontBuild = true;
            installPhase = ''
              mkdir -p $out
              tar -xzf $src -C $out
              chmod -R a+rX,u+w $out
              find $out/bin -type f -exec chmod a+x {} +
              ${lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
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
    in
    {
      # `null` on systems upstream does not publish for; the dev shell reads
      # this argument rather than re-deriving support from `system`.
      _module.args = {
        inherit moonbit;
      };

      packages = lib.optionalAttrs (moonbit != null) { inherit moonbit; };
    };
}
