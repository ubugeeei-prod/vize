#!/bin/sh
if [ "$1" = "branch" ]; then echo "$FAKE_BRANCH"; exit; fi
if [ "$1" = "rev-parse" ]; then echo "$FAKE_LOCAL_SHA"; exit; fi
if [ "$FAKE_REMOTE" = "error" ]; then echo "credential detail that must stay hidden" >&2; exit 3; fi
if [ "$FAKE_REMOTE" != "missing" ]; then echo "$FAKE_REMOTE_SHA refs/heads/$FAKE_BRANCH"; fi
