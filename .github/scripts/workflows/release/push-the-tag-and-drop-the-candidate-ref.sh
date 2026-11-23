#!/usr/bin/env bash
git push origin "HEAD:refs/tags/$RELEASE_TAG"
git push origin ":$RELEASE_CANDIDATE"
