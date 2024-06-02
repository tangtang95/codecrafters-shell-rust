#[allow(unused_imports)]
use std::io::{self, Write};

use thiserror::Error;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        match parse_command(&input) {
            Ok(command) => match command {
                Command::NoInput => println!(),
                Command::Exit(exit_code) => std::process::exit(exit_code),
            },
            Err(error) => println!("{}", error),
        }
    }
}

const EXIT_CMD: &str = "exit";

enum Command {
    NoInput,
    Exit(i32),
}

#[derive(Error, Debug)]
enum Error {
    #[error("{0}: command not found")]
    CommandNotFound(String),
    #[error("Error during parse of command {0}")]
    ParseCommandError(String),
}

fn parse_command(input: &str) -> Result<Command, Error> {
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
        Some(command) => Err(Error::CommandNotFound(command.to_string())),
        None => Ok(Command::NoInput),
    }
}
