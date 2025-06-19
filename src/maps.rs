use std::{
    collections::HashMap,
    ops::{Deref, DerefMut, Index},
};

pub struct JMapErr {
    fs: i32,
}
impl JMapErr {
    pub fn nomut() -> Self {
        Self { fs: 1 }
    }
    pub(crate) fn nomuts<T>() -> Result<T, Self> {
        Err(Self { fs: 1 })
    }
}
pub enum JMaps<'a> {
    Own(serde_json::Map<String, serde_json::Value>),
    Brow(&'a serde_json::Map<String, serde_json::Value>),
    BrowMut(&'a mut serde_json::Map<String, serde_json::Value>),
}
impl<'a> JMaps<'a> {
    pub fn new() -> Self {
        Self::Own(serde_json::Map::new())
    }

    pub fn reps(self) -> serde_json::Map<String, serde_json::Value> {
        let mut rts = serde_json::Map::new();
        match self {
            JMaps::Own(maps) => {
                return maps;
            }
            JMaps::Brow(maps) => {
                for (k, v) in maps {
                    rts.insert(k.clone(), v.clone());
                }
            }
            JMaps::BrowMut(maps) => {
                for (k, v) in maps {
                    rts.insert(k.clone(), v.clone());
                }
            }
        }
        rts
    }
    pub fn repv(self) -> serde_json::Value {
        serde_json::Value::Object(self.reps())
    }
    pub fn vmut(&mut self) -> Result<&mut serde_json::Map<String, serde_json::Value>, JMapErr> {
        match self {
            JMaps::Own(maps) => Ok(maps),
            JMaps::Brow(_) => JMapErr::nomuts(),
            JMaps::BrowMut(maps) => Ok(*maps),
        }
    }

    pub fn get<T: AsRef<str>>(&self, key: T) -> Option<&serde_json::Value> {
        match self {
            JMaps::Own(maps) => maps.get(key.as_ref()),
            JMaps::Brow(maps) => maps.get(key.as_ref()),
            JMaps::BrowMut(maps) => maps.get(key.as_ref()),
        }
    }
    pub fn get_mut<T: AsRef<str>>(&mut self, key: T) -> Option<&mut serde_json::Value> {
        match self {
            JMaps::Own(maps) => maps.get_mut(key.as_ref()),
            JMaps::Brow(_) => None,
            JMaps::BrowMut(maps) => maps.get_mut(key.as_ref()),
        }
    }
    pub fn insert<T: AsRef<str>>(&mut self, key: T, value: serde_json::Value) {
        match self {
            JMaps::Own(maps) => {
                maps.insert(key.as_ref().to_string(), value);
            }
            JMaps::Brow(_) => {}
            JMaps::BrowMut(maps) => {
                maps.insert(key.as_ref().to_string(), value);
            }
        }
    }
    pub fn remove<T: AsRef<str>>(&mut self, key: T) -> Option<serde_json::Value> {
        match self {
            JMaps::Own(maps) => maps.remove(key.as_ref()),
            JMaps::Brow(_) => None,
            JMaps::BrowMut(maps) => maps.remove(key.as_ref()),
        }
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
    pub fn get_map<S: AsRef<str>>(&'a self, key: S) -> Option<JMaps<'a>> {
        match self.get(key) {
            Some(v) => Self::from_mapv(v),
            _ => None,
        }
    }
    pub fn get_map_mut<S: AsRef<str>>(&'a mut self, key: S) -> Option<JMaps<'a>> {
        match self.get_mut(key) {
            Some(v) => Self::from_mapv_mut(v),
            _ => None,
        }
    }

    pub fn from_bts<T: AsRef<[u8]>>(bts: T) -> Result<Self, serde_json::Error> {
        let maps: serde_json::Map<String, serde_json::Value> =
            serde_json::from_slice(bts.as_ref())?;
        Ok(Self::from(maps))
    }
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        match self {
            JMaps::Own(maps) => serde_json::to_string(&maps),
            JMaps::Brow(maps) => serde_json::to_string(&maps),
            JMaps::BrowMut(maps) => serde_json::to_string(*maps),
        }
    }

    pub fn from_mapo(v: serde_json::Value) -> Option<Self> {
        if let serde_json::Value::Object(mp) = v {
            Some(Self::Own(mp))
        } else {
            None
        }
    }
    pub fn from_mapv(v: &'a serde_json::Value) -> Option<Self> {
        if let serde_json::Value::Object(mp) = v {
            Some(Self::Brow(mp))
        } else {
            None
        }
    }
    pub fn from_mapv_mut(v: &'a mut serde_json::Value) -> Option<Self> {
        if let serde_json::Value::Object(mp) = v {
            Some(Self::BrowMut(mp))
        } else {
            None
        }
    }
}

impl<'a> Deref for JMaps<'a> {
    type Target = serde_json::Map<String, serde_json::Value>;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
/* impl DerefMut for JMaps {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.maps
    }
} */
impl<'a> AsRef<serde_json::Map<String, serde_json::Value>> for JMaps<'a> {
    fn as_ref(&self) -> &serde_json::Map<String, serde_json::Value> {
        match self {
            JMaps::Own(maps) => maps,
            JMaps::Brow(maps) => *maps,
            JMaps::BrowMut(maps) => *maps,
        }
    }
}

impl<'a> From<serde_json::Map<String, serde_json::Value>> for JMaps<'a> {
    fn from(maps: serde_json::Map<String, serde_json::Value>) -> Self {
        Self::Own(maps)
    }
}
impl<'a> From<&'a serde_json::Map<String, serde_json::Value>> for JMaps<'a> {
    fn from(maps: &'a serde_json::Map<String, serde_json::Value>) -> Self {
        Self::Brow(maps)
    }
}
impl<'a> From<&'a mut serde_json::Map<String, serde_json::Value>> for JMaps<'a> {
    fn from(maps: &'a mut serde_json::Map<String, serde_json::Value>) -> Self {
        Self::BrowMut(maps)
    }
}

impl<'a> From<HashMap<String, String>> for JMaps<'a> {
    fn from(maps: HashMap<String, String>) -> Self {
        Self::from(&maps)
    }
}
impl<'a> From<&HashMap<String, String>> for JMaps<'a> {
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
/* impl<'a> From<HashMap<String, serde_json::Value>> for JMaps<'a> {
    fn from(maps: HashMap<String, serde_json::Value>) -> Self {
        let mut e = Self::new();
        for (k, v) in maps {
            e.insert(k, v);
        }
        e
    }
} */
/* impl From<&[u8]> for JMaps {
    fn from(maps: &[u8]) -> Self {
        Self::from_bts(maps).unwrap()
    }
} */

pub struct ArraJMaps {
    ls: Vec<serde_json::Value>,
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
impl From<Vec<serde_json::Value>> for ArraJMaps {
    fn from(ls: Vec<serde_json::Value>) -> Self {
        Self { ls }
    }
}
impl From<Vec<serde_json::Map<String, serde_json::Value>>> for ArraJMaps {
    fn from(ls: Vec<serde_json::Map<String, serde_json::Value>>) -> Self {
        let mut rts = Vec::with_capacity(ls.len());
        for v in ls {
            rts.push(serde_json::Value::Object(v));
        }
        Self::from(rts)
    }
}

impl Deref for ArraJMaps {
    type Target = Vec<serde_json::Value>;
    fn deref(&self) -> &Self::Target {
        &self.ls
    }
}
impl DerefMut for ArraJMaps {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ls
    }
}

pub struct ArraJMapsIter<'a> {
    idx: usize,
    ls: &'a Vec<serde_json::Value>,
}

impl<'a> Iterator for ArraJMapsIter<'a> {
    type Item = JMaps<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.ls.len() {
            let it = self.ls.get(self.idx)?;
            self.idx += 1;
            JMaps::from_mapv(it)
        } else {
            None
        }
    }
}
pub struct ArraJMapsIntoIter {
    ls: std::vec::IntoIter<serde_json::Value>,
}

impl Iterator for ArraJMapsIntoIter {
    type Item = JMaps<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        let it = self.ls.next()?;
        JMaps::from_mapo(it)
    }
}
impl IntoIterator for ArraJMaps {
    type Item = JMaps<'static>;
    type IntoIter = ArraJMapsIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        ArraJMapsIntoIter {
            ls: self.ls.into_iter(),
        }
    }
}
impl<'a> IntoIterator for &'a ArraJMaps {
    type Item = JMaps<'a>;
    type IntoIter = ArraJMapsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ArraJMapsIter {
            ls: &self.ls,
            idx: 0,
        }
    }
}
