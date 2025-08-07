#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Args {
    #[serde(deserialize_with = "not_empty_string_parser")]
    pub token: String,
    #[serde(deserialize_with = "not_empty_string_parser")]
    pub record: String,
    #[serde(rename = "zoneId", deserialize_with = "not_empty_string_parser")]
    pub zone_id: String,
    #[serde(default, deserialize_with = "url_parser")]
    pub providers: Vec<url::Url>,
}

pub fn parse() -> Result<Args, Error> {
    let path = std::env::args().nth(1).ok_or(Error::Path)?;
    let configuration = std::fs::read(path).map_err(Error::BadPath)?;
    serde_json::from_slice(&configuration).map_err(Error::Malformed)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing path to configuration file")]
    Path,
    #[error("Path to configuration is invalid: {0:?}")]
    BadPath(std::io::Error),
    #[error("Configuration file malformed: {0}")]
    Malformed(serde_json::Error),
}

fn not_empty_string_parser<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct Visitor;

    impl serde::de::Visitor<'_> for Visitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a non-empty string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            self.visit_string(String::from(v))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v.is_empty() {
                Err(serde::de::Error::custom("empty string for field"))
            } else {
                Ok(v)
            }
        }
    }

    deserializer.deserialize_string(Visitor)
}

fn url_parser<'de, D>(deserializer: D) -> Result<Vec<url::Url>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<url::Url>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a list of URLs")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(next) = seq.next_element::<&str>()? {
                let url = url::Url::parse(next).map_err(serde::de::Error::custom)?;
                vec.push(url);
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(Visitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let string = "{}";
        let parsed = serde_json::from_str::<Args>(string).unwrap_err();

        assert_eq!(
            parsed.to_string(),
            "missing field `token` at line 1 column 2"
        );
    }

    #[test]
    fn invalid_field() {
        let string = r#"{"yo":3}"#;
        let parsed = serde_json::from_str::<Args>(string).unwrap_err();

        assert_eq!(
            parsed.to_string(),
            "unknown field `yo`, expected one of `token`, `record`, `zoneId`, `providers` at line 1 column 5"
        );
    }

    #[test]
    fn empty_string() {
        let string = r#"{"token":""}"#;
        let parsed = serde_json::from_str::<Args>(string).unwrap_err();

        assert_eq!(
            parsed.to_string(),
            "empty string for field at line 1 column 11"
        );
    }

    #[test]
    fn bad_string() {
        let string = r#"{"token":3}"#;
        let parsed = serde_json::from_str::<Args>(string).unwrap_err();

        assert_eq!(
            parsed.to_string(),
            "invalid type: integer `3`, expected a non-empty string at line 1 column 10"
        );
    }

    #[test]
    fn bad_type_provider() {
        let string = r#"{"providers":3}"#;
        let parsed = serde_json::from_str::<Args>(string).unwrap_err();

        assert_eq!(
            parsed.to_string(),
            "invalid type: integer `3`, expected a list of URLs at line 1 column 14"
        );
    }

    #[test]
    fn bad_type_url() {
        let string = r#"{"providers":[3]}"#;
        let parsed = serde_json::from_str::<Args>(string).unwrap_err();

        assert_eq!(
            parsed.to_string(),
            "invalid type: integer `3`, expected a borrowed string at line 1 column 15"
        );
    }

    #[test]
    fn bad_url() {
        let string = r#"{"providers":[""]}"#;
        let parsed = serde_json::from_str::<Args>(string).unwrap_err();

        assert_eq!(
            parsed.to_string(),
            "relative URL without a base at line 1 column 17"
        );
    }

    #[test]
    fn missing_url() {
        let string = r#"{"token":"token","record":"record","zoneId":"zoneId"}"#;
        let parsed = serde_json::from_str::<Args>(string).unwrap();

        assert_eq!(parsed.token, "token");
        assert_eq!(parsed.record, "record");
        assert_eq!(parsed.zone_id, "zoneId");
        assert_eq!(parsed.providers, []);
    }

    #[test]
    fn empty_provider() {
        let string = r#"{"token":"token","record":"record","zoneId":"zoneId","providers":[]}"#;
        let parsed = serde_json::from_str::<Args>(string).unwrap();

        assert_eq!(parsed.token, "token");
        assert_eq!(parsed.record, "record");
        assert_eq!(parsed.zone_id, "zoneId");
        assert_eq!(parsed.providers, []);
    }

    #[test]
    fn full() {
        let string = r#"{"token":"token","record":"record","zoneId":"zoneId","providers":["https://localhost", "http:127.0.0.1"]}"#;
        let parsed = serde_json::from_str::<Args>(string).unwrap();

        assert_eq!(parsed.token, "token");
        assert_eq!(parsed.record, "record");
        assert_eq!(parsed.zone_id, "zoneId");
        assert_eq!(
            parsed.providers,
            [
                url::Url::parse("https://localhost").unwrap(),
                url::Url::parse("http:127.0.0.1").unwrap()
            ]
        );
    }
}
