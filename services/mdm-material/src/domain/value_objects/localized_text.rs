//! 多语言文本值对象

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 多语言文本
///
/// 支持多语言的文本值对象，包含默认文本和各语言的翻译
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LocalizedText {
    /// 默认文本
    default_text: String,
    /// 翻译映射 (locale -> text)
    translations: HashMap<String, String>,
}

impl LocalizedText {
    /// 创建新的多语言文本
    pub fn new(default_text: impl Into<String>) -> Self {
        Self {
            default_text: default_text.into(),
            translations: HashMap::new(),
        }
    }

    /// 添加翻译
    pub fn with_translation(mut self, locale: impl Into<String>, text: impl Into<String>) -> Self {
        self.translations.insert(locale.into(), text.into());
        self
    }

    /// 从翻译映射创建
    pub fn from_translations(
        default_text: impl Into<String>,
        translations: HashMap<String, String>,
    ) -> Self {
        Self {
            default_text: default_text.into(),
            translations,
        }
    }

    /// 获取默认文本
    pub fn default_text(&self) -> &str {
        &self.default_text
    }

    /// 获取指定语言的文本，如果不存在则返回默认文本
    pub fn get(&self, locale: &str) -> &str {
        self.translations
            .get(locale)
            .map(|s| s.as_str())
            .unwrap_or(&self.default_text)
    }

    /// 获取指定语言的文本（可选）
    pub fn get_translation(&self, locale: &str) -> Option<&str> {
        self.translations.get(locale).map(|s| s.as_str())
    }

    /// 设置翻译
    pub fn set_translation(&mut self, locale: impl Into<String>, text: impl Into<String>) {
        self.translations.insert(locale.into(), text.into());
    }

    /// 移除翻译
    pub fn remove_translation(&mut self, locale: &str) -> Option<String> {
        self.translations.remove(locale)
    }

    /// 获取所有翻译
    pub fn translations(&self) -> &HashMap<String, String> {
        &self.translations
    }

    /// 获取所有支持的语言
    pub fn locales(&self) -> impl Iterator<Item = &str> {
        self.translations.keys().map(|s| s.as_str())
    }

    /// 检查是否有指定语言的翻译
    pub fn has_translation(&self, locale: &str) -> bool {
        self.translations.contains_key(locale)
    }

    /// 更新默认文本
    pub fn set_default_text(&mut self, text: impl Into<String>) {
        self.default_text = text.into();
    }
}

impl From<String> for LocalizedText {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<&str> for LocalizedText {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl std::fmt::Display for LocalizedText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.default_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let text = LocalizedText::new("Hello");
        assert_eq!(text.default_text(), "Hello");
    }

    #[test]
    fn test_with_translation() {
        let text = LocalizedText::new("Hello")
            .with_translation("zh_CN", "你好")
            .with_translation("ja_JP", "こんにちは");

        assert_eq!(text.get("zh_CN"), "你好");
        assert_eq!(text.get("ja_JP"), "こんにちは");
        assert_eq!(text.get("fr_FR"), "Hello"); // 回退到默认
    }

    #[test]
    fn test_set_translation() {
        let mut text = LocalizedText::new("Hello");
        text.set_translation("zh_CN", "你好");
        assert_eq!(text.get("zh_CN"), "你好");
    }
}
