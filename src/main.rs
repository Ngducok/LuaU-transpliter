use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hluau")]
#[command(about = "HTML/CSS to Luau UI transpiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compile {
        #[arg(long)]
        html: PathBuf,
        #[arg(short, long)]
        css: Option<PathBuf>,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        standalone: bool,
        #[arg(long)]
        theme: Option<PathBuf>,
        #[arg(long)]
        assets: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Compile {
            html,
            css,
            output,
            standalone,
            theme: _,
            assets: _,
        } => {
            let html_content = fs::read_to_string(&html)?;
            let css_content = css
                .map(|p| fs::read_to_string(p))
                .transpose()?
                .unwrap_or_default();
            let luau = if standalone {
                hluau::compile_standalone(&html_content, &css_content)?
            } else {
                hluau::compile(&html_content, &css_content)?
            };
            if let Some(out) = output {
                fs::write(out, luau)?;
            } else {
                println!("{}", luau);
            }
        }
    }
    Ok(())
}
