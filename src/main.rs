use std::env;

use std::io::{stdin, stdout, StdoutLock, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone)]
struct ShellCommand {
    command: String,
    args: Vec<String>,
}

impl ShellCommand {
    fn new(command: &str) -> Self {
        let mut parts = command.split_whitespace();
        let command = parts.next().unwrap().into();
        let args = parts.map(String::from).collect();

        Self { command, args }
    }

    fn execute(&self, previous_command: Option<Child>, stdout: Stdio) -> std::io::Result<Child> {
        let stdin = previous_command.map_or(Stdio::inherit(), |output| {
            Stdio::from(output.stdout.unwrap())
        });

        Command::new(&self.command)
            .args(&self.args)
            .stdin(stdin)
            .stdout(stdout)
            .spawn()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = stdout().lock();
    loop {
        display_prompt(&mut lock);
        let commands = get_commands()?;
        let mut previous_command: Option<Child> = None;

        for (i, command) in commands.iter().enumerate() {
            let stdout = commands
                .get(i + 1)
                .map_or(Stdio::inherit(), |_| Stdio::piped());

            match command.command.as_str() {
                "cd" => {
                    change_dir(&command.args)?;
                    previous_command = None;
                }
                "exit" => return Ok(()),
                _ => match command.execute(previous_command, stdout) {
                    Ok(output) => previous_command = Some(output),
                    Err(e) => {
                        eprintln!("{}", e);
                        previous_command = None;
                    }
                },
            }
        }

        if let Some(mut command) = previous_command {
            command.wait()?;
        }
    }
}

fn display_prompt(lock: &mut StdoutLock) {
    write!(lock, "=> ").unwrap();
    stdout().flush().unwrap();
}

fn get_commands() -> std::io::Result<Vec<ShellCommand>> {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    let commands = input.trim().split(" | ");
    Ok(commands.map(ShellCommand::new).collect())
}

fn change_dir(args: &[String]) -> std::io::Result<()> {
    let default_path = String::from("/");
    let new_dir = args.get(0).unwrap_or(&default_path);
    let path = Path::new(new_dir);
    env::set_current_dir(path)
}
