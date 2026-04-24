# Phase 6: Reliability, Multi-Parameter Patterns, and Ergonomics

**Date:** 2026-04-25
**Status:** Draft

## Context

Phases 1–5B built a feature-complete workflow framework for single-parameter CASTEP sweeps on SLURM. Phase 5A was the first production run on a real HPC cluster, which surfaced a correctness bug (squeue false-positive marking failed jobs as Completed) and ergonomic gaps (must invoke binary from workdir). Phase 5B cleaned up API ergonomics but deferred reliability fixes and multi-parameter sweep support.

Phase 6 addresses:
- A **correctness bug** where collect-closure failures are silently ignored (task stays `Completed`)
- The gap between single-parameter and **multi-parameter sweeps** (product and pairwise)
- The **workdir constraint** that limits HPC usability
- **Retry ergonomics** for multi-parameter workflows via Unix pipeline composition
- **Documentation accuracy** issues accumulated over 3 phases

## Goals

### 1. CollectFailurePolicy: Collect Closure as Success Gate

**What:** Fix the correctness bug in `process_finished()` (`workflow_core/src/workflow.rs:373-383`) where `mark_completed(id)` runs *before* the collect closure, and collect failures only emit `tracing::warn!` — leaving the task marked `Completed` even when output validation fails.

**Why now:** This is a correctness bug observed in production (D.7: squeue returned empty output → assumed exit 0 → task marked Completed → collect saw missing output but warning was ignored). A `Completed` status must mean the calculation genuinely finished and passed validation.

**Design:**
- Reorder `process_finished()`: run collect *after* exit-code check but *before* `mark_completed()`
- If collect fails and policy is `FailTask` (default): `mark_failed()` with collect error message
- If collect fails and policy is `WarnOnly`: `mark_completed()` + `tracing::warn!` (backward compat)
- Add `CollectFailurePolicy` enum to `workflow_core::task`:
  ```rust
  #[derive(Debug, Clone, Default)]
  pub enum CollectFailurePolicy {
      #[default]
      FailTask,
      WarnOnly,
  }
  ```
- Add `collect_failure_policy: CollectFailurePolicy` field to `Task` (defaults to `FailTask`)
- Add builder method: `Task::collect_failure_policy(self, policy) -> Self`
- **Generic by design:** the framework defines the *policy* (what to do on collect failure); Layer 3 defines the *check* (what "success" means for CASTEP/VASP/QE). The framework never knows about "Total time" or any software-specific output.

**Critical files:**
- `workflow_core/src/task.rs` — add `CollectFailurePolicy` enum + field + builder
- `workflow_core/src/workflow.rs:360-416` — reorder `process_finished()` logic
- `workflow_core/src/prelude.rs` — re-export `CollectFailurePolicy`
- `workflow_core/tests/` — test both policies (collect-fail-marks-failed, collect-fail-warns-only)

### 2. Multi-Parameter Sweep: Build, Test on Cluster, Document

**What:** Build a real multi-parameter sweep (product and pairwise modes), run it on the HPC cluster, and document what we learn — including any framework gaps that surface.

**Why now:** Phase 5 only tested single-parameter sweeps. Multi-parameter sweeps are the real research use case (U × k-points, U × cutoff energy). The framework API *should* support this already, but we've never validated it on real hardware. Documentation without cluster validation risks shipping patterns that break in production.

**Design — Layer 3, not framework API:**
- **No new framework types.** The existing `Task::new` + `depends_on` + `add_task` API is believed sufficient. Cluster testing will confirm or reveal gaps.
- **`itertools::iproduct!`** for product sweeps (Cartesian: m×n tasks)
- **`.iter().zip()`** for pairwise sweeps (matched pairs: min(m,n) tasks)
- Both are one-line iterator changes — the difference is user intent, not framework capability
- **Dependent chains:** a `build_chain(params) -> Vec<Task>` function that wires `depends_on` internally (e.g., SCF → DOS per parameter combination)
- **Future note:** When Tier 2 interactive CLI arrives, sweep mode selection ("product or pairwise?") becomes a framework-level prompt. Until then, Layer 3 decides.

**Cluster validation targets:**
- Does `WorkflowSummary` give enough info to understand which *parameter combinations* failed (not just task IDs)?
- Does the collect closure for dependent stages (e.g., DOS) need access to upstream results? (If yes → typed result collection moves to Phase 7 priority)
- Are there DAG scaling issues with large parameter grids (e.g., 6×4 = 24 tasks × 2 stages = 48 nodes)?
- Is retry ergonomics sufficient with Unix pipes (see Goal 4)?

