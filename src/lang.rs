use std::borrow::Cow;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use dyn_clone::DynClone;
use strfmt::strfmt;
use crate::meta::GeneratorMeta;
use crate::rule::Rule;

pub type LangItem = (String, String);

#[derive(Default, Debug)]
pub struct LangFile {
    pub items: Vec<LangItem>
}

pub trait LangResult<'a>: 'a + DynClone + Debug {
    fn meta(&self) -> Cow<'a, GeneratorMeta>;
    fn result(&self) -> Cow<'_, Vec<LangItem>>;
    fn is_empty(&self) -> bool;
}

impl<'a> PartialEq<Box<dyn LangResult<'a>>> for Box<dyn LangResult<'a>> {
    fn eq(&self, other: &Box<dyn LangResult<'a>>) -> bool {
        self.result() == other.result()
    }
}

dyn_clone::clone_trait_object!(LangResult<'_>);

#[derive(PartialEq, Clone, Debug)]
pub struct DictLangResult<'a> {
    pub meta: &'a GeneratorMeta,
    pub result: Vec<LangItem>
}

impl<'a> LangResult<'a> for DictLangResult<'a> {
    fn meta(&self) -> Cow<'a, GeneratorMeta> {
        Cow::Borrowed(self.meta)
    }

    fn result(&self) -> Cow<'_, Vec<LangItem>> {
        Cow::Borrowed(&self.result)
    }

    fn is_empty(&self) -> bool {
        self.result.is_empty()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RuleLangResult<'a: 'b, 'b> {
    pub rule: &'b Rule,
    pub meta: Cow<'a, GeneratorMeta>,
    pub subs: Vec<Box<dyn LangResult<'a>>>,
    pub result: Vec<LangItem>
}

impl<'a: 'b, 'b> RuleLangResult<'a, 'b> {
    pub fn meta_combined(&self) -> Option<GeneratorMeta> {
        if self.is_empty() {
            return None;
        }
        let mut meta_source = self.meta().clone().into_owned();
        for sub in self.subs.iter() {
            meta_source.combine(&sub.meta());
        }
        if meta_source.is_empty() {
            None
        } else {
            Some(meta_source)
        }
    }

    pub fn result(&self) -> Vec<LangItem> {
        self.generate()
    }

    pub fn result_cached(&mut self, cached: bool) -> Vec<LangItem> {
        if self.result.is_empty() {
            self.result = self.generate();
        }
        self.result.clone()
    }

    pub fn generate(&self) -> Vec<LangItem> {
        let subs_sz = self.subs.len();

        if self.is_empty() {
            return Vec::default();
        }
        // init
        let mut sub_results: Vec<LangItem> = vec![];
        self.subs.iter().for_each(|x| x.result().into_iter().for_each(|y| sub_results.push(y)));
        let mut begins = vec![];
        let mut result = vec![];
        let s_fmt = &self.rule.source;
        let t_fmt = &self.rule.target;
        let mut s_store = String::new();
        let mut t_store = String::new();

        loop {
            // fill zeros
            for i in begins.len()..subs_sz {
                begins.push(&sub_results[i]);
            }
            // generate
            s_store.clear();
            t_store.clear();
            for (source, target) in begins.iter() {
                s_store.push_str(source);
                t_store.push_str(target);
            }
            result.push((s_fmt.replace("{0}", &s_store), t_fmt.replace("{0}", &t_store)));
            // step
            for i in (0..(subs_sz + 1)).rev() {
                unsafe {
                    let mut ptr = (&*begins[i - 1]) as *const LangItem;
                    ptr = ptr.add(1);
                    begins[i - 1] = ptr.as_ref().unwrap();
                }

                if &begins[i - 1].0 == &sub_results[i - 1].0 {
                    begins.pop();
                } else {
                    break;
                }
            }

            if begins.len() <= 0 { break; }
        }

        return result;
    }
}

impl<'a, 'b: 'a> LangResult<'a> for RuleLangResult<'a, 'b> {
    fn meta(&self) -> Cow<'a, GeneratorMeta> {
        self.meta.clone()
    }

    fn result(&self) -> Cow<'_, Vec<LangItem>> {
        Cow::Owned(self.generate())
    }

    fn is_empty(&self) -> bool {
        if self.subs.is_empty() || self.rule.subs.len() != self.subs.len() {
            return true;
        }
        if self.subs.iter().any(|x| x.is_empty()) {
            return true;
        }
        if self.result.is_empty() {
            return true;
        }
        return false;
    }
}

impl LangFile {
    pub fn write<P>(&self, path: P) -> std::io::Result<()> where P: AsRef<Path> {
        let file = File::options().write(true).create(true).open(path)?;
        let mut writer = BufWriter::new(file);
        writeln!(&mut writer, r#"# Configuration file
enablelangfile {{
    B:UseThisFileAsLanguageFile=true
}}

languagefile {{"#)?;
        for (key, value) in self.items.iter() {
            writeln!(&mut writer, "    {}={}", key, value)?;
        }
        writeln!(&mut writer, "}}\n")?;

        Ok(())
    }

    pub fn read<P>(path: P) -> std::io::Result<LangFile> where P: AsRef<Path> {
        let mut items: Vec<LangItem> = vec![];

        let mut lang_started = false;

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for x in reader.lines() {
            let line = x?;
            if lang_started {
                if line.find('=').is_some() {
                    let line = line.trim();
                    let split: Vec<&str> = line.split("=").collect();
                    items.push((split[0].to_string(), split[1].to_string()));
                } else {
                    if line.find("}").is_some() {
                        lang_started = false;
                    }
                }

            } else {
                if line.find("languagefile").is_some() {
                    lang_started = true;
                }
            }
        }

        Ok(LangFile { items })
    }
}