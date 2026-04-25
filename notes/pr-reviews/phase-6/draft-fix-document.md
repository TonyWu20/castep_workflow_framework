## Draft Fix Document

### Issue 1: Dead code branch in `read_task_ids`

**Classification:** Correctness
**File:** `workflow-cli/src/main.rs`
**Severity:** Minor
**Problem:** The `task_ids.is_empty()` branch in `read_task_ids` is unreachable. The clap attribute `#[arg(required = false, default_value = "-")]` ensures `task_ids` always contains at least one element (`"-"` when not supplied). The empty-branch on line 32 can never execute.
**Fix:** Remove the dead `task_ids.is_empty()` branch. If the intent was to detect the sentinel `"-"` value, match on `task_ids == ["-"]` instead. If the sentinel handling is no longer needed, simplify to always use `"-"` as the task ID argument.
