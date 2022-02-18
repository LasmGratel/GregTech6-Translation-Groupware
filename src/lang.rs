use crate::meta::GeneratorMeta;

type LangItem = (String, String);

pub struct LangFile {
    items: Vec<LangItem>
}

pub trait LangResult {
    fn meta(&self) -> &GeneratorMeta;
    fn result(&self) -> Vec<LangItem>;
}