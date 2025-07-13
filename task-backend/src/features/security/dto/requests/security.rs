// task-backend/src/features/security/dto/requests/security.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 日付範囲
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DateRange {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_date_range_deserialization() {
        let json = json!({
            "start_date": "2023-01-01T00:00:00Z",
            "end_date": "2023-12-31T23:59:59Z"
        });

        let date_range: DateRange = serde_json::from_value(json).unwrap();
        assert_eq!(
            date_range.start_date.to_rfc3339(),
            "2023-01-01T00:00:00+00:00"
        );
        assert_eq!(
            date_range.end_date.to_rfc3339(),
            "2023-12-31T23:59:59+00:00"
        );
    }
}
