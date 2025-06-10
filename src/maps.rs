use std::{
    collections::HashMap,
    ops::{Deref, Index},
};

pub struct JMaps {
    pub(crate) maps: serde_json::Map<String, serde_json::Value>,
}
impl JMaps {
    pub fn new() -> Self {
        Self {
            maps: serde_json::Map::new(),
        }
    }

    pub fn reps(self) -> serde_json::Map<String, serde_json::Value> {
        self.maps
    }
    pub fn vmut(&mut self) -> &mut serde_json::Map<String, serde_json::Value> {
        &mut self.maps
    }

    pub fn get<T: AsRef<str>>(&self, key: T) -> Option<&serde_json::Value> {
        self.maps.get(key.as_ref())
    }
    pub fn get_mut<T: AsRef<str>>(&mut self, key: T) -> Option<&mut serde_json::Value> {
        self.maps.get_mut(key.as_ref())
    }
    pub fn insert<T: AsRef<str>>(&mut self, key: T, value: serde_json::Value) {
        self.maps.insert(key.as_ref().to_string(), value);
    }
    pub fn remove<T: AsRef<str>>(&mut self, key: T) -> Option<serde_json::Value> {
        self.maps.remove(key.as_ref())
    }

    pub fn get_str<T: AsRef<str>>(&self, key: T) -> Option<String> {
        match self.get(key) {
            Some(serde_json::Value::String(v)) => Some(v.clone()),
            Some(serde_json::Value::Bool(v)) => Some(format!("{}", v)),
            Some(serde_json::Value::Number(v)) => Some(format!("{}", v)),
            _ => None,
        }
    }

    pub fn get_strs<T: AsRef<str>>(&self, key: T) -> String {
        self.get_str(key).unwrap_or_default()
    }

    pub fn get_bool<T: AsRef<str>>(&self, key: T) -> bool {
        match self.get(key) {
            Some(serde_json::Value::String(v)) => v == "true",
            Some(serde_json::Value::Bool(v)) => v.clone(),
            Some(serde_json::Value::Number(v)) => v.eq(&serde_json::Number::from(1)),
            _ => false,
        }
    }

    pub fn get_i64<T: AsRef<str>>(&self, key: T) -> Option<i64> {
        match self.get(key) {
            Some(serde_json::Value::Number(v)) => v.as_i64(),
            Some(serde_json::Value::String(v)) => v.parse::<i64>().ok(),
            _ => None,
        }
    }
    pub fn get_i64s<T: AsRef<str>>(&self, key: T) -> i64 {
        self.get_i64(key).unwrap_or(0)
    }

    pub fn from_bts<T: AsRef<[u8]>>(bts: T) -> Result<Self, serde_json::Error> {
        let maps: serde_json::Map<String, serde_json::Value> =
            serde_json::from_slice(bts.as_ref())?;
        Ok(Self::from(maps))
    }
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.maps)
    }

    pub fn from_mapv(v: serde_json::Value) -> Option<Self> {
        if let serde_json::Value::Object(mp) = v {
            Some(Self::from(mp))
        } else {
            None
        }
    }
}

impl Deref for JMaps {
    type Target = serde_json::Map<String, serde_json::Value>;
    fn deref(&self) -> &Self::Target {
        &self.maps
    }
}
impl AsRef<serde_json::Map<String, serde_json::Value>> for JMaps {
    fn as_ref(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.maps
    }
}

impl From<serde_json::Map<String, serde_json::Value>> for JMaps {
    fn from(maps: serde_json::Map<String, serde_json::Value>) -> Self {
        Self { maps }
    }
}

impl From<HashMap<String, String>> for JMaps {
    fn from(maps: HashMap<String, String>) -> Self {
        Self::from(&maps)
    }
}
impl From<&HashMap<String, String>> for JMaps {
    fn from(maps: &HashMap<String, String>) -> Self {
        let mut e = Self::new();
        for (k, v) in maps {
            e.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        e
    }
}

/* impl From<&HashMap<String, serde_json::Value>> for JMaps {
    fn from(maps: &HashMap<String, serde_json::Value>) -> Self {
        Self::from(&maps)
    }
} */
impl From<HashMap<String, serde_json::Value>> for JMaps {
    fn from(maps: HashMap<String, serde_json::Value>) -> Self {
        let mut e = Self::new();
        for (k, v) in maps {
            e.insert(k, v);
        }
        e
    }
}
/* impl From<&[u8]> for JMaps {
    fn from(maps: &[u8]) -> Self {
        Self::from_bts(maps).unwrap()
    }
} */

pub struct ArraJMaps {
    ls: Vec<serde_json::Map<String, serde_json::Value>>,
}

impl ArraJMaps {
    pub fn new() -> Self {
        Self { ls: vec![] }
    }
    pub fn from_bts<T: AsRef<[u8]>>(bts: T) -> Result<Self, serde_json::Error> {
        let ls: Vec<serde_json::Map<String, serde_json::Value>> =
            serde_json::from_slice(bts.as_ref())?;
        Ok(Self::from(ls))
    }
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.ls)
    }
}
impl From<Vec<serde_json::Map<String, serde_json::Value>>> for ArraJMaps {
    fn from(ls: Vec<serde_json::Map<String, serde_json::Value>>) -> Self {
        Self { ls }
    }
}

impl Deref for ArraJMaps {
    type Target = Vec<serde_json::Map<String, serde_json::Value>>;
    fn deref(&self) -> &Self::Target {
        &self.ls
    }
}

pub struct ArraJMapsIter {
    inner: std::vec::IntoIter<serde_json::Map<String, serde_json::Value>>,
}

impl Iterator for ArraJMapsIter {
    type Item = JMaps;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|maps| JMaps { maps })
    }
}
impl IntoIterator for ArraJMaps {
    type Item = JMaps;
    type IntoIter = ArraJMapsIter;

    fn into_iter(self) -> Self::IntoIter {
        ArraJMapsIter {
            inner: self.ls.into_iter(),
        }
    }
}
