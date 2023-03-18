use rustc_hash::FxHashSet;
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{cell::RefCell, fmt, mem::transmute, ops::Deref};

thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
}

#[derive(Debug, Default)]
struct Cache {
    strings: FxHashSet<Box<str>>,
}

impl Cache {
    fn get(&mut self, s: &str) -> &'static str {
        if let Some(ptr) = self.strings.get(s) {
            unsafe { transmute(&**ptr) }
        } else {
            let ptr: Box<str> = s.into();
            let ret: &'static str = unsafe { transmute(&*ptr) };
            self.strings.insert(ptr);
            ret
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InternedString(&'static str);

impl InternedString {
    pub fn new(s: &str) -> Self {
        Self(CACHE.with(|cache| cache.borrow_mut().get(s)))
    }
}

impl Deref for InternedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValueIntern {
    // Null,
    // Bool(bool),
    Number(i64), // int to skip eq problems
    String(InternedString),
    Array(Vec<ValueIntern>),
    Map(Vec<(ValueIntern, ValueIntern)>),
}

impl PartialEq<str> for ValueIntern {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::String(s) => &**s == other,
            _ => false,
        }
    }
}

impl<'de> Deserialize<'de> for ValueIntern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor {})
    }
}

struct ValueVisitor {}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = ValueIntern;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a json like value")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut out = vec![];
        while let Some((k, v)) = map.next_entry::<ValueIntern, ValueIntern>()? {
            out.push((k, v));
        }
        Ok(ValueIntern::Map(out))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut v = vec![];
        while let Some(e) = seq.next_element()? {
            v.push(e);
        }
        Ok(ValueIntern::Array(v))
    }

    fn visit_borrowed_str<E: serde::de::Error>(
        self,
        v: &'de str,
    ) -> std::result::Result<Self::Value, E> {
        Ok(ValueIntern::String(InternedString::new(v)))
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> std::result::Result<Self::Value, E> {
        Ok(ValueIntern::Number(v))
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> std::result::Result<Self::Value, E> {
        Ok(ValueIntern::Number(v as i64))
    }
}
