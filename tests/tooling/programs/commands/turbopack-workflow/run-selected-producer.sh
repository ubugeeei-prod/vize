#!/bin/sh
case "$*" in *turbopack-workflow.test.ts*) exit 0;; esac
printf "intentional producer failure\n"
exit 23
