use clap::Parser;
use exsb::Cli;

fn main() -> exsb::Result<()> {
    Cli::parse().execute()
}