**Deliverables:**
- Add `itertools` to workspace `[dependencies]`
- Extend `examples/hubbard_u_sweep_slurm` with multi-parameter sweep support (product + pairwise modes, dependent task chains)
- Run on HPC cluster; record findings (gaps found → feed into Phase 7 scope)
- Add "Multi-Parameter Sweep Patterns" section to ARCHITECTURE.md with validated code examples
- Document both sweep modes with clear guidance on when to use which

### 3. `--workdir` / Root Directory Support

**What:** Allow the workflow binary to be invoked from any directory, not just the directory where `runs/`, `logs/`, and the state file should be created.

**Why now:** This was explicitly called the "most user-visible ergonomic gap" from Phase 5A production runs (D.6). HPC submission scripts frequently run binaries from a different directory.

**Design:**
- Add `root_dir: Option<PathBuf>` field to `Workflow` in `workflow_core`
- Add builder: `Workflow::with_root_dir(self, dir: impl Into<PathBuf>) -> Self`
- In `run()`, if `root_dir` is `Some(dir)`, resolve all relative task workdirs against `dir` (i.e., `dir.join(task.workdir)`)
- Log dir resolution also uses `root_dir` if set
- `QueuedRunner::submit()` log path absolutization uses `root_dir` (subsumes D.4)
- Layer 3 examples add `--workdir` via clap: `#[arg(long, default_value = ".")]`

**Critical files:**
- `workflow_core/src/workflow.rs` — add `root_dir` field + builder + resolution in `run()`
- `workflow_utils/src/runner.rs` — `QueuedRunner::submit()` uses root_dir for log paths
- `examples/hubbard_u_sweep_slurm/src/main.rs` — add `--workdir` clap flag

### 4. `workflow-cli retry` Stdin Support

**What:** Make `retry` accept task IDs from stdin, enabling Unix pipeline composition for parameter-subset retry.

**Why now:** Multi-parameter sweeps (Goal 2) create many tasks with structured IDs (e.g., `scf_U3.0_kpt8x8x8`). When a parameter subset fails, researchers need to retry by pattern. Rather than implementing glob/regex matching inside the CLI (which would require dry-run mode, multi-pattern handling, and reimplements `grep`), we leverage the Unix pipeline — the most universal and composable approach.

**Design:**
- Detect stdin is a pipe (not a TTY): if `task_ids` is empty and stdin is piped, read task IDs from stdin (one per line, skip blanks)
- Convention: `workflow-cli retry state.json -` reads from stdin explicitly (like `cat -`)
- This composes with any Unix tool for Tier 1 users:
  ```bash
  # Retry all failed U3.0 tasks
  workflow-cli status .workflow.json | grep 'U3.0.*Failed' | cut -d: -f1 \
    | workflow-cli retry .workflow.json -

  # Retry from a file
  workflow-cli retry .workflow.json - < retry-list.txt
  ```
- Approach B (`--match` glob) deferred: it requires dry-run confirmation mode, gets clumsy with multiple patterns, and reimplements grep. May be revisited for Tier 2 UX.
- Approach C (`--from-file`) is subsumed by stdin — `< file` achieves the same result.

**Critical files:**
- `workflow-cli/src/main.rs` — modify `Retry` command to accept stdin input

### 5. Documentation Accuracy Sweep

**What:** Fix all 6 known doc-vs-code mismatches from Phase 5B deferrals.

**Why now:** These accumulate and create misleading expectations for anyone reading the docs. Land last so docs reflect all Phase 6 API changes.

**Items:**
1. ARCHITECTURE.md: `setup`/`collect` builder signature — doc shows `<F>` returning `Result<(), WorkflowError>`, actual is `<F, E>` with `E: std::error::Error + Send + Sync + 'static`
2. ARCHITECTURE.md: `JsonStateStore::new` — doc shows `impl Into<String>`, actual takes `&str` (or update the impl to accept `impl Into<String>`)
3. ARCHITECTURE.md: `load`/`load_raw` — shown as instance methods, actually static constructors returning `Result<Self, WorkflowError>`
4. ARCHITECTURE_STATUS.md: Phase 3/4 entries — stale `TaskClosure` and `downstream_of` descriptions that contradict Phase 5B changes
5. `parse_empty_string` test — strengthen assertion from `!err.is_empty()` to `err.contains("invalid")` or similar
6. Trailing newline in `workflow_utils/src/prelude.rs`

