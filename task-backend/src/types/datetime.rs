use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn inner(&self) -> DateTime<Utc> {
        self.0
    }

    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.timestamp().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        DateTime::from_timestamp(timestamp, 0)
            .map(Self)
            .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))
    }
}

pub mod optional_timestamp {
    use super::*;

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(dt) => dt.timestamp().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<i64>::deserialize(deserializer)? {
            Some(ts) => DateTime::from_timestamp(ts, 0)
                .map(Some)
                .ok_or_else(|| serde::de::Error::custom("Invalid timestamp")),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_timestamp_serialization() {
        let now = Utc::now();
        let timestamp = Timestamp::from_datetime(now);

        let serialized = serde_json::to_string(&timestamp).unwrap();
        let expected = now.timestamp().to_string();
        assert_eq!(serialized, expected);

        let deserialized: Timestamp = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.inner().timestamp(), now.timestamp());
    }

    #[test]
    fn test_optional_timestamp_serialization() {
        #[derive(Serialize, Deserialize)]
        struct TestStruct {
            #[serde(with = "optional_timestamp")]
            date: Option<DateTime<Utc>>,
        }

        let test_some = TestStruct {
            date: Some(Utc::now()),
        };
        let serialized = serde_json::to_string(&test_some).unwrap();
        assert!(serialized.contains(&test_some.date.unwrap().timestamp().to_string()));

        let test_none = TestStruct { date: None };
        let serialized = serde_json::to_string(&test_none).unwrap();
        assert_eq!(serialized, r#"{"date":null}"#);
    }
}
