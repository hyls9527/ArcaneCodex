#![allow(missing_docs)]
use crate::models::ImageCategory;

pub struct ConsistencyChecker;

impl ConsistencyChecker {
    pub fn check_category_vs_tags(category: &ImageCategory, tags: &[String]) -> Vec<String> {
        let mut conflicts = Vec::new();

        if tags.is_empty() {
            return conflicts;
        }

        let category_tags = Self::expected_tags_for_category(category);

        let unexpected_tags: Vec<&String> = tags
            .iter()
            .filter(|tag| {
                category_tags.iter().all(|expected| !tag.contains(expected))
                    && !Self::is_generic_tag(tag)
            })
            .collect();

        if !unexpected_tags.is_empty() {
            let tag_str = unexpected_tags
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            conflicts.push(format!(
                "分类({}) 与标签不一致: {}",
                category.as_str(),
                tag_str
            ));
        }

        conflicts
    }

    pub fn check_category_vs_description(
        category: &ImageCategory,
        description: &str,
    ) -> Vec<String> {
        let mut conflicts = Vec::new();

        if description.trim().is_empty() {
            return conflicts;
        }

        let desc_lower = description.to_lowercase();

        let category_indicators = Self::category_description_indicators(category);
        let opposite_indicators = Self::opposite_category_indicators(category);

        let has_category_signal = category_indicators
            .iter()
            .any(|keyword| desc_lower.contains(keyword));
        let has_opposite_signal = opposite_indicators
            .iter()
            .any(|keyword| desc_lower.contains(keyword));

        if !has_category_signal && has_opposite_signal {
            let opposite_str = opposite_indicators.join(" / ");
            conflicts.push(format!(
                "分类({}) 与描述内容矛盾。描述中包含 {} 相关关键词",
                category.as_str(),
                opposite_str
            ));
        }

        conflicts
    }

    pub fn check_tags_vs_description(tags: &[String], description: &str) -> Vec<String> {
        let mut conflicts = Vec::new();

        if tags.is_empty() || description.trim().is_empty() {
            return conflicts;
        }

        let desc_lower = description.to_lowercase();

        for tag in tags {
            let tag_indicators = Self::tag_description_indicators(tag);
            if !tag_indicators.is_empty() {
                let has_signal = tag_indicators
                    .iter()
                    .any(|keyword| desc_lower.contains(keyword));
                if !has_signal {
                    conflicts.push(format!("标签({}) 在描述中未找到支持内容", tag));
                }
            }
        }

        conflicts
    }

    pub fn check_all(category: &ImageCategory, tags: &[String], description: &str) -> Vec<String> {
        let mut all_conflicts = Vec::new();

        all_conflicts.extend(Self::check_category_vs_tags(category, tags));
        all_conflicts.extend(Self::check_category_vs_description(category, description));
        all_conflicts.extend(Self::check_tags_vs_description(tags, description));

        all_conflicts
    }

    pub fn has_conflicts(conflicts: &[String]) -> bool {
        !conflicts.is_empty()
    }

