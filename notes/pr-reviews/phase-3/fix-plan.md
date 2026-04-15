# Phase 3 Fix Plan — v6 (2026-04-16)

This plan addresses 6 issues found in the v6 review. TASK-1 replaces `now_iso8601`'s hand-rolled calendar arithmetic with the `time` crate (eliminates the correctness risk without needing edge-case tests). TASK-2 through TASK-6 are minor API polish items. TASK-3 (FailedTask struct) has the largest blast radius; TASK-4 and TASK-5 depend on it.

---

## Execution Phases

| Phase              | Tasks                          | Notes                                                       |
| ------------------ | ------------------------------ | ----------------------------------------------------------- |
| Phase 1 (parallel) | TASK-1, TASK-2, TASK-3, TASK-6 | All independent — different files or non-overlapping sections |
| Phase 2 (parallel) | TASK-4, TASK-5                 | Depend on TASK-3; TASK-4 and TASK-5 touch different files   |
| Final              | —                              | `cargo test --workspace && cargo clippy --workspace`        |

**File conflict note:** TASK-5 and TASK-6 both edit `workflow_core/src/lib.rs` but touch different lines. Assign them to different phases (TASK-6 in Phase 1, TASK-5 in Phase 2) to avoid races.

---

### TASK-1: Replace hand-rolled `now_iso8601` with `time` crate

**File:** `workflow_core/src/state.rs`, `workflow_core/Cargo.toml`
**Severity:** Major
**Problem:** `now_iso8601()` contains hand-rolled civil calendar arithmetic (Neri-Schneider method) with zero direct unit tests. Correctness risk on edge dates (century leap years, month boundaries).
**Fix:** Add `time = { version = "0.3", features = ["formatting"] }` to `workflow_core` dependencies and replace the function body with a single `time` crate call.

**Step 1 — Add dependency.**

File: `workflow_core/Cargo.toml`

Before (inside `[dependencies]`):
```toml
thiserror = "1"
```

After:
```toml
thiserror = "1"
time = { version = "0.3", features = ["formatting"] }
```

**Step 2 — Replace `now_iso8601` function body.**

File: `workflow_core/src/state.rs`

Before (the entire `now_iso8601` function):
```rust
fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}
```

After:
```rust
fn now_iso8601() -> String {
    use time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
```

**Verification:** `cargo test -p workflow_core && cargo clippy -p workflow_core`

---

### TASK-2: Add doc comment to `ExecutionMode::Queued` variant

**File:** `workflow_core/src/task.rs`
**Severity:** Minor
**Problem:** `Queued` is a public variant that `Workflow::run()` always rejects at runtime. No warning for consumers.

Before (inside `pub enum ExecutionMode`):
```rust
    Queued {
        submit_cmd: String,
        poll_cmd: String,
        cancel_cmd: String,
    },
```

After:
```rust
    /// Not yet implemented. Constructing a task with this mode will cause
    /// `Workflow::run()` to return `Err(WorkflowError::InvalidConfig)`.
    /// Reserved for future HPC queue integration (SLURM/PBS).
    Queued {
        submit_cmd: String,
        poll_cmd: String,
        cancel_cmd: String,
    },
```

**Verification:** `cargo clippy -p workflow_core`

---

### TASK-3: Replace anonymous tuple in `WorkflowSummary::failed` with `FailedTask` struct

**File:** `workflow_core/src/workflow.rs`
**Severity:** Minor
**Problem:** `pub failed: Vec<(String, String)>` — field order (task_id, error_message) is documented only by comment. A named struct is self-documenting.

**Edit 1 — Insert `FailedTask` struct** immediately before the `WorkflowSummary` struct. Find this existing line and insert the new struct before it:

Before (the anchor line to search for):
```rust
/// Summary of workflow execution results.
```

After (insert the new struct before that anchor, keeping the anchor line unchanged):
```rust
/// A task that failed during workflow execution.
#[derive(Debug, Clone)]
pub struct FailedTask {
    pub id: String,
    pub error: String,
}

/// Summary of workflow execution results.
```

**Edit 2 — Update `WorkflowSummary::failed` field type:**

Before:
```rust
    pub failed: Vec<(String, String)>, // (task_id, error_message)
```

