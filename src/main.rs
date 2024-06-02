use std::collections::HashMap;
use std::io::{self, Write};

use thiserror::Error;

fn main() {
    let path_env = std::env::var("PATH").unwrap();
    let path_entries = path_env.split(':').collect::<Vec<&str>>();
    let mut command_map: HashMap<String, String> = HashMap::new();
    for path_entry in path_entries {
        let dir = std::fs::read_dir(path_entry);
        if let Ok(dir) = dir {
            for dir_entry in dir {
                let dir_entry = dir_entry.unwrap();
                let filename = dir_entry.file_name().into_string().unwrap();
                if command_map.get(&filename).is_none() {
                    command_map.insert(filename, path_entry.to_string());
                }
            }
        }
    }

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        match parse_command(&input, &command_map) {
            Ok(command) => match command {
                Command::NoInput => println!(),
                Command::Exit(exit_code) => std::process::exit(exit_code),
                Command::Echo(text) => println!("{}", text),
                Command::Type(cmd) => match cmd.as_ref() {
                    EXIT_CMD | ECHO_CMD | TYPE_CMD => println!("{} is a shell builtin", cmd),
                    cmd => match command_map.get(cmd) {
                        Some(path_src) => println!("{} is {}/{}", cmd, path_src, cmd),
                        None => println!("{} not found", cmd),
                    },
                },
                Command::ExternalProgram {
                    executable_path,
                    args,
                } => {
                    let mut program = std::process::Command::new(executable_path);
                    match program.args(args).output() {
                        Ok(output) => {
                            io::stdout().write_all(&output.stdout).unwrap();
                            io::stderr().write_all(&output.stderr).unwrap();
                        }
                        Err(err) => println!("{}", err),
                    }
                }
            },
            Err(error) => println!("{}", error),
        }
    }
}

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";

enum Command {
    NoInput,
    Exit(i32),
    Echo(String),
    Type(String),
    ExternalProgram {
        executable_path: String,
        args: Vec<String>,
    },
}

#[derive(Error, Debug)]
enum Error {
    #[error("{0}: command not found")]
    CommandNotFound(String),
    #[error("Error during parse of command {0}")]
    ParseCommandError(String),
}

fn parse_command(input: &str, command_map: &HashMap<String, String>) -> Result<Command, Error> {
    let mut splitted_input = input.trim().split(' ');
    match splitted_input.next() {
        Some(EXIT_CMD) => {
            let exit_code = splitted_input.next().unwrap_or("0");
            let exit_code = exit_code
                .parse::<i32>()
                .map_err(|_| Error::ParseCommandError(EXIT_CMD.to_string()))?;
            match splitted_input.next() {
                Some(_) => Err(Error::ParseCommandError(EXIT_CMD.to_string())),
                None => Ok(Command::Exit(exit_code)),
            }
        }
        Some(ECHO_CMD) => {
            let echo_text = splitted_input.collect::<Vec<&str>>().join(" ");
            Ok(Command::Echo(echo_text.to_string()))
        }
        Some(TYPE_CMD) => {
            let command_name = splitted_input.collect::<Vec<&str>>().join(" ");
            Ok(Command::Type(command_name))
        }
        Some(command) => match command_map.get(command) {
            Some(path_src) => {
                let executable_path = format!("{}/{}", path_src, command);
                let args = splitted_input
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>();
                Ok(Command::ExternalProgram {
                    executable_path,
                    args,
                })
            }
            None => Err(Error::CommandNotFound(command.to_string())),
        },
        None => Ok(Command::NoInput),
    }
}
