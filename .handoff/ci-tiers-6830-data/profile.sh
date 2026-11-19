#!/bin/bash
# profile.sh <binary> <test-name> <outprefix>
B="$1"; T="$2"; O="$3"
cd "$(dirname "$0")/../../../../../../../Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-aa6fdbae04abd6574/crates/vize_canon" 2>/dev/null || cd /Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-aa6fdbae04abd6574/crates/vize_canon
start=$(date +%s)
"$B" "$T" --exact > "$O.out" 2>&1 &
pid=$!
sleep 15
sample "$pid" 10 -file "$O.sample" >/dev/null 2>&1
wait "$pid"
echo "exit=$? seconds=$(( $(date +%s) - start ))"
