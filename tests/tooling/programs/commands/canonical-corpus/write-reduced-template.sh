#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/canonical-corpus/write-reduced-template.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
printf '%s\n' "$@" > "@@OUTPUT@@/args.txt"
printf '<template><span/></template>' > "$6"
