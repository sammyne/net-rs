use std::collections::HashMap;

use super::errors::Error;

/// Values maps a string key to a list of values.
/// It is typically used for query parameters and form values.
/// Unlike in the http.Header map, the keys in a Values map
/// are case-sensitive.
#[derive(Debug, Default)]
pub struct Values(pub HashMap<String, Vec<String>>);

impl Values {
    /// add adds the value to key. It appends to any existing
    /// values associated with key.
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

    /// del deletes the values associated with key.
    pub fn del(&mut self, key: &str) {
        self.0.remove(key);
    }

    /// encode encodes the values into "URL encoded" form
    /// ("bar=baz&foo=quux") sorted by key.
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

    /// get gets the first value associated with the given key.
    /// If there are no values associated with the key, get returns
    /// None. To access multiple values, use the map directly.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|v| v[0].as_str())
    }

    /// set sets the key to value. It replaces any existing values.
    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: ToString,
        V: ToString,
    {
        self.0.insert(key.to_string(), vec![value.to_string()]);
    }
}

/// parse_query parses the URL-encoded query string and returns
/// a map listing the values specified for each key.
/// parse_query always returns a non-nil map containing all the
/// valid query parameters found; err describes the first decoding error
/// encountered, if any.
///
/// Query is expected to be a list of key=value settings separated by
/// ampersands or semicolons. A setting without an equals sign is
/// interpreted as a key set to an empty value.
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
