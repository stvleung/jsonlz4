//! Command-line interface for the [`jsonlz4`] crate.

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{ArgAction, Parser};

/// Compress or decompress Mozilla Firefox `.jsonlz4` / `.mozlz4` files.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Compress IN_FILE instead of decompressing.
    #[arg(short = 'c', long = "compress", action = ArgAction::SetTrue,
          conflicts_with = "decompress")]
    compress: bool,

    /// Decompress IN_FILE (the default).
    #[arg(short = 'd', long = "decompress", action = ArgAction::SetTrue)]
    decompress: bool,

    /// Input file. Use `-` to read from standard input.
    #[arg(value_name = "IN_FILE")]
    input: PathBuf,

    /// Output file. Use `-` (or omit) to write to standard output.
    #[arg(value_name = "OUT_FILE")]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // The helper formats with full chain; clap already prints its own.
            eprintln!("jsonlz4: error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let input = read_input(&cli.input)?;

    let output = if cli.compress {
        jsonlz4::encode(&input)?
    } else {
        jsonlz4::decode(&input)?
    };

    write_output(cli.output.as_deref(), &output)?;
    Ok(())
}

fn read_input(path: &Path) -> io::Result<Vec<u8>> {
    if is_dash(path) {
        let mut buf = Vec::new();
        io::stdin().lock().read_to_end(&mut buf)?;
        Ok(buf)
    } else {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

fn write_output(path: Option<&Path>, data: &[u8]) -> io::Result<()> {
    match path {
        None => write_stdout(data),
        Some(p) if is_dash(p) => write_stdout(data),
        Some(p) => File::create(p)?.write_all(data),
    }
}

fn write_stdout(data: &[u8]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(data)?;
    handle.flush()
}

fn is_dash(path: &Path) -> bool {
    path.as_os_str() == "-"
}
