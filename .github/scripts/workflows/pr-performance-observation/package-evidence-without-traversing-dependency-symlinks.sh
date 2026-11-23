#!/usr/bin/env bash
if [[ -d performance-observation ]]; then
  tar --exclude=node_modules --exclude=.vize --exclude=__out__ \
    -czf performance-observation.tgz performance-observation
fi
