//! Image category classification enum and string conversion.
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Categorizes an image into one of seven types based on AI analysis: Landscape, Person, Object, Animal, Architecture, Document, or Other.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageCategory {
    /// Natural scenery or outdoor landscape photo.
    Landscape,
    /// Portrait or group photo with people.
    Person,
    /// Photo of an inanimate object or product.
    Object,
    /// Photo of an animal or pet.
    Animal,
    /// Photo of a building or architectural structure.
    Architecture,
    /// Photo of a document, screenshot, or text-heavy image.
    Document,
    /// Any image that does not fit into the other categories.
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

/// ImageCategory utility methods.
impl ImageCategory {
    /// Returns the Chinese display name for this category variant.
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
