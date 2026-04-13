# Builder Pattern Integration Plan for Workflow Framework

## Context

Tony wants to introduce `bon-3.9.1` to add builder pattern support to the workflow framework's user-facing API. The goal is consistent chain-calling style for Layer 3 project crates that define workflows.

**Scope:**
- **Target:** `Task` and `Workflow` in `workflow_core` (Layer 1 public API)
- **Out of scope:** `workflow_utils` internals (TaskExecutor, file I/O, etc.) - Layer 3 users don't touch these
- **Approach:** Hybrid - keep simple constructors, add builders for configuration

## Current API Analysis

### Task (workflow_core/src/task.rs)

**Current implementation:**
```rust
pub struct Task {
    pub id: String,
    pub dependencies: Vec<String>,
    pub execute_fn: Arc<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
    pub monitors: Vec<MonitoringHook>,
}

impl Task {
    pub fn new<F>(id: impl Into<String>, f: F) -> Self
    where F: Fn() -> anyhow::Result<()> + Send + Sync + 'static
    {
        Self {
            id: id.into(),
            dependencies: Vec::new(),
            execute_fn: Arc::new(f),
            monitors: Vec::new(),
        }
    }

    pub fn depends_on(mut self, id: impl Into<String>) -> Self {
        self.dependencies.push(id.into());
        self
    }

    pub fn add_monitor(mut self, hook: MonitoringHook) -> Self {
        self.monitors.push(hook);
        self
    }
}
```

**Current usage:**
```rust
Task::new("task_id", || Ok(()))
    .depends_on("dep1")
    .depends_on("dep2")
    .add_monitor(hook)
```

**Assessment:** Already excellent! Chain-calling works well. No major changes needed.

### Workflow (workflow_core/src/workflow.rs)

**Current implementation:**
```rust
pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    state_path: PathBuf,
    max_parallel: usize,  // ❌ Not configurable!
}

impl Workflow {
    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::with_state_dir(name, ".")
    }

    pub fn with_state_dir(name: impl Into<String>, dir: impl Into<PathBuf>) -> Result<Self> {
        let name = name.into();
        let state_path = dir.into().join(format!(".{}.workflow.json", name));
        Ok(Self { 
            name, 
            tasks: HashMap::new(), 
            state_path, 
            max_parallel: num_cpus()  // ❌ Hardcoded!
        })
    }

    pub fn add_task(&mut self, task: Task) -> Result<()> { ... }
    pub fn run(&mut self) -> Result<()> { ... }
}
```

**Current usage:**
```rust
let mut workflow = Workflow::new("name")?;
workflow.add_task(task1)?;
workflow.add_task(task2)?;
workflow.run()?;
```

**Problems:**
1. `max_parallel` is hardcoded to `num_cpus()` - not configurable
2. Two constructors (`new`, `with_state_dir`) - inconsistent
3. Mutable `add_task()` breaks chain-calling

## Proposed Design: Simplified Hybrid Approach

### Task: Keep Current API (No Changes)

**Rationale:**
- Current API is already ergonomic
- Closure handling is simple and clean
- Chain-calling works perfectly
- `bon` would add complexity without benefit

**Keep as-is:**
```rust
Task::new("id", || Ok(()))
    .depends_on("a")
    .depends_on("b")
    .add_monitor(hook)
```

### Workflow: Add Builder for Configuration Only

**Goal:** Expose `max_parallel` configuration without breaking existing code.

**Simplified approach - builder replaces both constructors:**

```rust
#[bon]
impl Workflow {
    #[builder]
    pub fn new(
        name: String,
        #[builder(default = PathBuf::from("."))]
        state_dir: PathBuf,
        #[builder(default)]
        max_parallel: Option<usize>,
    ) -> Result<Self> {
        let max_parallel = max_parallel.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        });
        
        if max_parallel == 0 {
            bail!("max_parallel must be at least 1");
        }
        
        let state_path = state_dir.join(format!(".{}.workflow.json", name));
        Ok(Self {
            name,
            tasks: HashMap::new(),
            state_path,
            max_parallel,
        })
    }
}
```

**Usage patterns:**

**Simple (backward compatible):**
```rust
let mut workflow = Workflow::builder().name("workflow").build()?;
workflow.add_task(task1)?;
workflow.add_task(task2)?;
workflow.run()?;
```

**With configuration:**
```rust
let mut workflow = Workflow::builder()
    .name("workflow")
    .state_dir("./runs")
    .max_parallel(8)
    .build()?;
workflow.add_task(task)?;
```

**Note:** `add_task()` keeps its current signature `&mut self -> Result<()>`. Mutable reference chaining is non-idiomatic in Rust. Users construct workflow, then mutate it.

## Implementation Plan

### Step 1: Add `bon` Dependency

