# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/nix/scripts/restore-parser-sources.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rm -rf "$out/upstream/forks/oxc_parser" "$out/upstream/forks/oxc_parser_compat"
cp -R @PARSER@ "$out/upstream/forks/oxc_parser"
cp -R @PARSER_COMPAT@ "$out/upstream/forks/oxc_parser_compat"
chmod -R +w "$out/upstream/forks/oxc_parser" "$out/upstream/forks/oxc_parser_compat"
