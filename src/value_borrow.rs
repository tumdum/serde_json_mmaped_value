use std::fmt;

use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueBorrow<'a> {
    // Null,
    // Bool(bool),
    Number(i64), // int to skip eq problems
    String(&'a str),
    Array(Vec<ValueBorrow<'a>>),
    Map(Vec<(ValueBorrow<'a>, ValueBorrow<'a>)>),
}

impl<'a> PartialEq<str> for ValueBorrow<'a> {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::String(s) => s == &other,
            _ => false,
        }
    }
}

impl<'de> Deserialize<'de> for ValueBorrow<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor {})
    }
}

struct ValueVisitor {}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = ValueBorrow<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a json like value")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut out = vec![];
        while let Some((k, v)) = map.next_entry::<ValueBorrow, ValueBorrow>()? {
            out.push((k, v));
        }
        Ok(ValueBorrow::Map(out))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut v = vec![];
        while let Some(e) = seq.next_element()? {
            v.push(e);
        }
        Ok(ValueBorrow::Array(v))
    }

    fn visit_borrowed_str<E: serde::de::Error>(
        self,
        v: &'de str,
    ) -> std::result::Result<Self::Value, E> {
        Ok(ValueBorrow::String(v))
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> std::result::Result<Self::Value, E> {
        Ok(ValueBorrow::Number(v))
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> std::result::Result<Self::Value, E> {
        Ok(ValueBorrow::Number(v as i64))
    }
}
