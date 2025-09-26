use std::io::{
	self,
	Write,
};

/// Runs the interactive Read-Eval-Print Loop (REPL).
///
/// This function provides a simple shell for running rinkle commands. It reads
/// lines from stdin, parses them, and re-executes the main rinkle binary with
/// the provided arguments. It handles `exit`, `quit`, and `help` internally.
pub fn run() -> std::io::Result<()> {
	println!("Interactive mode. Type 'help' or 'exit'.");
	loop {
		print!("> ");
		io::stdout().flush()?;
		let mut line = String::new();
		if io::stdin().read_line(&mut line)? == 0 {
			break;
		}
		let line = line.trim();
		if line.is_empty() {
			continue;
		}
		if matches!(line, "exit" | "quit") {
			break;
		}
		if line == "help" {
			println!(
				"Commands: list | status [--json] | link <pkg[@ver]>... | \
				 remove <pkg>... | use-profile <name> | vsc <pkg> <ver> | exit"
			);
			continue;
		}

		let parts: Vec<_> = line.split_whitespace().collect();
		if parts.is_empty() {
			continue;
		}

		// Execute command by spawning self
		if let Err(e) = execute_command(&parts) {
			eprintln!("command failed: {}", e);
		}
	}
	Ok(())
}

fn execute_command(args: &[&str]) -> std::io::Result<()> {
	let exe = std::env::current_exe()?;
	let status = std::process::Command::new(exe).args(args).status()?;

	if !status.success() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			format!("command {:?} failed", args),
		));
	}
	Ok(())
}
