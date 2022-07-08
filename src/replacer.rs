use std::any::Any;
use std::borrow::{Cow};
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use crate::{Config, LangFile, RuntimeOptions};
use crate::generator::Generator;
use crate::lang::LangResult;
use crate::meta::GeneratorMeta;

pub trait GroupRepository<'a> {
    fn get_group_results(&self, group: &str) -> Ref<Vec<Box<dyn LangResult<'a>>>>;
}

#[derive(Debug)]
pub struct Replacer<'a, G> where G: Generator<'a> + Eq + Hash {
    pub config: Config,
    pub options: RuntimeOptions,
    generators: Vec<&'a G>,
    result_cache: RefCell<HashMap<&'a G, Vec<Box<dyn LangResult<'a>>>>>,
    group_cache: RefCell<HashMap<String, Vec<Box<dyn LangResult<'a>>>>>,
}

impl<'a, G> Replacer<'a, G> where G: Generator<'a> + Eq + Hash {
    pub fn get_generator_results(self: &Box<Self>, gen: &'a G) -> Option<Vec<Box<dyn LangResult<'a>>>> {
        if let Some(rcache_found) = self.result_cache.borrow().get(gen) {
            if self.result_cache.borrow().iter().last().map(|x| x.1 == rcache_found).unwrap_or(false) {
                let new_results = gen.results(self as &Box<dyn GroupRepository<'a>>);
                self.result_cache.borrow_mut().insert(gen, new_results.clone());
                return Some(new_results);
            } else {
                return Some(rcache_found.clone());
            }
        }
        None
    }

    pub fn replace(self: &mut Box<Self>) -> std::io::Result<()> {
        let main_source = LangFile::read(self.options.main_source_path.as_ref().unwrap())?;

        let mut extra_dict: HashMap<String, String> = HashMap::new();

        let mut valid_extra_source = false;
        if let Some(extra_lang) = self.options.extra_source_path.as_ref().and_then(|x| LangFile::read(x).ok()) {
            extra_lang.items.into_iter().for_each(|(k, v)| { extra_dict.insert(k, v); });
            valid_extra_source = true;
        }

        let extra_target_path = self.options.extra_target_path.as_ref();

        let mut main_target = LangFile::default();
        let mut extra_target = LangFile::default();
        let mut extra_source_removal = HashSet::new();

        for (key, source_text) in main_source.items.into_iter() {
            let mut succ = false;
            let mut succ_dict = false;
            let mut succ_extra = false;

            let mut target_text = String::default();
            let mut target_text_dict = String::default();
            let mut target_text_extra = String::default();

            // dict
            let config_extensions = &self.options.extensions;
            let source_meta = GeneratorMeta {
                namespace: key.clone(),
                extensions: config_extensions.clone().into_iter().map(|x| x.to_str().unwrap().to_string()).collect(),
                ..Default::default()
            };

            let dict = self.generate_map();
            for (meta, lang_result) in dict.iter().filter(|(_, x)| x == &source_text) {
                if meta.as_ref().eq(&source_meta) {
                    target_text_dict = lang_result.to_string();
                    succ_dict = true;
                }
            }

            // extra
            if valid_extra_source {
                if let Some((_, extra)) = extra_dict.iter().find(|(_, x)| &key == *x) {
                    target_text_extra = extra.clone();
                    succ_extra = true;
                }
            }

            // judge
            if succ_dict && succ_extra {
                // CONFLICT
                target_text = target_text_dict.clone();
                succ = true;
            } else if succ_dict && !succ_extra {
                // REPLACED
                target_text = target_text_dict.clone();
                succ = true;
            } else if !succ_dict && succ_extra {
                // FALLBACK
                target_text = target_text_extra.clone();
                succ = true;
            } else if !succ_dict && !succ_extra {
                // FAILED
                target_text = source_text.clone();
                succ = false;
            }

            main_target.items.push((key.clone(), target_text.clone()));
            if extra_target_path.is_some() {
                // output all unknown items
                if !succ {
                    extra_target.items.push((key.clone(), target_text.clone()));
                }
            }
            if self.options.remove_redundant_fallback {
                // remove matched
                if succ_extra && succ_dict {
                    extra_source_removal.insert(key.clone());
                }
            }
        }

        // write

        let main_target_path = self.options.main_target_path.as_ref().expect("No target path");
        main_target.write(main_target_path)?;
        if let Some(extra_target_path) = extra_target_path {
            extra_target.write(extra_target_path)?;
        }
        if self.options.remove_redundant_fallback {
            let mut extra_source_modified = LangFile::default();
            for (k, v) in extra_dict.iter() {
                if !extra_source_removal.contains(k) {
                    extra_source_modified.items.push((k.to_string(), v.to_string()));
                }
            }
            extra_source_modified.write(self.options.extra_source_path.as_ref().unwrap())?;
        }
        Ok(())
    }

    pub fn generate(self: &Box<Self>) -> Vec<Box<dyn LangResult<'a>>> {
        let mut results = vec![];
        for x in self.generators.iter() {
            if x.meta().completed {
                let new_results = self.get_generator_results(x);
                if let Some(new_results) = new_results {
                    new_results.into_iter().for_each(|x| results.push(x));
                }
            }
        }

        results
    }

    pub fn generate_map(self: &Box<Self>) -> Vec<(Cow<'a, GeneratorMeta>, String)> {
        let mut result = vec![];
        let lang_list = self.generate();
        for lang_item in lang_list.into_iter() {
            let meta: Cow<'a, GeneratorMeta> = lang_item.meta();
            let lang_generated = lang_item.result();
            for (_, dst) in lang_generated.into_owned().into_iter() {
                // let meta: Cow<'a, GeneratorMeta> = lang_item.meta();

                result.push((meta.clone(), dst));
            }
        }
        result
    }
}

impl<'a, G> GroupRepository<'a> for Box<Replacer<'a, G>> where G: Generator<'a> + Eq + Hash {
    fn get_group_results(&self, group: &str) -> Ref<Vec<Box<(dyn LangResult<'a> + 'a)>>> {
        if self.group_cache.borrow().contains_key(group) {
            Ref::map(self.group_cache.borrow(), |group_cache| {
                group_cache.get(group).unwrap()
            })
        } else {
            let mut results = vec![];
            for gen in self.generators.clone().into_iter() {
                if &gen.meta().group == group {
                    if let Some(new_results) = self.get_generator_results(gen.clone()) {
                        new_results.into_iter().for_each(|x| results.push(x));
                    }
                }
            }
            self.group_cache.borrow_mut().insert(group.to_string(), results);
            Ref::map(self.group_cache.borrow(), |group_cache| {
                group_cache.get(group).unwrap()
            })
        }
    }
}