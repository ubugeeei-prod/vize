#!/bin/bash
# Compare lint divergence for Vue 2 projects before/after the dialect harness change.
S=/private/tmp/claude-501/-Users-ubugeeei-Source-github-com-ubugeeei-prod-vize/a4027c37-ba23-4df3-b9d7-89015938e95f/scratchpad
WT=/Users/ubugeeei/Source/github.com/ubugeeei-prod/vize.rpm-lane
cd $WT
PROJECTS=$1; BEFORE=$2; AFTER=$3; BACK=$4
git checkout -q --detach $BEFORE && rm -rf $S/v2-before && /bin/bash $S/tools/local-lint.sh $S/v2-before '.' $PROJECTS
git checkout -q --detach $AFTER && rm -rf $S/v2-after && /bin/bash $S/tools/local-lint.sh $S/v2-after '.' $PROJECTS
git checkout -q $BACK
