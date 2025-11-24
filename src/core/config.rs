use clap::{Parser, Subcommand};
use std::path::PathBuf;
#[derive(Parser, Debug, Clone)]
#[command(name = "ink", author, version, about)]
pub struct Config {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(global = true)]
    pub file: Option<PathBuf>,
    #[arg(long, global = true)]
    pub main_path: Option<PathBuf>,
    #[arg(long, global = true)]
    pub windowed: bool,
}
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Init {
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
}
impl Config {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
