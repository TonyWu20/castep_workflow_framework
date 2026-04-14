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
