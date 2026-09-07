#!/usr/bin/env bash
#
# scf_watcher.sh -- task-agnostic early-stop watcher for CASTEP jobs.
#
# Runs from the task workdir (the framework periodic hook sets the cwd).
# It tails <seed>.castep, tracks SCF cycles and energy gains, and flags
# the two stall signatures observed in long CeO2 geometry runs:
#
#   1. Cycle cap: a single BFGS SCF block consumed more than MAX_SCF
#      cycles without converging.
#   2. Oscillation: the minimum |energy gain| over the last WINDOW ticks
#      is still above TOL_MULT x TOL, i.e. the gain oscillates around
#      the convergence tolerance instead of decreasing.
#
# On first detection it writes .scf_stall_detected.<seed> plus a log
# line, and optionally terminates the run:
#
#   - SLURM_JOBID set  -> scancel
#   - KILL_PATTERN set -> pkill -TERM -f (local runs)
#
# The script is task-agnostic: SinglePoint, GeometryOptimization, and
# band-structure seeds all produce the same "<-- SCF" table format.
# Geometry runs reset the cycle counter at each BFGS block boundary;
# single-point runs count the whole SCF table.
#
# Usage:
#   bash scf_watcher.sh --seed NiO
#
# Optional overrides come from .scf_watch.conf in the workdir
# (KEY=VALUE lines, sourced as bash):
#   MAX_SCF      max SCF cycles per BFGS block (default 500)
#   WINDOW       tick window for the oscillation check (default 50)
#   TOL_MULT     stall multiplier on the tolerance (default 3)
#   TOL          elec_energy_tol from the .param file (default 1e-05)
#   KILL_PATTERN pkill -f pattern for local runs (optional)
#   SLURM_JOBID  scancel target for queued runs (optional)
#
# Exit code is always 0: the hook is a report channel, and termination
# is a side effect, not the exit status.

set -u

SEED=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --seed) SEED="$2"; shift 2 ;;
    *) echo "scf_watcher: unknown arg: $1" >&2; exit 0 ;;
  esac
done
if [[ -z "$SEED" ]]; then
  echo "scf_watcher: --seed required" >&2
  exit 0
fi

MAX_SCF=500
WINDOW=50
TOL_MULT=3
TOL=1e-05
KILL_PATTERN=""
SLURM_JOBID=""
CONF=".scf_watch.conf"
if [[ -f "$CONF" ]]; then
  # shellcheck disable=SC1090
  source "$CONF"
fi

CASTEP_FILE="${SEED}.castep"
STATE=".scf_watch.${SEED}.state"
GAINS=".scf_watch.${SEED}.gains"
MARKER=".scf_stall_detected.${SEED}"
LOG=".scf_watch.log"

log() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >> "$LOG"
}

# Nothing to watch yet: CASTEP has not opened the output file.
if [[ ! -f "$CASTEP_FILE" ]]; then
  exit 0
fi

# Latest SCF cycle line and the latest BFGS iteration boundary.
last_line=$(tail -c 4000000 "$CASTEP_FILE" 2>/dev/null | \
  rg -o '[0-9]+[[:space:]]+-?[0-9.eE+-]+[[:space:]]+-?[0-9.eE+-]+[[:space:]]+-?[0-9.eE+-]+[[:space:]]+[0-9.]+[[:space:]]+<-- SCF' \
  | tail -1 || true)
if [[ -z "$last_line" ]]; then
  exit 0
fi
cycle=$(awk '{print $1}' <<< "$last_line" | tr -d ' ')
gain=$(awk '{print $4}' <<< "$last_line")

bfgs=$(tail -c 4000000 "$CASTEP_FILE" 2>/dev/null | \
  rg 'BFGS: (starting|finished) iteration' | tail -1 \
  | awk '{for(i=1;i<=NF;i++) if($i=="iteration"){print $(i+1); break}}' || true)
if [[ -z "$bfgs" ]]; then
  bfgs=0
fi

# State: previous tick's (cycle, bfgs block, block start cycle).
prev_cycle=0
prev_bfgs=$bfgs
block_start_cycle=0
if [[ -f "$STATE" ]]; then
  read -r prev_cycle prev_bfgs block_start_cycle < "$STATE" 2>/dev/null || true
fi

if [[ -z "$prev_cycle" || -z "$prev_bfgs" || -z "$block_start_cycle" ]]; then
  prev_cycle=0
  prev_bfgs=$bfgs
  block_start_cycle=0
fi

# BFGS boundary: the SCF cycle counter restarts; open a new block.
if [[ "$bfgs" != "$prev_bfgs" ]]; then
  block_start_cycle=$cycle
  prev_cycle=$cycle
  : > "$GAINS"
  printf '%s %s %s\n' "$cycle" "$bfgs" "$cycle" > "$STATE"
  exit 0
fi

# No progress since the last tick (same cycle): skip the gain window.
if [[ "$cycle" -le "$prev_cycle" ]]; then
  exit 0
fi

# Record the absolute energy gain; keep only the WINDOW-line tail.
gain_abs=$(awk -v g="$gain" 'BEGIN{g+=0; if(g<0) g=-g; printf "%.10g\n", g}')
echo "$gain_abs" >> "$GAINS"
tail -n "$WINDOW" "$GAINS" > "${GAINS}.tmp"
mv "${GAINS}.tmp" "$GAINS"
printf '%s %s %s\n' "$cycle" "$bfgs" "$block_start_cycle" > "$STATE"

block_cycles=$(( cycle - block_start_cycle ))
if [[ $block_cycles -lt 0 ]]; then
  block_cycles=$cycle
fi

stalled=""

# Check 1: per-block cycle cap.
if [[ $block_cycles -ge $MAX_SCF ]]; then
  stalled="BFGS block ${bfgs} consumed ${block_cycles} SCF cycles (cap ${MAX_SCF}) without converging"
fi

# Check 2: oscillation stall on the gain window.
n_gains=$(wc -l < "$GAINS" 2>/dev/null | tr -d ' ')
if [[ -z "$stalled" && $n_gains -ge $WINDOW ]]; then
  min_gain=$(sort -g "$GAINS" | head -1)
  threshold=$(awk -v t="$TOL" -v m="$TOL_MULT" 'BEGIN{printf "%.10g", t*m}')
  if awk -v g="$min_gain" -v th="$threshold" 'BEGIN{exit !(g>th)}'; then
    stalled="min |gain| ${min_gain} over last ${n_gains} ticks exceeds threshold ${threshold} (SCF oscillation stall, BFGS block ${bfgs})"
  fi
fi

if [[ -n "$stalled" && ! -f "$MARKER" ]]; then
  echo "$stalled (BFGS block ${bfgs}, SCF cycle ${cycle})" > "$MARKER"
  log "EARLY-STOP: $stalled"
  if [[ -n "$SLURM_JOBID" ]]; then
    log "early-stop action: scancel ${SLURM_JOBID}"
    scancel "$SLURM_JOBID" 2>/dev/null || log "scancel failed (job may have left the queue)"
  elif [[ -n "$KILL_PATTERN" ]]; then
    log "early-stop action: pkill -TERM -f '${KILL_PATTERN}'"
    pkill -TERM -f "$KILL_PATTERN" 2>/dev/null || log "pkill failed (process may already be gone)"
  else
    log "report-only: no SLURM_JOBID or KILL_PATTERN configured"
  fi
fi

exit 0
