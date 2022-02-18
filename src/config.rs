use std::ffi::OsString;
use std::path::Path;
use clap::{Args, Command, Parser};
use serde::{Serialize, Deserialize};
use crate::meta::GeneratorMeta;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub lang: String,
    pub generators: Vec<GeneratorMeta>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            lang: String::from("zh"),
            version: String::new(),
            generators: vec![],
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "gt6tg")]
#[clap(about = "GregTech 6 Translation Groupware")]
pub struct RuntimeOptions {
    /// Source file
    #[clap(short = 's', long = "source")]
    pub main_source_path: Option<OsString>,

    /// Extra source file
    #[clap(long = "extra_source")]
    pub extra_source_path: Option<OsString>,

    /// Target file
    #[clap(short = 't', long = "target")]
    pub main_target_path: Option<OsString>,

    /// Extra target file
    #[clap(long = "extra_target")]
    pub extra_target_path: Option<OsString>,

    /// config file
    #[clap(short, long = "config")]
    pub config_path: Option<OsString>,

    /// workplace for language files and configs
    #[clap(short, long = "workplace", default_value = "workplace")]
    pub workplace_path: OsString,

    /// language code
    #[clap(short, long = "language", default_value = "zh")]
    pub lang: String,

    #[clap(short, long = "extensions")]
    pub extensions: Vec<OsString>,

    #[clap(short, long = "remove", parse(try_from_str), default_value_t = false)]
    pub remove_redundant_fallback: bool
}

impl RuntimeOptions {
    pub fn determine_paths(self) -> Self {
        let mut this = self;
        if this.main_source_path.is_none() {
            this.main_source_path = Some(Path::new(&this.workplace_path).join("en").join("GregTech.lang").into_os_string())
        }
        if this.extra_source_path.is_none() {
            this.extra_source_path = Some(Path::new(&this.workplace_path).join(&this.lang).join("GregTech.fallback.lang").into_os_string())
        }
        if this.main_target_path.is_none() {
            this.main_target_path = Some(Path::new(&this.workplace_path).join(&this.lang).join("GregTech.lang").into_os_string())
        }
        if this.extra_target_path.is_none() {
            this.extra_target_path = Some(Path::new(&this.workplace_path).join(&this.lang).join("GregTech.unknown.lang").into_os_string())
        }
        if this.config_path.is_none() {
            this.config_path = Some(Path::new(&this.workplace_path).join("config.yml").into_os_string())
        }

        this
    }
}