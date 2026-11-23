#!/usr/bin/env bash
vp node "$VIZE_CI_FILES/bind-the-source-host-receipt-to-this-source-and-run.ts"
tar -C "$RUNNER_TEMP" -czf "$RUNNER_TEMP/instance-globals-10617/source-host.tar.gz" instance-globals-source-host
