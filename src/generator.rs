use std::borrow::{BorrowMut, Cow};
use std::hash::Hash;
use std::rc::Rc;
use crate::lang::{DictLangResult, LangItem, LangResult, RuleLangResult};
use crate::meta::GeneratorMeta;
use crate::replacer::GroupRepository;
use crate::rule::Rule;

pub enum GeneratorType<'a> {
    Dict(DictGenerator<'a>),
    Rule(RuleGenerator<'a>),
}

pub trait Generator<'a> {
    fn meta(&self) -> &'a GeneratorMeta;
    fn results(&self, repo: &Box<dyn GroupRepository<'a>>) -> Vec<Box<dyn LangResult<'a>>>;
}

#[derive(PartialEq, Eq, Hash)]
pub struct DictGenerator<'a> {
    pub meta: &'a GeneratorMeta,
    pub dict: Vec<LangItem>
}

impl<'a> Generator<'a> for DictGenerator<'a> {
    fn meta(&self) -> &'a GeneratorMeta {
        self.meta
    }

    fn results(&self, repo: &Box<dyn GroupRepository<'a>>) -> Vec<Box<dyn LangResult<'a>>> {
        let result = DictLangResult {
            meta: self.meta,
            result: self.dict.clone()
        };
        vec![Box::new(result)]
    }
}

pub struct RuleGenerator<'a> {
    pub meta: &'a GeneratorMeta,
    pub(crate) rules: &'a Vec<Rule>
}

impl<'a, 'b: 'a> Generator<'a> for RuleGenerator<'a> {
    fn meta(&self) -> &'a GeneratorMeta {
        self.meta
    }

    fn results(&self, repo: &Box<dyn GroupRepository<'a>>) -> Vec<Box<dyn LangResult<'a>>> {
        let mut results: Vec<Box<dyn LangResult<'a>>> = vec![];
        for rule in self.rules.iter() {
            let subs = &rule.subs;
            let subs_size = subs.len();
            if subs_size == 0 {
                continue;
            }
            // init
            let mut sub_results = vec![];
            for sub in subs {
                let mut sub_result = repo.get_group_results(sub).clone();
                sub_result.retain(|x| !x.is_empty());
                if sub_result.is_empty() {
                    drop(sub_result);
                    break;
                }
                sub_results.push(sub_result);
            }

            if sub_results.is_empty() {
                continue; // init failed, see clear() above
            }

            let mut begins = vec![];

            loop {
                // fill
                for i in begins.len()..subs_size {
                    begins.push(sub_results[i].clone());
                }

                let mut lang_combination = vec![];
                for begin in begins.iter() {
                    lang_combination.push(begin);
                }
                let mut generated_result = RuleLangResult {
                    rule,
                    meta: Cow::Borrowed(self.meta),
                    subs: lang_combination.into_iter().flatten().map(|x| (*x).clone()).collect(),
                    result: vec![]
                };
                if let Some(generated_meta) = generated_result.meta_combined() {
                    generated_result.meta = Cow::Owned(generated_meta);
                    results.push(Box::new(generated_result));
                }

                // step
                for i in (0..(subs_size + 1)).rev() {

                    if &begins[i - 1][0].result() == &sub_results[i - 1][0].result() {
                        begins.pop();
                    } else {
                        break;
                    }
                }

                if begins.is_empty() {
                    break;
                }
            }
        }
        results
    }
}