use std::{env, process};

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub windowed: bool,
}

impl Config {
    pub fn parse() -> Self {
        let mut config = Self::default();
        let args: Vec<String> = env::args().collect();

        for arg in &args[1..] {
            match arg.as_str() {
                "--windowed" => {
                    config.windowed = true;
                }
                "--help" => {
                    println!("Usage: {} [FLAGS] [FILE]", args[0]);
                    println!("\nA simple UI renderer based on GTK4 and Lua.");
                    println!("\nFlags:");
                    println!("    --windowed    Force the UI to run in a regular window.");
                    println!("    --help        Display this help message and exit.");
                    process::exit(0);
                }
                _ => {}
            }
        }
        config
    }
}
