# TASK-18: Delete `workflow_cli`, create `workflow-cli` skeleton

- **Scope**: Remove old `workflow_cli/` directory, scaffold new `workflow-cli` binary crate.
- **Crate/Module**: `workflow-cli/` (NEW), `Cargo.toml` (workspace)
- **Depends On**: TASK-14
- **Enables**: TASK-19, TASK-20
- **Can Run In Parallel With**: TASK-17

### Acceptance Criteria

- `workflow_cli/` deleted.
- `"workflow-cli"` in workspace members; `clap = { version = "4", features = ["derive"] }` in `[workspace.dependencies]`.
- `workflow-cli/Cargo.toml`: `name = "workflow-cli"`, `[[bin]]`, deps: `workflow_core` (path), `clap` (workspace), `anyhow` (workspace).
- Skeleton subcommands `Status`, `Retry`, `Inspect` — all `todo!()`.
- `cargo check -p workflow-cli` passes.

### Implementation

**`workflow-cli/Cargo.toml`**:

```toml
[package]
name = "workflow-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "workflow-cli"
path = "src/main.rs"

[dependencies]
workflow_core = { path = "../workflow_core" }
clap = { workspace = true }
anyhow = { workspace = true }
```

**`workflow-cli/src/main.rs`**:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "workflow-cli", about = "Workflow state inspection tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Status { state_file: String },
    Retry {
        state_file: String,
        #[arg(required = true)]
        task_ids: Vec<String>,
    },
    Inspect {
        state_file: String,
        task_id: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status { state_file } => todo!(),
        Commands::Retry { state_file, task_ids } => todo!(),
        Commands::Inspect { state_file, task_id } => todo!(),
    }
}
```

**Verify**: `cargo check -p workflow-cli`
