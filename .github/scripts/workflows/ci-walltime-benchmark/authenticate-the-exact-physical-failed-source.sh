#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = 0b82a31fc9dd8634786fc17ec3bd3eeb8427ae6e
test "$(git rev-parse 'HEAD^{tree}')" = b865787dc492550b568d7cea3bcf458f75b11cfb
test -z "$(git status --porcelain --untracked-files=no)"
