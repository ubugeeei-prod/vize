#!/usr/bin/env bash
vize-ci-apt-retry sudo apt-get update -qq
vize-ci-apt-retry sudo apt-get install -y --no-install-recommends valgrind
