use crate::{value_borrow::ValueBorrow, value_intern::ValueIntern};

pub trait Queryable {
    fn get_all<'a>(&'a self, key: &'static str) -> Vec<&'a Self>;
    fn contains(&self, arg: &str) -> bool;
}

impl<'a> Queryable for ValueBorrow<'a> {
    fn get_all(&self, key: &'static str) -> Vec<&ValueBorrow<'a>> {
        match self {
            Self::Map(v) => v.iter().filter(|(k, _)| k == key).map(|(_, v)| v).collect(),
            Self::Array(v) => v.iter().flat_map(|e| e.get_all(key)).collect(),
            _ => vec![],
        }
    }

    fn contains(&self, arg: &str) -> bool {
        match self {
            Self::String(s) => s.contains(arg),
            _ => false,
        }
    }
}

impl Queryable for ValueIntern {
    fn get_all(&self, key: &'static str) -> Vec<&ValueIntern> {
        match self {
            Self::Map(v) => v.iter().filter(|(k, _)| k == key).map(|(_, v)| v).collect(),
            Self::Array(v) => v.iter().flat_map(|e| e.get_all(key)).collect(),
            _ => vec![],
        }
    }

    fn contains(&self, arg: &str) -> bool {
        match self {
            Self::String(s) => s.contains(arg),
            _ => false,
        }
    }
}

impl Queryable for serde_json::Value {
    fn get_all(&self, key: &'static str) -> Vec<&serde_json::Value> {
        match self {
            serde_json::Value::Object(v) => v
                .iter()
                .filter(|(k, _)| **k == key)
                .map(|(_, v)| v)
                .collect(),
            serde_json::Value::Array(v) => v.iter().flat_map(|e| e.get_all(key)).collect(),
            _ => vec![],
        }
    }
    fn contains(&self, arg: &str) -> bool {
        match self {
            serde_json::Value::String(v) => v.contains(arg),
            _ => false,
        }
    }
}
