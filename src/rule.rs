use serde::{Serialize, Deserialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "s")]
    pub source: String,

    #[serde(rename = "t")]
    pub target: String,

    pub subs: Vec<String>
}
