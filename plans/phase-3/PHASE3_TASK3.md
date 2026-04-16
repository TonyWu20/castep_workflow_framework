# TASK-3: Complete the monitoring dependency flip

> **Working directory**: all shell commands are run from the **workspace root** (`castep_workflow_framework/`).

## Current state

The codebase is **partially migrated**. Read this before touching anything.

**Already done — do not redo:**
- `workflow_core/src/monitoring.rs` — four data types only, no `execute_hook`, no `TaskExecutor`. ✅
- `workflow_core/src/lib.rs` — `pub mod monitoring;` + re-exports all four types. ✅
- `workflow_core/Cargo.toml` — no dependency on `workflow_utils`. ✅
- `workflow_utils/src/lib.rs` — has `mod monitoring;`, re-exports `execute_hook` and the four hook types. ✅
- `workflow_utils/Cargo.toml` — depends on `workflow_core = { path = "../workflow_core" }`. ✅

**Still broken — two files to fix:**
1. `workflow_utils/src/monitoring.rs` — duplicate type definitions (lines 6–57) must be removed.
2. `workflow_core/src/workflow.rs` — four dead calls to `crate::monitoring::execute_hook` (lines 49, 258, 295, 395) must be replaced with `tracing::debug!` stubs.

**Why stubs, not an interim hook executor field:** TASK-5 defines the `HookExecutor` trait and TASK-11 rewires `Workflow` around `Arc<dyn HookExecutor>`. Adding any interim closure field to `Workflow` now would be deleted in TASK-11. All four call sites already discard the result with `let _ =`; no test depends on hooks firing. Stubs are behaviourally equivalent for all current tests.

## Step 1 — Confirm broken state

```bash
cargo check -p workflow_core 2>&1 | head -30
```

Expected: exactly four errors, all in `workflow_core/src/workflow.rs` — `cannot find function execute_hook in module crate::monitoring`. No other errors. (`workflow_utils` is not checked here because its duplicate-type issue does not yet cause a compile error — it is a correctness problem, not a compilation one.)

## Step 2 — Back up

> Both backup files may already exist from a prior run. Overwriting them with the current state is intentional — they should reflect the state immediately before *this* execution.

```bash
cp workflow_utils/src/monitoring.rs workflow_utils/src/monitoring.rs.backup
cp workflow_core/src/workflow.rs workflow_core/src/workflow.rs.backup
```

## Step 3 — Replace `workflow_utils/src/monitoring.rs`

Write this exact content (entire file replacement — removes all duplicate type definitions, keeps only `execute_hook`):

```rust
use anyhow::Result;
use crate::executor::TaskExecutor;
use workflow_core::{HookContext, HookResult, MonitoringHook};

/// Executes a monitoring hook with the given context.
/// This is a free function because it needs access to TaskExecutor from workflow_utils.
pub fn execute_hook(hook: &MonitoringHook, ctx: &HookContext) -> Result<HookResult> {
    let mut parts = hook.command.split_whitespace();
    let cmd = parts.next().unwrap_or_default();
    let args: Vec<String> = parts.map(String::from).collect();
    let result = TaskExecutor::new(&ctx.workdir)
        .command(cmd)
        .args(args)
        .env("TASK_ID", &ctx.task_id)
        .env("TASK_STATE", &ctx.state)
        .env("WORKDIR", ctx.workdir.to_string_lossy().as_ref())
        .env("EXIT_CODE", ctx.exit_code.map(|c| c.to_string()).unwrap_or_default())
        .execute()?;
    Ok(HookResult { success: result.success(), output: result.stdout })
}
```

Verify: `cargo check -p workflow_utils` — zero errors.

## Step 4 — Stub out `workflow_core/src/workflow.rs`

Four edits, each replacing one dead call site. Plus one edit to disable `spawn_for_task`.

**Do NOT add any new fields to `Workflow`. Do NOT touch the builder or `resume`.**

---

**Edit 4a — Disable `spawn_for_task` (replace entire body)**

The function's sole purpose is to spawn periodic hook threads. Without an executor those threads are meaningless. Replace the **entire function body** — not just the opening lines — to avoid leaving dead code after a `return`, which would trigger an `unreachable_code` warning.

