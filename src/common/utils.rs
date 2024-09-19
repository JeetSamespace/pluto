pub mod handle_duration_string {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}s", duration.as_secs());
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let secs = s
            .trim_end_matches('s')
            .parse()
            .map_err(serde::de::Error::custom)?;
        Ok(Duration::from_secs(secs))
    }
}
