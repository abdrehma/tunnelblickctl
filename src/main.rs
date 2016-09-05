#[macro_use]
extern crate clap;
extern crate tabwriter;

use std::env;
use std::error::Error;
use std::io::prelude::*;
use std::process::{Command, Stdio};
use tabwriter::TabWriter;

mod cli;

struct Tunnelblick {
    command: String,
    args: Vec<String>,
    script: String,
}

impl Tunnelblick {
    fn command(command: &str) -> Tunnelblick {
        Tunnelblick {
            command: command.to_owned(),
            args: Vec::new(),
            script: include_str!("tunnelblick.applescript").to_owned(),
        }
    }

    fn arg(&mut self, arg: &str) -> &mut Tunnelblick {
        self.args.push(arg.to_owned());
        self
    }

    fn args(&mut self, args: &[&str]) -> &mut Tunnelblick {
        for arg in args {
            self.arg(arg);
        }
        self
    }

    fn compile_script(&self) -> String {
        let command = match self.command.as_ref() {
            "run" => String::from("run Tunnelblick"),
            _ => {
                format!("tell Tunnelblick to {}({})",
                        self.command,
                        // Quote all arguments when rendering script.
                        self.args
                            .iter()
                            .map(|arg| format!("{:?}", arg))
                            .collect::<Vec<String>>()
                            .join(","))
            }
        };
        return format!("{}\n{}", self.script, command);
    }

    fn send(&self) -> String {
        let script = self.compile_script();

        let process = match Command::new("osascript")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn() {
            Err(why) => panic!("couldn't spawn osascript: {}", why.description()),
            Ok(process) => process,
        };

        match process.stdin.unwrap().write_all(script.as_bytes()) {
            Err(why) => panic!("couldn't write to osascript stdin: {}", why.description()),
            Ok(_) => {}
        }

        let mut s = String::new();
        match process.stdout.unwrap().read_to_string(&mut s) {
            Err(why) => panic!("couldn't read osascript stdout: {}", why.description()),
            Ok(_) => {}
        }

        match self.command.as_ref() {
            "showStatus" => {
                let mut tw = TabWriter::new(Vec::new());
                tw.write(s.as_bytes()).unwrap();
                tw.flush().unwrap();
                return String::from_utf8(tw.unwrap()).unwrap();
            }
            _ => return s,
        }
    }
}

fn version() -> String {
    let cli_version = crate_version!();
    let app_version = Tunnelblick::command("getVersion").send();
    return format!("{} {}\nTunnelblick {}",
                   env!("CARGO_PKG_NAME"),
                   cli_version,
                   app_version);
}

fn main() {
    let matches = cli::cli().get_matches();

    if matches.is_present("version") {
        print!("{}", version());
        return;
    }

    let output = match matches.subcommand() {
        ("connect", Some(m)) => {
            Tunnelblick::command("connect").arg(m.value_of("name").unwrap()).send()
        }
        ("disconnect", Some(m)) => {
            Tunnelblick::command("disconnect").arg(m.value_of("name").unwrap()).send()
        }
        ("list", Some(_)) => Tunnelblick::command("listTunnels").send(),
        ("status", Some(_)) => Tunnelblick::command("showStatus").send(),
        ("quit", Some(_)) => Tunnelblick::command("quit").send(),
        ("launch", Some(_)) => Tunnelblick::command("run").send(),
        _ => "".to_owned(),
    };
    print!("{}", output);
}
