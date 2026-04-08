# Fix Subtasks: qwopus3.5 branch

Reviewed and verified. Apply in dependency order.

---

## T1 — Add `workdir` field to `Task` struct

**File:** `workflow_core/src/task.rs`
**Depends on:** none

Add `use std::path::PathBuf;` if not present. Add `pub workdir: PathBuf` to the struct:

```rust
// Before:
pub struct Task {
    pub id: String,
    pub dependencies: Vec<String>,
    pub execute_fn: Arc<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
    pub monitors: Vec<MonitoringHook>,
}

// After:
pub struct Task {
    pub id: String,
    pub dependencies: Vec<String>,
    pub execute_fn: Arc<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
    pub monitors: Vec<MonitoringHook>,
    pub workdir: PathBuf,
}
```

---

## T2 — Initialise `workdir` in `Task::new()`

**File:** `workflow_core/src/task.rs`
**Depends on:** T1

```rust
// Before:
Self {
    id: id.into(),
    dependencies: Vec::new(),
    execute_fn: Arc::new(f),
    monitors: Vec::new(),
}

// After:
Self {
    id: id.into(),
    dependencies: Vec::new(),
    execute_fn: Arc::new(f),
    monitors: Vec::new(),
    workdir: PathBuf::from("."),
}
```

---

## T3 — Extract monitors and workdirs into HashMaps in `run()`

**File:** `workflow_core/src/workflow.rs`
**Depends on:** T1

Add import (correct crate-root re-exports):
```rust
use workflow_utils::{HookContext, HookTrigger, MonitoringHook};
```

After the `fns` map, add:
```rust
let monitors: HashMap<String, Vec<MonitoringHook>> = self
    .tasks
    .iter()
    .map(|(id, t)| (id.clone(), t.monitors.clone()))
    .collect();

let task_workdirs: HashMap<String, PathBuf> = self
    .tasks
    .iter()
    .map(|(id, t)| (id.clone(), t.workdir.clone()))
    .collect();
```

---

## T4 — Fire `OnStart` hooks after `mark_running`

**File:** `workflow_core/src/workflow.rs`
**Depends on:** T3

```rust
// Before:
if let Some(f) = fns.get(&id).cloned() {
    state.lock().unwrap().mark_running(&id);
    let handle = std::thread::spawn(move || f());
    handles.insert(id, handle);
}

// After:
if let Some(f) = fns.get(&id).cloned() {
    // task threads don't hold the lock when they panic — poisoning is not expected
    state.lock().unwrap().mark_running(&id);
    if let Some(hooks) = monitors.get(&id) {
        let ctx = HookContext {
            task_id: id.clone(),
            workdir: task_workdirs[&id].clone(),
            state: "running".to_string(),
            exit_code: None,
        };
        for hook in hooks.iter().filter(|h| matches!(h.trigger, HookTrigger::OnStart)) {
            let _ = hook.execute(&ctx);
        }
    }
    let handle = std::thread::spawn(move || f());
    handles.insert(id, handle);
}
```

---

## T5 — Fire `OnComplete` hooks after `mark_completed`

**File:** `workflow_core/src/workflow.rs`
**Depends on:** T3

```rust
// Before:
Ok(()) => s.mark_completed(&id),

// After:
Ok(()) => {
    s.mark_completed(&id);
    if let Some(hooks) = monitors.get(&id) {
        let ctx = HookContext {
            task_id: id.clone(),
            workdir: task_workdirs[&id].clone(),
            state: "completed".to_string(),
            exit_code: Some(0),
        };
        for hook in hooks.iter().filter(|h| matches!(h.trigger, HookTrigger::OnComplete)) {
            let _ = hook.execute(&ctx);
        }
    }
}
```

---

## T6 — Fire `OnFailure` hooks after `mark_failed`

**File:** `workflow_core/src/workflow.rs`
**Depends on:** T3

```rust
// Before:
Err(e) => s.mark_failed(&id, e.to_string()),

// After:
Err(e) => {
    s.mark_failed(&id, e.to_string());
    if let Some(hooks) = monitors.get(&id) {
        let ctx = HookContext {
            task_id: id.clone(),
            workdir: task_workdirs[&id].clone(),
            state: "failed".to_string(),
            exit_code: None,
        };
        for hook in hooks.iter().filter(|h| matches!(h.trigger, HookTrigger::OnFailure)) {
            let _ = hook.execute(&ctx);
        }
    }
}
```

---

## T7 — Delete dead `num_cpus` function

**File:** `workflow_core/src/workflow.rs`
**Depends on:** none

Remove entirely:
```rust
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}
```

---

## T8 — Replace bare `unwrap()` on `handles.remove`

**File:** `workflow_core/src/workflow.rs`
**Depends on:** none

```rust
// Before:
handles.remove(&id).unwrap()

// After:
handles.remove(&id).expect("id was just confirmed present in finished list")
```

---

## T9 — Annotate all `state.lock().unwrap()` sites

**File:** `workflow_core/src/workflow.rs`
**Depends on:** none

For every occurrence of `state.lock().unwrap()` in `run()`, add the comment on the line above:
```rust
// task threads don't hold the lock when they panic — poisoning is not expected
state.lock().unwrap()
```

There are four call sites. Annotate each one individually.

---

## Dependency order

```
T1 → T2
T1 → T3 → T4
          T3 → T5
          T3 → T6
T7 (independent)
T8 (independent)
T9 (independent)
```
