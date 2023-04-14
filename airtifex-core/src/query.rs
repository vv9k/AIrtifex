use std::string::ToString;
use url::form_urlencoded;

pub trait UrlQuery {
    fn as_query(&self) -> String;
}

/// Creates an endpoint with a query
pub fn construct_ep<E, Q>(ep: E, query: Option<Q>) -> String
where
    E: Into<String>,
    Q: AsRef<str>,
{
    let mut ep = ep.into();
    if let Some(query) = query {
        ep = append_query(ep, query);
    }
    ep
}

/// Appends a query to an endpoint
pub fn append_query<Q>(mut ep: String, query: Q) -> String
where
    Q: AsRef<str>,
{
    ep.push('?');
    ep.push_str(query.as_ref());
    ep
}

/// Encodes `key` and `val` as urlencoded values.
pub fn encoded_pair<K, V>(key: K, val: V) -> String
where
    K: AsRef<str>,
    V: ToString,
{
    form_urlencoded::Serializer::new(String::new())
        .append_pair(key.as_ref(), &val.to_string())
        .finish()
}
