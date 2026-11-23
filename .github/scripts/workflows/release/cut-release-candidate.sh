#!/usr/bin/env bash
node tools/support/release/release/main.ts cut ${RELEASE_VERSION:+"$RELEASE_VERSION"}
