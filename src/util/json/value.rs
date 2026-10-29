use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64, String),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(m) => m.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn at(&self, path: &[&str]) -> Option<&Value> {
        let mut v = self;
        for k in path {
            v = v.get(k)?;
        }
        Some(v)
    }

    pub fn str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn num(&self) -> Option<f64> {
        match self {
            Value::Num(n, _) => Some(*n),
            _ => None,
        }
    }

    pub fn bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn arr(&self) -> Option<&[Value]> {
        match self {
            Value::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn is_obj(&self) -> bool {
        matches!(self, Value::Obj(_))
    }

    pub fn set(&mut self, key: &str, val: Value) {
        let Value::Obj(members) = self else {
            return;
        };
        let Some(first) = members.iter().position(|(k, _)| k == key) else {
            members.push((String::from(key), val));
            return;
        };
        let mut index = 0;
        members.retain(|(k, _)| {
            index += 1;
            index - 1 == first || k != key
        });
        members[first].1 = val;
    }

    pub fn remove(&mut self, key: &str) -> bool {
        if let Value::Obj(members) = self {
            let n = members.len();
            members.retain(|(k, _)| k != key);
            return members.len() != n;
        }
        false
    }
}
