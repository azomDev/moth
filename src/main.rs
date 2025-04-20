mod hot_reload;
mod idk;
mod parsing;
mod types;

use hot_reload::serve_hot_reload;
use idk::transpile;
use std::{fs::write, path::Path};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
	name = "moth",
	version = "0.1",
	about = "A CLI for working with moth's own markdown format"
)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	Serve(ServeArgs),
	Transpile(TranspileArgs),
}

#[derive(Parser)]
struct ServeArgs {
	#[arg()]
	file: String,

	#[arg(long, default_value_t = 8000)]
	ws_port: u16,

	#[arg(long, default_value_t = 8001)]
	http_port: u16,
}

#[derive(Parser)]
struct TranspileArgs {
	#[arg()]
	file: String,

	#[arg(long, default_value_t = false)]
	no_base_64: bool,

	#[arg(long, default_value_t = String::from("output.html"))]
	out_file: String,
}

fn main() {
	let args = Cli::parse();

	match args.command {
		Commands::Serve(serve_args) => {
			println!("{}", serve_args.ws_port);
			serve_hot_reload(serve_args.file, serve_args.ws_port, serve_args.http_port);
		}
		Commands::Transpile(transpile_args) => {
			let html_content = transpile(&transpile_args.file, !transpile_args.no_base_64);
			let output_path = Path::new(&transpile_args.out_file);
			match write(output_path, html_content) {
				Ok(_) => println!("File successfully written to {}", transpile_args.out_file),
				Err(e) => eprintln!("Error writing file: {}", e),
			}
		}
	}
}
