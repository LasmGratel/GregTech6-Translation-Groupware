use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::AddAssign;
use std::str::FromStr;
use either::Either;
use crate::rule::Rule;
use serde::{Serialize, Deserialize, Deserializer, de};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct GeneratorMeta {
    pub group: String,

    #[serde(default)]
    pub namespace: String,

    #[serde(default)]
    pub completed: bool,

    #[serde(default)]
    pub extensions: HashSet<String>,

    #[serde(default)]
    pub dict: Option<HashMap<String, StringOrHashMap>>,

    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct StringOrHashMap {
    #[serde(with = "either::serde_untagged")]
    pub inner: Either<String, HashMap<String, String>>
}

impl GeneratorMeta {
    pub fn with_namespace(&self, namespace: String) -> Self {
        let mut new = self.clone();
        new.namespace = namespace;
        new
    }

    pub fn is_empty(&self) -> bool {
        self.group.is_empty()
    }

    pub fn combine(&mut self, rhs: &Self) {
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
        rhs.extensions.iter().all(|x| self.extensions.insert(x.clone()));
    }
}

impl Hash for GeneratorMeta {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.group.hash(state);
        self.namespace.hash(state);
    }
}