use std::collections::HashMap;

pub struct JMaps {
    pub(crate) maps: serde_json::Map<String, serde_json::Value>,
}

impl JMaps {
    pub fn new() -> Self {
        Self {
            maps: serde_json::Map::new(),
        }
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

    pub fn get_strsr<T: AsRef<str>>(&self, key: T) -> String {
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
