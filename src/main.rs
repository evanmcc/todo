use chrono::prelude::*;
use chrono::Weekday::*;
use clap::{App, Arg, SubCommand};
use std::convert::TryInto;
use std::io;
use std::io::prelude::*;
use std::time::SystemTime;
use std::{
    env, fs,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

use toml::Value;
use toml::Value::Table;

fn main() {
    let matches = App::new("todo")
        .version("0.1.0")
        .author("pevm <mcclanahan@gmail.com>")
        .about("bad todo application")
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .takes_value(false)
                .help("return only ok or notok depending on whether you're done"),
        )
        .subcommand(
            SubCommand::with_name("done")
                .about("marks an item as done")
                .arg(
                    Arg::with_name("item")
                        .index(1)
                        .required(true)
                        .help("print debug information verbosely"),
                ),
        )
        .get_matches();

    let quiet = match matches.occurrences_of("q") {
        0 => false,
        1 => true,
        _ => true,
    };

    let home_dir = env::var("HOME").unwrap();

    let todo_path = Path::new(&home_dir).join(".todo");
    let config_path = todo_path.clone().join("todo.toml");

    match fs::metadata(&todo_path) {
        Ok(_) => (),
        Err(_) => {
            println!("~/.todo does not exist");
            return;
        }
    };

    let config = match get_config(config_path) {
        Ok(Table(config)) => config,
        Ok(_) => {
            println!("didn't understand config");
            return;
        }
        Err(error) => {
            println!("error: {}", error);
            return;
        }
    };

    match matches.subcommand_name() {
        Some("done") => {
            println!("done");
            if let Some(ref matches) = matches.subcommand_matches("done") {
                if let Some(item) = matches.value_of("item") {
                    println!("item {}", item);
                    match config.get(item) {
                        Some(_) => {
                            let touch_path = todo_path.join(item);
                            let res = touch(touch_path);
                            println!("ok: {:?}", res);
                        }
                        _ => println!("unknown item {}", item),
                    }
                } else {
                    println!("matches {:?}", matches);
                }
            } else {
                println!("clap error");
            }
        }
        _ => {
            let mut all_good = true;

            for (name, interval) in config {
                let check_path = todo_path.clone().join(name.clone());
                //println!("check path {}", check_path.display());
                let good = match fs::metadata(check_path) {
                    Ok(md) => {
                        if let Ok(time) = md.modified() {
                            if check_interval(time, interval) {
                                if !quiet {
                                    println!("{}: ok", &name);
                                }
                                true
                            } else {
                                // print how old?
                                if !quiet {
                                    println!("{}: not_ok", &name);
                                }
                                false
                            }
                        } else {
                            false
                        }
                    }
                    Err(_) => {
                        if !quiet {
                            println!("{}: missing", &name);
                        }
                        false
                    }
                };

                if !good {
                    all_good = false;
                }
            }
            if all_good {
                println!("ok");
            } else {
                println!("not_ok");
            }
        }
    }
}

fn check_interval(mod_time: std::time::SystemTime, interval: Value) -> bool {
    let now = SystemTime::now();
    if let Ok(duration) = now.duration_since(mod_time) {
        match interval {
            Value::Integer(i) => duration.as_secs() < (i * 24 * 60 * 60).try_into().unwrap(),
            Value::String(s) => match s.as_str() {
                "weekdays" => {
                    if is_weekday() {
                        duration.as_secs() < (24 * 60 * 60)
                    } else {
                        duration.as_secs() < (7 * 24 * 60 * 60)
                    }
                }
                _ => {
                    println!("unknown");
                    false
                }
            },
            _ => false,
        }
    } else {
        panic!("what the hell");
    }
}

fn get_config(config_path: PathBuf) -> Result<Value, String> {
    let mut file = match File::open(&config_path) {
        Ok(file) => file,
        Err(_) => {
            return Err("config does not exist".to_string());
        }
    };

    let mut config_toml = String::new();

    file.read_to_string(&mut config_toml)
        .unwrap_or_else(|err| panic!("Error while reading config: [{}]", err));

    match config_toml.parse::<Value>() {
        Ok(config) => Ok(config),
        Err(_error) => Err("could not read config".to_string()),
    }
}

fn touch(path: PathBuf) -> io::Result<()> {
    match OpenOptions::new().create(true).write(true).open(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn is_weekday() -> bool {
    let today: DateTime<Local> = Local::now();
    match today.weekday() {
        Mon | Tue | Wed | Thu | Fri => true,
        _ => false,
    }
}
