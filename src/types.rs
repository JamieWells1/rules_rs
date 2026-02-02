// Shared domain types

use std::collections::HashMap;

// Aliases

// Tag name -- e.g. "colour"
pub type TagName = String;

// Tag values -- e.g. ["red", "green"]
pub type TagValues = Vec<String>;

// Number of subrule -- e.g. 42
pub type SubRuleNumber = i32;

// Object structure -- e.g. "colour": ["green"]
pub type Object = HashMap<String, Vec<String>>;

// Clause in subrule -- e.g. "colour": "green"
pub type TagKvMap = HashMap<String, String>;

// Structs

pub struct Tag {
    pub name: TagName,
    pub values: TagValues,
}

pub enum ComparisonOp {
    ISEQ,
    NOEQ,
    // To be supported in future:
    // GREQ,
    // LEEQ,
}

pub enum LogicalOp {
    AND,
    OR,
}

pub struct SubRule {
    pub expected_count: i32,
    pub actual_count: i32,
    // No. elements (tag_kvs) should be == no. elements (comparison_op - 1)
    pub comparison_ops: Vec<ComparisonOp>,
    pub tag_kvs: TagKvMap,
}

// Impls

impl Default for SubRule {
    fn default() -> Self {
        SubRule {
            expected_count: 2,
            actual_count: 0,
            comparison_ops: Vec::new(),
            tag_kvs: HashMap::new(),
        }
    }
}
