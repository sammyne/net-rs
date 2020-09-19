use std::collections::HashMap;

use super::errors::Error;

#[derive(Debug, Default)]
pub struct Values(pub HashMap<String, Vec<String>>);

impl Values {
    // @TODO: make key generic
    pub fn add<K, V>(&mut self, key: K, value: V)
    where
        K: ToString,
        V: ToString,
    {
        let key = key.to_string();
        if !self.0.contains_key(&key) {
            self.0.insert(key.clone(), Vec::new());
        }

        self.0.get_mut(&key).unwrap().push(value.to_string());
    }

    pub fn del(&mut self, key: &str) {
        self.0.remove(key);
    }

    pub fn encode(&self) -> String {
        let keys = {
            let mut keys = self.0.iter().map(|(k, _)| k).collect::<Vec<_>>();
            keys.sort();

            keys
        };

        let mut out = String::new();
        for k in keys {
            let values = self.0.get(k).unwrap();
            let key_escaped = super::query_escape(k);

            for v in values {
                if out.len() > 0 {
                    out.push('&');
                }

                out.push_str(key_escaped.as_str());
                out.push('=');
                out.push_str(super::query_escape(v).as_str());
            }
        }

        out
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|v| v[0].as_str())
    }

    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: ToString,
        V: ToString,
    {
        self.0.insert(key.to_string(), vec![value.to_string()]);
    }
}

pub fn parse_query(query: &str) -> Result<Values, (Values, Error)> {
    let mut err: Option<Error> = None;
    let mut out = Values(HashMap::new());

    let kv = query
        .split(|v| v == '&' || v == ';')
        .filter(|v| v.len() != 0)
        .map(|v| v.splitn(2, '=').collect::<Vec<_>>());

    for x in kv {
        let (k, v) = if x.len() == 1 {
            (x[0], "")
        } else {
            (x[0], x[1])
        };

        let kk = match super::query_unescape(k) {
            Ok(v) => v,
            Err(e) => {
                if err.is_none() {
                    err = Some(e);
                }
                continue;
            }
        };

        let vv = match super::query_unescape(v) {
            Ok(v) => v,
            Err(e) => {
                if err.is_none() {
                    err = Some(e);
                }
                continue;
            }
        };

        if !out.0.contains_key(&kk) {
            out.0.insert(kk.to_string(), Vec::new());
        }

        out.0.get_mut(&kk).unwrap().push(vv);
    }

    match err {
        Some(v) => Err((out, v)),
        None => Ok(out),
    }
}
