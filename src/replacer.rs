use std::collections::HashMap;
use crate::Config;
use crate::generator::Generator;
use crate::lang::LangResult;

pub trait GroupRepository {

}

pub struct Replacer<'a> {
    config: &'a Config,
    generators: Vec<&'a Generator<'a>>,
    result_cache: HashMap<&'a Generator<'a>, Vec<Box<&'a dyn LangResult>>>,
    group_cache: HashMap<&'a str, Vec<Box<&'a dyn LangResult>>>,
}

impl<'a> Replacer<'a> {
    pub fn get_generator_results(&self, gen: &'a Generator<'a>) {

    }

    pub fn replace(&mut self) {

    }
}