#!/bin/sh
prefix=${1:--}
ulimit "${prefix}u" 2>/dev/null || ulimit "${prefix}p"