After:
```rust
    pub failed: Vec<FailedTask>,
```

**Edit 3 — Update construction in `run()`:**

Before:
```rust
                TaskStatus::Failed { error } => failed.push((id, error)),
```

After:
```rust
                TaskStatus::Failed { error } => failed.push(FailedTask { id, error }),
```

**Verification:** `cargo check -p workflow_core 2>&1 | head -30` (expect errors in callers — fixed in TASK-4 and TASK-5)

---

### TASK-4: Update `WorkflowSummary::failed` callers in `workflow_core` tests and integration tests

**File:** `workflow_core/tests/integration.rs`, `workflow_core/tests/timeout_integration.rs`
**Severity:** Minor (depends on TASK-3)
**Problem:** Two test files destructure `WorkflowSummary::failed` as tuples `(id, _)` which will not compile after TASK-3.

**Edit 1 — `workflow_core/tests/integration.rs`:**

Before:
```rust
    assert!(summary1.failed.iter().any(|(id, _)| id == "b"));
```

After:
```rust
    assert!(summary1.failed.iter().any(|f| f.id == "b"));
```

**Edit 2 — `workflow_core/tests/timeout_integration.rs`:**

Before:
```rust
    let (_, err) = summary.failed.iter().find(|(id, _)| id == "sleeper").expect("sleeper should fail");
    assert!(err.contains("timed out"), "error was: {}", err);
```

After:
```rust
    let f = summary.failed.iter().find(|f| f.id == "sleeper").expect("sleeper should fail");
    assert!(f.error.contains("timed out"), "error was: {}", f.error);
```

**Verification:** `cargo test -p workflow_core && cargo clippy -p workflow_core`

---

### TASK-5: Re-export `FailedTask` from `workflow_core/src/lib.rs`

**File:** `workflow_core/src/lib.rs`
**Severity:** Minor (depends on TASK-3)
**Problem:** `FailedTask` must be re-exported so downstream crates can name the type.

Before:
```rust
pub use workflow::{Workflow, WorkflowSummary};
```

After:
```rust
pub use workflow::{FailedTask, Workflow, WorkflowSummary};
```

**Verification:** `cargo check -p workflow_core`

---

### TASK-6: Add comments to `init_default_logging` and `executor.rs` re-exports

**File:** `workflow_core/src/lib.rs`, `workflow_utils/src/executor.rs`
**Severity:** Minor
**Problem:** `init_default_logging` returns `Box<dyn Error>` with no explanation; the `pub use workflow_core::…` re-exports in `executor.rs` have no comment.

**Edit 1 — `workflow_core/src/lib.rs`:** Add comment above the `init_default_logging` function (before its `/// Initialize default…` doc comment):

Before:
```rust
/// Initialize default tracing subscriber with env-based filtering.
/// Call once at start of main(). Controlled via RUST_LOG env var.
/// Returns error if already initialized (safe, won't panic).
#[cfg(feature = "default-logging")]
pub fn init_default_logging() -> Result<(), Box<dyn std::error::Error>> {
```

After:
```rust
// Returns Box<dyn Error> rather than WorkflowError because tracing_subscriber's
// SetGlobalDefaultError is not convertible to any WorkflowError variant without
// introducing a logging-specific variant that doesn't belong in the domain error type.
/// Initialize default tracing subscriber with env-based filtering.
/// Call once at start of main(). Controlled via RUST_LOG env var.
/// Returns error if already initialized (safe, won't panic).
#[cfg(feature = "default-logging")]
pub fn init_default_logging() -> Result<(), Box<dyn std::error::Error>> {
```

**Edit 2 — `workflow_utils/src/executor.rs`:** Add comment above the two `pub use` lines:

Before:
```rust
pub use workflow_core::WorkflowError;
pub use workflow_core::{ProcessRunner, ProcessHandle, ProcessResult};
```

After:
```rust
// Re-exported so consumers that only depend on `workflow_utils` can access
// the core process/error types without a direct `workflow_core` dependency.
pub use workflow_core::WorkflowError;
pub use workflow_core::{ProcessHandle, ProcessResult, ProcessRunner};
```

**Verification:** `cargo clippy --workspace`
