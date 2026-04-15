## v5 (2026-04-16)

# Phase 3 Fix Plan — Post-v4-Fixes Review (v5)

This plan addresses 3 minor style/organization issues found after the v4 fix round. All items confirmed present in the actual source on the `phase-3` branch. All three are in different files and fully independent.

---

## Execution Phases

| Phase              | Tasks                      | Notes                                     |
| ------------------ | -------------------------- | ----------------------------------------- |
| Phase 1 (parallel) | TASK-1, TASK-2, TASK-3     | All in different files — fully independent |
| Final              | —                          | `cargo check --workspace`                 |

---

### TASK-1: Consolidate split `use` block in `workflow.rs`

**File:** `workflow_core/src/workflow.rs`
**Depends on:** None
**Can run in parallel with:** TASK-2, TASK-3

**Why:** The `use std::collections`, `use std::sync`, and `use std::time::Duration` imports are placed after the `InFlightTask` struct definition instead of with the other imports at the top of the file. This splits a logical import block around a struct definition.

**Before** (lines 8–22, the gap between `use crate::HookExecutor;` and the `pub struct Workflow`):

```rust
use crate::HookExecutor;

/// A handle to a running task with metadata.
pub(crate) struct InFlightTask {
    pub handle: Box<dyn ProcessHandle>,
    pub started_at: Instant,
    pub monitors: Vec<crate::monitoring::MonitoringHook>,
    pub collect: Option<TaskClosure>,
    pub workdir: std::path::PathBuf,
}

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
```

**After:**

```rust
use crate::HookExecutor;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// A handle to a running task with metadata.
pub(crate) struct InFlightTask {
    pub handle: Box<dyn ProcessHandle>,
    pub started_at: Instant,
    pub monitors: Vec<crate::monitoring::MonitoringHook>,
    pub collect: Option<TaskClosure>,
    pub workdir: std::path::PathBuf,
}
```

**Verification:** `cargo check --workspace`

---

### TASK-2: Move mid-file `pub use` re-export to top of `executor.rs`

**File:** `workflow_utils/src/executor.rs`
**Depends on:** None
**Can run in parallel with:** TASK-1, TASK-3

**Why:** The `pub use workflow_core::{ProcessRunner, ProcessHandle, ProcessResult};` re-export is buried at line 104 between two impl blocks instead of at the top of the file with the other `pub use`. This requires two edits: add it at the top, then remove the mid-file copy.

**Edit 1 — add re-export after existing `pub use` at top of file:**

Before:

```rust
pub use workflow_core::WorkflowError;

pub struct TaskExecutor {
```

After:

```rust
pub use workflow_core::WorkflowError;
pub use workflow_core::{ProcessRunner, ProcessHandle, ProcessResult};

pub struct TaskExecutor {
```

**Edit 2 — remove mid-file re-export (anchor: lines 101–105):**

Before:

```rust
}


pub use workflow_core::{ProcessRunner, ProcessHandle, ProcessResult};

/// Concrete implementation of the ProcessRunner trait for system processes.
```

After:

```rust
}

/// Concrete implementation of the ProcessRunner trait for system processes.
```

**Verification:** `cargo check --workspace`

---

### TASK-3: Re-export `TaskClosure` from `workflow_core` crate root

**File:** `workflow_core/src/lib.rs`
**Depends on:** None
**Can run in parallel with:** TASK-1, TASK-2

**Why:** `TaskClosure` is defined in the `task` module and used by downstream crates, but it is not included in the `pub use task::{...}` re-export line in `lib.rs`. Downstream code must use the full path `workflow_core::task::TaskClosure` instead of `workflow_core::TaskClosure`.

**Before:**

```rust
pub use task::{ExecutionMode, Task};
```

**After:**

```rust
pub use task::{ExecutionMode, Task, TaskClosure};
```

**Verification:** `cargo check --workspace`

---

## Dependency Graph

```mermaid
graph TD
  TASK-1
  TASK-2
  TASK-3
```

All three tasks are fully independent with no edges between them.

**Final verification after all tasks:**

```bash
cargo check --workspace
```
