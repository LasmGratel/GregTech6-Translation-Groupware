use clap::Parser;
use crate::config::{Config, RuntimeOptions};
use crate::lang::LangFile;

fn main() {
    let options: RuntimeOptions = RuntimeOptions::parse().determine_paths();
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(options.config_path.unwrap()).expect("Cannot read config file")).expect("Cannot parse config");
    println!("{:?}", config.generators);
    let mut lang = LangFile::read("workplace/en/GregTech.lang").unwrap();
    // println!("{:?}", lang);
}

pub mod config;
pub mod generator;
pub mod meta;
pub mod rule;
pub mod replacer;
pub mod lang;
pub mod result;