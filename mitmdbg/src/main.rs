use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mitmdbg")]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Tui,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Tui) | None => {
            std::fs::create_dir_all("workspace")?;
            let logs = logs::init(std::path::Path::new("workspace/log.txt"), 1000)?;
            tracing::info!("mitmdbg starting");
            let backend = backend::mock::run()?;
            tui::run(logs, backend)?;
        }
    }
    Ok(())
}
