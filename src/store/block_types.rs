use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockType {
    Text,
    Heading1,
    Heading2,
    Heading3,
    Todo,
    BulletedList,
    NumberedList,
    Toggle,
    Code,
    Quote,
    Divider,
    Callout,
    Image,
    Bookmark,
    Equation,
}

impl BlockType {
    pub fn to_db_string(&self) -> &'static str {
        match self {
            BlockType::Text => "text",
            BlockType::Heading1 => "heading1",
            BlockType::Heading2 => "heading2",
            BlockType::Heading3 => "heading3",
            BlockType::Todo => "todo",
            BlockType::BulletedList => "bulleted_list",
            BlockType::NumberedList => "numbered_list",
            BlockType::Toggle => "toggle",
            BlockType::Code => "code",
            BlockType::Quote => "quote",
            BlockType::Divider => "divider",
            BlockType::Callout => "callout",
            BlockType::Image => "image",
            BlockType::Bookmark => "bookmark",
            BlockType::Equation => "equation",
        }
    }

    pub fn from_db_string(s: &str) -> Self {
        match s {
            "heading1" => BlockType::Heading1,
            "heading2" => BlockType::Heading2,
            "heading3" => BlockType::Heading3,
            "todo" => BlockType::Todo,
            "bulleted_list" => BlockType::BulletedList,
            "numbered_list" => BlockType::NumberedList,
            "toggle" => BlockType::Toggle,
            "code" => BlockType::Code,
            "quote" => BlockType::Quote,
            "divider" => BlockType::Divider,
            "callout" => BlockType::Callout,
            "image" => BlockType::Image,
            "bookmark" => BlockType::Bookmark,
            "equation" => BlockType::Equation,
            _ => BlockType::Text,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            BlockType::Text => "Text",
            BlockType::Heading1 => "Heading 1",
            BlockType::Heading2 => "Heading 2",
            BlockType::Heading3 => "Heading 3",
            BlockType::Todo => "To-do",
            BlockType::BulletedList => "Bulleted List",
            BlockType::NumberedList => "Numbered List",
            BlockType::Toggle => "Toggle",
            BlockType::Code => "Code",
            BlockType::Quote => "Quote",
            BlockType::Divider => "Divider",
            BlockType::Callout => "Callout",
            BlockType::Image => "Image",
            BlockType::Bookmark => "Bookmark",
            BlockType::Equation => "Equation",
        }
    }
}
