#!/usr/bin/env bash
cpu_count() {
  if command -v nproc >/dev/null 2>&1; then
    nproc
  elif command -v sysctl >/dev/null 2>&1; then
    sysctl -n hw.ncpu 2>/dev/null || echo 1
  else
    echo 1
  fi
}
thread_count() {
  local count
  if [ -d /proc ]; then
    count="$(grep -sh '^Threads:' /proc/[0-9]*/status | awk '{sum += $2} END {print sum + 0}' || true)"
  else
    count="$(ps -axo nlwp= 2>/dev/null | awk '{sum += $1} END {print sum + 0}' || true)"
  fi
  echo "${count:-0}"
}
if [ "$RUNNER_ENVIRONMENT" = "github-hosted" ]; then
  # Temporary hosted-runner fallback while Blacksmith is degraded.
  # Restore the vue-parity job to blacksmith-32vcpu-ubuntu-2404 and remove these hosted caps/scales.
  export RAYON_NUM_THREADS=1
  export GOMAXPROCS=1
  export VIZE_CHECK_FIXTURES_BUDGET_CPU_FLOOR=12
  export VIZE_TIER_L_BUDGET_SCALE=2
  export VIZE_PERF_BUDGET_SCALE=2
else
  export RAYON_NUM_THREADS=4
fi
echo "RAYON_NUM_THREADS=$RAYON_NUM_THREADS" >> "$GITHUB_ENV"
if [ -n "${GOMAXPROCS:-}" ]; then
  echo "GOMAXPROCS=$GOMAXPROCS" >> "$GITHUB_ENV"
fi
if [ -n "${VIZE_CHECK_FIXTURES_BUDGET_CPU_FLOOR:-}" ]; then
  echo "VIZE_CHECK_FIXTURES_BUDGET_CPU_FLOOR=$VIZE_CHECK_FIXTURES_BUDGET_CPU_FLOOR" >> "$GITHUB_ENV"
fi
if [ -n "${VIZE_TIER_L_BUDGET_SCALE:-}" ]; then
  echo "VIZE_TIER_L_BUDGET_SCALE=$VIZE_TIER_L_BUDGET_SCALE" >> "$GITHUB_ENV"
fi
if [ -n "${VIZE_PERF_BUDGET_SCALE:-}" ]; then
  echo "VIZE_PERF_BUDGET_SCALE=$VIZE_PERF_BUDGET_SCALE" >> "$GITHUB_ENV"
fi
mkdir -p target/vize-tests/metrics/check-fixtures-topology
threads_total="$(thread_count)"
echo "VIZE_RUNNER_BASELINE_THREADS=$threads_total" >> "$GITHUB_ENV"
{
  echo "runner_environment=$RUNNER_ENVIRONMENT"
  echo "cpus=$(cpu_count)"
  echo "rayon_num_threads=$RAYON_NUM_THREADS"
  echo "gomaxprocs=${GOMAXPROCS:-unset}"
  echo "budget_cpu_floor=${VIZE_CHECK_FIXTURES_BUDGET_CPU_FLOOR:-unset}"
  echo "tier_l_budget_scale=${VIZE_TIER_L_BUDGET_SCALE:-unset}"
  echo "perf_budget_scale=${VIZE_PERF_BUDGET_SCALE:-unset}"
  echo "ulimit_u=$(ulimit -u)"
  echo "ulimit_Hu=$(ulimit -Hu)"
  echo "cgroup=$(cat /proc/self/cgroup 2>/dev/null | tr '\n' ' ' || echo unavailable)"
  echo "pids_current=$(cat /sys/fs/cgroup/pids.current 2>/dev/null || echo unavailable)"
  echo "pids_max=$(cat /sys/fs/cgroup/pids.max 2>/dev/null || echo unavailable)"
  echo "threads_total=$threads_total"
} > target/vize-tests/metrics/check-fixtures-topology/runner-baseline.txt
cat target/vize-tests/metrics/check-fixtures-topology/runner-baseline.txt
