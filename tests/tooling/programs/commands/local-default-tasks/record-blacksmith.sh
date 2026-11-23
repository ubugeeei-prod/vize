#!/bin/sh
printf '%s\n' "$*" >> "$FAKE_BLACKSMITH_LOG"
printf '%s\n' tbx_testbox_id