    fn expected_tags_for_category(category: &ImageCategory) -> Vec<&'static str> {
        match category {
            ImageCategory::Landscape => vec![
                "山", "水", "自然", "风景", "天空", "树", "海", "日落", "日出",
            ],
            ImageCategory::Person => vec!["人", "脸", "笑容", "群体", "儿童", "老人"],
            ImageCategory::Object => vec!["物品", "产品", "静物", "桌子", "椅子", "工具"],
            ImageCategory::Animal => vec!["狗", "猫", "鸟", "鱼", "马", "动物", "宠物"],
            ImageCategory::Architecture => vec!["建筑", "楼房", "桥梁", "塔", "室内", "城市"],
            ImageCategory::Document => vec!["文字", "截图", "文档", "表格", "扫描"],
            ImageCategory::Other => vec![],
        }
    }

    fn is_generic_tag(tag: &str) -> bool {
        matches!(
            tag,
            "彩色" | "黑白" | "清晰" | "模糊" | "高清" | "夜景" | "白天" | "室内" | "室外"
        )
    }

    fn category_description_indicators(category: &ImageCategory) -> Vec<&'static str> {
        match category {
            ImageCategory::Landscape => {
                vec!["风景", "自然", "山", "水", "天空", "树木", "草地", "湖"]
            }
            ImageCategory::Person => vec!["人", "脸", "笑容", "肖像", "人物", "自拍"],
            ImageCategory::Object => vec!["物品", "产品", "静物", "物体", "工具"],
            ImageCategory::Animal => vec!["动物", "狗", "猫", "鸟", "鱼", "宠物", "野生"],
            ImageCategory::Architecture => vec!["建筑", "楼房", "桥梁", "塔", "大厦", "教堂"],
            ImageCategory::Document => vec!["文字", "文档", "截图", "扫描", "表格", "书"],
            ImageCategory::Other => vec![],
        }
    }

    fn opposite_category_indicators(category: &ImageCategory) -> Vec<&'static str> {
        match category {
            ImageCategory::Landscape => vec!["人", "建筑", "动物", "文档"],
            ImageCategory::Person => vec!["风景", "动物", "建筑", "文档"],
            ImageCategory::Object => vec!["风景", "人物", "动物", "建筑", "文档"],
            ImageCategory::Animal => vec!["风景", "人物", "建筑", "文档"],
            ImageCategory::Architecture => vec!["风景", "人物", "动物", "文档"],
            ImageCategory::Document => vec!["风景", "人物", "动物", "建筑"],
            ImageCategory::Other => vec![],
        }
    }

    fn tag_description_indicators(tag: &str) -> Vec<&'static str> {
        match tag {
            t if t.contains("狗") => vec!["狗", "宠物", "毛"],
            t if t.contains("猫") => vec!["猫", "宠物", "胡须"],
            t if t.contains("山") => vec!["山", "峰", "登山", "山顶"],
            t if t.contains("海") => vec!["海", "水", "波浪", "沙滩"],
            t if t.contains("建筑") => vec!["建筑", "楼房", "大厦"],
            t if t.contains("人") => vec!["人", "脸", "笑容"],
            t if t.contains("文") => vec!["文字", "文档", "截图"],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_category_vs_tags_consistent() {
        let category = ImageCategory::Animal;
        let tags = vec!["狗".to_string(), "宠物".to_string()];
        let conflicts = ConsistencyChecker::check_category_vs_tags(&category, &tags);
        assert!(
            conflicts.is_empty(),
            "Animal category with dog/pet tags should be consistent"
        );
    }

    #[test]
    fn test_check_category_vs_tags_inconsistent() {
        let category = ImageCategory::Animal;
        let tags = vec!["建筑".to_string(), "桥梁".to_string()];
        let conflicts = ConsistencyChecker::check_category_vs_tags(&category, &tags);
        assert!(
            !conflicts.is_empty(),
            "Animal category with architecture tags should conflict"
        );
    }

    #[test]
    fn test_check_category_vs_tags_empty_tags() {
        let category = ImageCategory::Landscape;
        let tags: Vec<String> = vec![];
        let conflicts = ConsistencyChecker::check_category_vs_tags(&category, &tags);
        assert!(conflicts.is_empty(), "Empty tags should have no conflicts");
    }

    #[test]
    fn test_check_category_vs_tags_generic_tags_allowed() {
        let category = ImageCategory::Animal;
        let tags = vec!["狗".to_string(), "彩色".to_string(), "高清".to_string()];
        let conflicts = ConsistencyChecker::check_category_vs_tags(&category, &tags);
        assert!(
            conflicts.is_empty(),
            "Generic tags should not cause conflicts"
        );
    }

    #[test]
    fn test_check_category_vs_description_consistent() {
        let category = ImageCategory::Person;
        let description = "这张图片显示了一个微笑的人的脸部特写";
        let conflicts = ConsistencyChecker::check_category_vs_description(&category, description);
        assert!(
            conflicts.is_empty(),
            "Person category with person description should be consistent"
        );
    }

    #[test]
    fn test_check_category_vs_description_contradictory() {
        let category = ImageCategory::Person;
        let description = "这是一座高大的桥梁建筑，横跨河流";
        let conflicts = ConsistencyChecker::check_category_vs_description(&category, description);
        assert!(
            !conflicts.is_empty(),
            "Person category with architecture description should conflict"
        );
    }

    #[test]
    fn test_check_category_vs_description_empty() {
        let category = ImageCategory::Landscape;
        let description = "";
        let conflicts = ConsistencyChecker::check_category_vs_description(&category, description);
        assert!(
            conflicts.is_empty(),
            "Empty description should have no conflicts"
        );
    }

    #[test]
    fn test_check_category_vs_description_no_signal() {
        let category = ImageCategory::Landscape;
        let description = "这是一张照片";
        let conflicts = ConsistencyChecker::check_category_vs_description(&category, description);
        assert!(
            conflicts.is_empty(),
            "Neutral description with no category signal should not conflict"
        );
    }

    #[test]
    fn test_check_tags_vs_description_consistent() {
        let tags = vec!["狗".to_string(), "宠物".to_string()];
        let description = "一只可爱的宠物狗在草地上玩耍";
        let conflicts = ConsistencyChecker::check_tags_vs_description(&tags, description);
        assert!(
            conflicts.is_empty(),
            "Tags matching description should be consistent"
        );
    }

    #[test]
    fn test_check_tags_vs_description_inconsistent() {
        let tags = vec!["狗".to_string()];
        let description = "这是一座高楼大厦，矗立在城市中心";
        let conflicts = ConsistencyChecker::check_tags_vs_description(&tags, description);
        assert!(
            !conflicts.is_empty(),
            "Dog tag with building description should be inconsistent"
        );
    }

    #[test]
    fn test_check_all_no_conflicts() {
        let category = ImageCategory::Animal;
        let tags = vec!["狗".to_string(), "宠物".to_string()];
        let description = "一只可爱的小狗，毛茸茸的宠物";
        let conflicts = ConsistencyChecker::check_all(&category, &tags, description);
        assert!(
            conflicts.is_empty(),
            "Fully consistent input should have no conflicts"
        );
    }

    #[test]
    fn test_check_all_with_conflicts() {
        let category = ImageCategory::Animal;
        let tags = vec!["建筑".to_string()];
        let description = "一座高大的桥梁建筑";
        let conflicts = ConsistencyChecker::check_all(&category, &tags, description);
        assert!(
            !conflicts.is_empty(),
            "Inconsistent input should have conflicts"
        );
    }

    #[test]
    fn test_has_conflicts_empty() {
        assert!(!ConsistencyChecker::has_conflicts(&[]));
    }

    #[test]
    fn test_has_conflicts_present() {
        assert!(ConsistencyChecker::has_conflicts(&[
            "conflict 1".to_string()
        ]));
    }
}
