# TaskExecutor vs Task closure

`TaskExecutor` and `Task::execute_fn` operate at different levels and are not interchangeable.

## `Task::execute_fn`

A Rust closure: `Arc<dyn Fn() -> anyhow::Result<()> + Send + Sync>`.

- Runs arbitrary Rust code in a `std::thread::spawn` call inside `Workflow::run()`
- Represents the workflow task itself

## `TaskExecutor`

A `std::process::Command` builder with fields: `workdir`, `command`, `args`, `env`.

- Spawns an external OS process via `.execute()` (blocking) or `.spawn()` (async handle)
- Has no closure support — cannot accept a `Fn()`

## How they compose

A task closure may *use* `TaskExecutor` internally to launch an external binary:

```
Workflow::run()
  └─ std::thread::spawn(|| task.execute_fn())   // runs the Rust closure
       └─ TaskExecutor::new(workdir)
              .command("castep")
              .execute()                         // spawns the OS process
```

## MonitoringHook

Also uses `TaskExecutor` internally to run shell hook commands. Fired by the scheduler at task lifecycle events (OnStart, OnComplete, OnFailure) — not by the task closure.