Find (the complete current function, from signature to closing brace — around lines 32–63):
```rust
    fn spawn_for_task(&mut self, task_id: String, hooks: &[MonitoringHook], ctx: HookContext) {
        let mut task_handles = Vec::new();

        for hook in hooks {
            if let HookTrigger::Periodic { interval_secs } = hook.trigger {
                let stop = Arc::new(AtomicBool::new(false));
                let stop_clone = stop.clone();
                let hook_clone = hook.clone();
                let ctx_clone = ctx.clone();

                let thread = std::thread::spawn(move || {
                    while !stop_clone.load(Ordering::Relaxed) {
                        std::thread::sleep(Duration::from_secs(interval_secs));
                        if stop_clone.load(Ordering::Relaxed) {
                            break;
                        }

                        let _ = crate::monitoring::execute_hook(&hook_clone, &ctx_clone);
                    }
                });

                task_handles.push(PeriodicHookHandle {
                    thread,
                    stop_signal: stop,
                });
            }
        }

        if !task_handles.is_empty() {
            self.handles.insert(task_id, task_handles);
        }
    }
```

Replace with:
```rust
    fn spawn_for_task(&mut self, task_id: String, hooks: &[MonitoringHook], ctx: HookContext) {
        // TODO(TASK-11): re-enable once Arc<dyn HookExecutor> is wired into Workflow.
        // Hook execution is deferred until the HookExecutor trait is defined (TASK-5)
        // and injected into the engine rewrite (TASK-11).
        let _ = (task_id, hooks, ctx);
    }
```

> The `PeriodicHookManager` struct, its `handles` field, and its `Drop` impl are left intact — TASK-11 restores the full body of this function with the injected executor.

---

**Edit 4b — Call site 1 (line ~49, inside the periodic thread closure)**

Find:
```rust
                        let _ = crate::monitoring::execute_hook(&hook_clone, &ctx_clone);
```

Replace with:
```rust
                        tracing::debug!(hook_name = %hook_clone.name, "Periodic hook triggered; execution deferred (TASK-5/TASK-11)");
```

---

**Edit 4c — Call site 2 (line ~258, OnComplete)**

Find:
```rust
                                let _ = crate::monitoring::execute_hook(hook, &ctx);
                            }
                        }
                    }
                    Err(e) => {
```

Replace with:
```rust
                                tracing::debug!(hook_name = %hook.name, task_id = %id, "OnComplete hook triggered; execution deferred (TASK-5/TASK-11)");
                            }
                        }
                    }
                    Err(e) => {
```

---

**Edit 4d — Call site 3 (line ~295, OnFailure)**

Find:
```rust
                                let _ = crate::monitoring::execute_hook(hook, &ctx);
                            }
                        }
                    }
                }
                s.save(&self.state_path)?;
```

Replace with:
```rust
                                tracing::debug!(hook_name = %hook.name, task_id = %id, "OnFailure hook triggered; execution deferred (TASK-5/TASK-11)");
                            }
                        }
                    }
                }
                s.save(&self.state_path)?;
```

---

**Edit 4e — Call site 4 (line ~395, OnStart)**

Find:
```rust
                                let _ = crate::monitoring::execute_hook(hook, &ctx);
                            }
                        }

                        let handle = std::thread::spawn(move || f());
```

Replace with:
```rust
                                tracing::debug!(hook_name = %hook.name, task_id = %id, "OnStart hook triggered; execution deferred (TASK-5/TASK-11)");
                            }
                        }

                        let handle = std::thread::spawn(move || f());
```

---

Verify: `cargo check -p workflow_core` — zero errors.

## Step 5 — Confirm no `workflow_utils` in `workflow_core`

```bash
grep -rn "workflow_utils" workflow_core/src/
```

Expected: no output. This checks the entire `workflow_core/src/` directory, not just the two files edited above.

## Step 6 — Final verification

```bash
cargo check --workspace && cargo test --workspace
```

Both must pass with zero errors and zero warnings.

## Acceptance criteria

| Check | Expected |
|-------|----------|
| `workflow_utils/src/monitoring.rs` | Only `execute_hook`; types imported from `workflow_core` |
| `workflow_core/src/workflow.rs` | No `crate::monitoring::execute_hook` calls; `spawn_for_task` returns early; no new fields on `Workflow` |
| `cargo check --workspace` | Zero errors, zero warnings |
| `cargo test --workspace` | All pass |

## What changes in later tasks

- **TASK-5**: Defines `pub trait HookExecutor` in `workflow_core/src/monitoring.rs` and implements `ShellHookExecutor` in `workflow_utils`. The `tracing::debug!` stubs are not replaced here — they remain until TASK-11.
- **TASK-11**: Rewrites `Workflow` to accept `Arc<dyn HookExecutor>`. The `PeriodicHookManager` early return is removed and the manager is wired to use `hook_executor.execute_hook(...)`. The stubs are replaced with real dispatch.
