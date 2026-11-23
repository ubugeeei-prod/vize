#!/usr/bin/env bash
git submodule update --init --depth 1 -- tests/_fixtures/_git/vue-element-admin
vp install --frozen-lockfile --prefer-offline
