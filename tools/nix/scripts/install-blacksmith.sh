# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/nix/scripts/install-blacksmith.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
runHook preInstall
install -Dm755 $src $out/bin/blacksmith
runHook postInstall
