use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageCategory {
    Landscape,
    Person,
    Object,
    Animal,
    Architecture,
    Document,
    Other,
}

impl FromStr for ImageCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "风景" | "landscape" => Self::Landscape,
            "人物" | "person" => Self::Person,
            "物品" | "object" => Self::Object,
            "动物" | "animal" => Self::Animal,
            "建筑" | "architecture" | "building" => Self::Architecture,
            "文档" | "document" => Self::Document,
            _ => Self::Other,
        })
    }
}

impl ImageCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Landscape => "风景",
            Self::Person => "人物",
            Self::Object => "物品",
            Self::Animal => "动物",
            Self::Architecture => "建筑",
            Self::Document => "文档",
            Self::Other => "其他",
        }
    }
}
