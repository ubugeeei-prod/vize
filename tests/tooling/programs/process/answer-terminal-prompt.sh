#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/process/answer-terminal-prompt.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test -t 0 || exit 9
printf "Continue? "
read answer
echo "got:$answer"
exit 3