**File:** `workflow_core/Cargo.toml`

Add:
```toml
[dependencies]
bon = "3.9.1"
```

Note: Using `std::thread::available_parallelism()` for CPU count (no external dependency needed).

### Step 2: Modify Workflow API

**File:** `workflow_core/src/workflow.rs`

Changes:
1. Add `use bon::bon;` import
2. Replace `new()` and `with_state_dir()` with single `#[builder]` method
3. Use `Option<usize>` for `max_parallel` with runtime default via `unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4))`
4. Add validation: `max_parallel > 0` check (empty name allowed - creates `.workflow.json`)
5. Keep `add_task(&mut self) -> Result<()>` signature unchanged
6. Preserve state path calculation: `state_dir.join(format!(".{}.workflow.json", name))`

### Step 3: Update Tests

**File:** `workflow_core/src/workflow.rs` (tests module)

Update tests to use new builder API:
- Update existing tests to use `Workflow::builder().name("test").build()?`
- Test builder with custom `max_parallel`
- Test builder with custom `state_dir`
- Test validation: empty name should error
- Test validation: `max_parallel(0)` should error

### Step 4: Keep Task API Unchanged

**File:** `workflow_core/src/task.rs`

No changes needed - current API is already optimal.

### Step 5: Update Documentation

Add rustdoc comments on the builder method explaining:
- Available configuration options (`name`, `state_dir`, `max_parallel`)
- Default values
- Example usage

## Benefits

### For Layer 3 Users

**Consistent API style:**
```rust
// Task: consuming builder for construction
Task::new("id", closure)
    .depends_on("a")
    .add_monitor(hook)

// Workflow: builder for construction, mutation for tasks
let mut wf = Workflow::builder()
    .name("wf")
    .max_parallel(8)
    .build()?;
wf.add_task(task1)?;
wf.add_task(task2)?;
wf.run()?;
```

**Exposed configuration:**
- `max_parallel` now configurable (was hardcoded)
- `state_dir` explicit in builder (was positional parameter)
- Future config options easy to add

**Migration required (minor breaking change):**
- Old: `Workflow::new("name")?` 
- New: `Workflow::builder().name("name").build()?`
- Old: `Workflow::with_state_dir("name", "dir")?`
- New: `Workflow::builder().name("name").state_dir("dir").build()?`
- Simple find-replace migration

**IDE-friendly:**
- Autocomplete shows all options
- Doc comments on each setter
- Type-safe defaults

## Trade-offs

### Accepted

1. **Compilation time:** `bon` adds ~10-20% to workflow_core compile time (acceptable)
2. **Dependency weight:** One additional proc-macro crate (minimal impact)
3. **Learning curve:** Users need to understand when to use builder vs constructor (mitigated by docs)

### Avoided

1. **Closure complexity:** Task keeps simple `new(id, closure)` - no builder needed
2. **Mutable reference chaining:** `add_task()` keeps standard signature - no awkward `&mut Self` returns
3. **Over-engineering:** Only apply builder where it adds value (Workflow config)

## Critical Files

### To Modify

1. `workflow_core/Cargo.toml` - Add `bon = "3.9.1"`
2. `workflow_core/src/workflow.rs` - Replace constructors with builder
3. `workflow_core/src/workflow.rs` (tests) - Update all test cases to use builder API

## Verification

### Test Cases

1. **Builder with defaults:**
   ```rust
   let wf = Workflow::builder().name("test").build()?;
   assert_eq!(wf.name, "test");
   assert_eq!(wf.max_parallel, num_cpus());
   ```

2. **Builder with custom config:**
   ```rust
   let wf = Workflow::builder()
       .name("test")
       .max_parallel(4)
       .state_dir("/tmp")
       .build()?;
   assert_eq!(wf.max_parallel, 4);
   ```

3. **Validation - zero parallelism:**
   ```rust
   let result = Workflow::builder().name("test").max_parallel(0).build();
   assert!(result.is_err());
   ```

4. **Task addition:**
   ```rust
   let mut wf = Workflow::builder().name("test").build()?;
   wf.add_task(Task::new("a", || Ok(())))?;
   wf.add_task(Task::new("b", || Ok(())))?;
   wf.run()?;
   ```


## Summary

**Simplified hybrid approach:**
- **Task:** Keep current API unchanged (already excellent)
- **Workflow:** Replace constructors with `bon` builder
- **Mutation:** Keep standard `add_task(&mut self)` - no mutable reference chaining

**Key decisions:**
1. Use `Option<usize>` for `max_parallel` with runtime default (not compile-time)
2. Add validation in constructor (empty name, zero parallelism)
3. Accept minor breaking change for cleaner long-term API
4. Avoid non-idiomatic mutable reference chaining

**Result:** Clean, configurable API for Layer 3 users. Simple migration path for existing code.
