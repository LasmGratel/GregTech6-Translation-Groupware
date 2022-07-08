use std::ffi::OsString;
use std::path::Path;
use clap::{Parser};
use either::Either;
use serde::{Serialize, Deserialize};
use crate::generator::{DictGenerator, Generator, RuleGenerator};
use crate::lang::LangResult;
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

    pub fn generators(&self) {
        let mut list: Vec<Box<dyn Generator>> = vec![];
        for meta in self.generators.iter() {
            if let Some(rules) = &meta.rules {
                // RuleGenerator
                list.push(Box::new(RuleGenerator {
                    meta, rules
                }));
            } else if let Some(dict) = &meta.dict {
                for (key, value) in dict.iter() {
                    if let Either::Left(str) = &value.inner {
                        list.push(Box::new(DictGenerator {
                            meta, dict: vec![(key.to_string(), str.to_string())]
                        }));

                        /*
                        if (dict_item.is_map()) {

                } else {
                    // normal dict
                    if (!gen) {
                        gen = std::make_shared<DictGenerator>(meta);
                        gen->dict() = std::make_shared<LangList>();
                    }
                    auto &dict = *(gen->dict());
                    dict.emplace_back(_csubstr2str(dict_item.key()),
                                      _csubstr2str(dict_item.val()));
                }*/
                    } else if let Either::Right(map) = &value.inner {
                        for (child_key, child_value) in map.iter() {
                            let mut dict = vec![];
                            dict.push((key.to_string(), child_value.to_string()));
                            list.push(Box::new(DictGenerator {
                                meta, dict
                            }));
                        }
                        /*
                        for (auto dict_item_children : dict_item.children()) {
                        auto local_gen = std::make_shared<DictGenerator>(
                            std::make_shared<NSGeneratorMeta>(
                                _csubstr2str(dict_item_children.key()), meta));
                        local_gen->dict() = std::make_shared<LangList>();
                        auto &dict = *(local_gen->dict());
                        dict.emplace_back(_csubstr2str(dict_item.key()),
                                          _csubstr2str(dict_item_children.val()));
                        result.push_back(std::move(local_gen));
                    }*/
                    }
                }
            }
        }
    }
/*
// DictGenerator
        if (child.is_map()) {
            shared_ptr<DictGenerator> gen = nullptr;
            for (auto dict_item : child.children()) {

            }
            if (gen) {
                // final commit
                result.push_back(std::move(gen));
            }
        } else {
            throw std::invalid_argument("invalid dict for generator");
        }
    } else if (ckey == "rules") {
    // RuleGenerator
    if (child.is_seq()) {
    auto gen = std::make_shared<RuleGenerator>(meta);
    auto &rules = gen->rules();
    for (auto rule : child.children()) {
    rules.push_back(parse_rule(rule));
    }
    result.push_back(std::move(gen));
    } else {
    throw std::invalid_argument("invalid rules for generator");
    }
    }
    }*/
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