**Critical files:**
- `ARCHITECTURE.md`
- `ARCHITECTURE_STATUS.md`
- `examples/hubbard_u_sweep_slurm/src/config.rs` (test fix)
- `workflow_utils/src/prelude.rs` (trailing newline)

## Scope Boundaries

**In scope:**
- `CollectFailurePolicy` enum + reordered `process_finished()` logic
- Multi-parameter sweep: build, test on HPC cluster, document findings
- Extended example with product + pairwise modes and dependent task chains
- `--workdir` / `root_dir` support in `Workflow`
- `workflow-cli retry` stdin support for Unix pipeline composition
- All 6 deferred doc/test fixes from Phase 5B

**Out of scope:**
- Typed result collection (Phase 7 — large API surface, needs own design iteration)
- Portable SLURM job script template (D.1 — no second user/cluster yet)
- `TaskChain` abstraction (premature — wait for 3+ real multi-stage workflows)
- Framework-level sweep builder/combinator (premature — `iproduct!` + `zip` sufficient)
- `--match` glob pattern for retry (reimplements grep; Unix pipes are more universal; revisit for Tier 2 UX)
- Tier 2 interactive CLI (future phase)
- `std::path::absolute` standalone (subsumed by `root_dir` resolution)

## Design Notes

**CollectFailurePolicy must remain software-agnostic.** The framework defines the *mechanism* (run collect, check result, apply policy). Layer 3 defines the *criteria* (what "success" means). This ensures the framework works for CASTEP, VASP, QE, or any future code without modification.

**Multi-parameter sweeps need cluster validation, not just documentation.** The framework sees `Vec<Task>` — it doesn't know or care how tasks were generated. `itertools::iproduct!` and `zip` are the right generation tools. But whether `WorkflowSummary`, `retry`, and `collect` closures work well for multi-param DAGs is unproven. Running on real hardware will surface gaps that analysis alone cannot. Any gaps found feed directly into Phase 7 scope.

**`root_dir` resolution strategy:** Only resolve relative paths. If a task's workdir is already absolute, leave it alone. This preserves existing behavior for code that doesn't set `root_dir`.

**Retry via Unix pipes, not built-in pattern matching.** The CLI's job is to accept task IDs and reset them. Pattern matching (grep), field extraction (cut/awk), and composition (pipes) are the shell's job. This follows the Unix philosophy and avoids reimplementing grep poorly. When Tier 2 UX arrives and users can't be expected to know Unix pipes, `--match` glob support may be added with mandatory dry-run confirmation.

## Deferred Items Absorbed

| Item | Source | Absorbed into |
|---|---|---|
| D.7: squeue false-positive | Phase 5A | Goal 1 (CollectFailurePolicy) |
| CollectFailurePolicy | Phase 5B out-of-scope | Goal 1 |
| D.6: `--workdir` flag | Phase 5A | Goal 3 |
| D.4: `std::path::absolute` log paths | Phase 5A | Goal 3 (subsumed by root_dir) |
| ARCHITECTURE.md signature mismatches (3 items) | Phase 5B | Goal 5 |
| ARCHITECTURE_STATUS.md stale entries | Phase 5B | Goal 5 |
| `parse_empty_string` weak assertion | Phase 5B | Goal 5 |
| Trailing newline `prelude.rs` | Phase 5B | Goal 5 |

## Sequencing

```
Goal 1: CollectFailurePolicy          (workflow_core — reliability fix, touches workflow.rs)
Goal 3: --workdir / root_dir          (workflow_core — also touches workflow.rs, builds on Goal 1)
Goal 4: retry stdin support           (workflow-cli — small, independent)
Goal 2: Multi-param patterns          (docs + example — benefits from stable API after 1/3)
Goal 5: Documentation sweep           (lands last — reflects all API changes from 1-4)
```

## Open Questions

None — scope is agreed with user. Cluster testing in Goal 2 may surface new questions that feed into Phase 7.

## Verification

After each goal:
```
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```

Goal 1: Integration test — task with collect closure that fails should be marked `Failed` (not `Completed`)
Goal 2: Extended example compiles, `--dry-run` shows correct task ordering, and **real HPC run** completes with correct status reporting for multi-param sweep
Goal 3: Binary invoked from different directory correctly creates `runs/` under `--workdir` path
Goal 4: Pipe `echo "task_id" | workflow-cli retry state.json -` works; verify with `status` afterward
Goal 5: `cargo doc --workspace` builds clean; all ARCHITECTURE.md code blocks match `grep` of actual signatures
