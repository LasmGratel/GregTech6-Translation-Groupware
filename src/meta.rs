use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Write;
use std::marker::PhantomData;
use std::ops::AddAssign;
use std::str::FromStr;
use either::Either;
use crate::rule::Rule;
use serde::{Serialize, Deserialize, Deserializer, de};
use serde::de::{MapAccess, Visitor};

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratorMeta {
    pub group: String,

    #[serde(default)]
    pub namespace: String,

    #[serde(default)]
    pub completed: bool,

    #[serde(default)]
    pub extensions: HashSet<String>,

    #[serde(default)]
    pub dict: HashMap<String, StringOrHashMap>,

    #[serde(default)]
    pub rules: Vec<Rule>,

    #[serde(skip)]
    pub cached: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct StringOrHashMap {
    #[serde(with = "either::serde_untagged")]
    inner: Either<String, HashMap<String, String>>
}

impl GeneratorMeta {
    pub fn is_empty(&self) -> bool {
        self.group.is_empty()
    }
}

impl AddAssign for GeneratorMeta {
    fn add_assign(&mut self, rhs: Self) {
        let mut this_ns = &self.namespace;
        let that_ns = &rhs.namespace;
        if this_ns.rfind(that_ns) == Some(0) {
            // this_ns is started with that_ns
            // unchanged
        } else if that_ns.rfind(this_ns) == Some(0) {
            // that_ns is started with this_ns
            this_ns = that_ns;
        } else {
            // not valid
            self.group.clear();
            return;
        }
        rhs.extensions.into_iter().all(|x| self.extensions.insert(x));
    }
}