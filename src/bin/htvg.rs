use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use htvg::{compile_document, CompileOptions, CompileResult};

#[derive(Parser)]
#[command(name = "htvg", version, about = "HTVG - JSON element tree to SVG compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile an HTVG document to SVG
    Compile {
        /// Input JSON file (self-contained document with meta + content)
        input: PathBuf,

        /// Output SVG file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Override output width
        #[arg(short, long)]
        width: Option<f32>,

        /// Pretty-print the SVG output
        #[arg(long)]
        pretty: bool,
    },
    /// Print version info
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            width,
            pretty: _pretty,
        } => {
            let json = match fs::read_to_string(&input) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            // Try self-contained document format first, fall back to bare element
            let result: CompileResult = match compile_document(&json) {
                Ok(r) => {
                    // Apply CLI overrides
                    if let Some(w) = width {
                        // Re-compile with overridden width
                        let doc: htvg::HtvgDocument = serde_json::from_str(&json).unwrap();
                        let mut opts = doc.meta;
                        opts.width = w;
                        htvg::compile_element(&doc.content, &opts).unwrap_or(r)
                    } else {
                        r
                    }
                }
                Err(_) => {
                    // Try bare element format
                    let opts = CompileOptions {
                        width: width.unwrap_or(800.0),
                        ..CompileOptions::default()
                    };
                    match htvg::compile(&json, &opts) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Compile error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            };

            let svg = &result.svg;

            match output {
                Some(path) => {
                    if let Err(e) = fs::write(&path, svg) {
                        eprintln!("Error writing {}: {}", path.display(), e);
                        std::process::exit(1);
                    }
                    eprintln!(
                        "Wrote {} ({}x{})",
                        path.display(),
                        result.width,
                        result.height
                    );
                }
                None => {
                    println!("{}", svg);
                }
            }
        }
        Commands::Version => {
            println!("htvg {}", env!("CARGO_PKG_VERSION"));
        }
    }
}
