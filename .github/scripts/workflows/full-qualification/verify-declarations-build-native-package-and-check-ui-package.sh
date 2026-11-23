#!/usr/bin/env bash
vp run --filter './npm/fresco-native' check:generated && vp run --filter './npm/fresco-native' check:types && vp run --filter './npm/native' build:ci && vp run --filter './npm/ui' check
