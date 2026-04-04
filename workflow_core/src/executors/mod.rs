pub mod local;
pub mod slurm;
pub use local::LocalExecutor;
pub use slurm::SlurmExecutor;
