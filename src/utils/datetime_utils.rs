use chrono::SecondsFormat::Millis;
use chrono::{DateTime, Utc};

pub struct TimeUtils;

impl TimeUtils {
  pub fn utc_now() -> DateTime<Utc> {
    Utc::now()
  }
}

pub struct TimeUtilsBuilder {
  datetime: DateTime<Utc>,
}

impl TimeUtilsBuilder {
  pub fn new(datetime: DateTime<Utc>) -> Self {
    TimeUtilsBuilder { datetime }
  }

  pub fn to_timestamp(self) -> i64 {
    self.datetime.timestamp()
  }

  pub fn to_string(self) -> String {
    self.datetime.to_rfc3339_opts(Millis, true)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::datetime_utils::TimeUtils;
  use chrono::TimeZone;

  #[tokio::test]
  async fn test_utc_now() {
    let now_utc: DateTime<Utc> = TimeUtils::utc_now();
    let now_system: DateTime<Utc> = Utc::now();

    let difference: i64 = (now_system - now_utc).num_seconds();
    assert!(
      difference >= 0 && difference < 2,
      "Time difference is too large"
    );
  }

  #[tokio::test]
  async fn test_datetime_builder_new() {
    let datetime = Utc.with_ymd_and_hms(2024, 9, 17, 12, 34, 56).unwrap();
    let builder: TimeUtilsBuilder = TimeUtilsBuilder::new(datetime);
    assert_eq!(builder.datetime, datetime);
  }

  #[tokio::test]
  async fn test_datetime_builder_to_timestamp() {
    let datetime = Utc.with_ymd_and_hms(2024, 9, 17, 12, 34, 56).unwrap();
    let builder: TimeUtilsBuilder = TimeUtilsBuilder::new(datetime);
    assert_eq!(builder.to_timestamp(), datetime.timestamp());
  }

  #[tokio::test]
  async fn test_to_string() {
    let datetime = Utc.with_ymd_and_hms(2024, 9, 17, 12, 34, 56).unwrap();
    let builder: TimeUtilsBuilder = TimeUtilsBuilder::new(datetime);
    assert_eq!(builder.to_string(), datetime.to_rfc3339_opts(Millis, true));
  }
}
