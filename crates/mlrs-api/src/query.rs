//! Raw query-string handling.
//!
//! `serde_urlencoded` silently tolerates repeated keys, but the API contract
//! requires a `400 VALIDATION_ERROR` for them. Parsing the raw string keeps
//! duplicates visible so the handlers can reject them explicitly.

#[derive(Debug, Clone, Default)]
pub struct QueryPairs {
    pairs: Vec<(String, String)>,
}

impl QueryPairs {
    pub fn parse(raw: Option<&str>) -> Self {
        let pairs = form_urlencoded::parse(raw.unwrap_or("").as_bytes())
            .into_owned()
            .collect();
        Self { pairs }
    }

    /// The first value for `name`.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// The name of the first repeated parameter in `names`, if any.
    pub fn repeated(&self, names: &[&str]) -> Option<String> {
        names.iter().find_map(|name| {
            let count = self.pairs.iter().filter(|(key, _)| key == name).count();
            (count > 1).then(|| (*name).to_string())
        })
    }
}
