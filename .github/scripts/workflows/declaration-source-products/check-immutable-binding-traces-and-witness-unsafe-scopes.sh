#!/usr/bin/env bash
set -euo pipefail
cd "$RUNNER_TEMP"
curl -fsSL --retry 3 -o literal-alloy.jar https://repo.maven.apache.org/maven2/org/alloytools/org.alloytools.alloy.dist/6.2.0/org.alloytools.alloy.dist-6.2.0.jar
echo "$ALLOY_SHA256  literal-alloy.jar" | sha256sum -c -
java -Djava.awt.headless=true -jar literal-alloy.jar exec -q -c '*' -t none -o literal-instances "$GITHUB_WORKSPACE/tests/formal/primitive-assignments/PreferConst.als"
jq -e '.commands | keys == ["LaterWrite","NonemptyTrace","RepeatedName","SameTrace"] and (.SameTrace | .type == "check" and (.solution | length) == 0) and ([.LaterWrite,.NonemptyTrace,.RepeatedName] | all(.type == "run" and (.solution | length) == 1))' literal-instances/receipt.json
