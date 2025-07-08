use chrono::Duration;
#[allow(unused_imports)]
use chrono::{DateTime, Utc};

#[allow(dead_code)]
pub struct TimeHandler;

#[allow(dead_code)]
impl TimeHandler {
    pub fn new() -> Self {
        TimeHandler
    }

    pub fn get_current_time(&self) -> DateTime<Utc> {
        Utc::now()
    }

    pub fn format_time(&self, time: DateTime<Utc>) -> String {
        time.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn parse_time(&self, time_str: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(time_str).unwrap().with_timezone(&Utc)
    }

    pub fn add_seconds(&self, time: DateTime<Utc>, seconds: i64) -> DateTime<Utc> {
        time + Duration::seconds(seconds)
    }

    pub fn add_minutes(&self, time: DateTime<Utc>, minutes: i64) -> DateTime<Utc> {
        time + Duration::minutes(minutes)
    }

    pub fn add_hours(&self, time: DateTime<Utc>, hours: i64) -> DateTime<Utc> {
        time + Duration::hours(hours)
    }

    pub fn add_days(&self, time: DateTime<Utc>, days: i64) -> DateTime<Utc> {
        time + Duration::days(days)
    }

    pub fn add_weeks(&self, time: DateTime<Utc>, weeks: i64) -> DateTime<Utc> {
        time + Duration::weeks(weeks)
    }

    pub fn add_months(&self, time: DateTime<Utc>, months: i64) -> DateTime<Utc> {
        time + Duration::days(months * 30)
    }

    pub fn add_years(&self, time: DateTime<Utc>, years: i64) -> DateTime<Utc> {
        time + Duration::days(years * 365)
    }

    pub fn subtract_seconds(&self, time: DateTime<Utc>, seconds: i64) -> DateTime<Utc> {
        time - Duration::seconds(seconds)
    }

    pub fn subtract_minutes(&self, time: DateTime<Utc>, minutes: i64) -> DateTime<Utc> {
        time - Duration::minutes(minutes)
    }

    pub fn subtract_hours(&self, time: DateTime<Utc>, hours: i64) -> DateTime<Utc> {
        time - Duration::hours(hours)
    }

    pub fn subtract_days(&self, time: DateTime<Utc>, days: i64) -> DateTime<Utc> {
        time - Duration::days(days)
    }

    pub fn subtract_weeks(&self, time: DateTime<Utc>, weeks: i64) -> DateTime<Utc> {
        time - Duration::weeks(weeks)
    }

    pub fn subtract_months(&self, time: DateTime<Utc>, months: i64) -> DateTime<Utc> {
        time - Duration::days(months * 30)
    }

    pub fn subtract_years(&self, time: DateTime<Utc>, years: i64) -> DateTime<Utc> {
        time - Duration::days(years * 365)
    }
}
