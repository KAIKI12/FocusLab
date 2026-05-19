//! 逻辑日 / 日期分界线工具。
//!
//! "逻辑日"不等于自然日 — 它从「日期分界线」(默认凌晨 4:00)跨到下一个分界线。
//! 研究生常熬夜到凌晨 1-2 点,这类工作应归属"今天"而不是"明天"。
//!
//! 详见 docs/04 §10.1-10.3。
//!
//! 所有按日期统计的 SQL(日结算 / streak / DTA 分配)都用 [`logical_date_range`]
//! 生成的 UTC 区间做 `WHERE start_time >= ?1 AND start_time < ?2`。

use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Timelike, Utc};
use chrono_tz::Tz;

/// 默认时区 — MVP 阶段固定 Asia/Shanghai。跨时区场景留到 Phase 4。
pub const DEFAULT_TZ: Tz = chrono_tz::Asia::Shanghai;

/// 默认日期分界线(凌晨 4 点)。用户可在 settings 覆盖。
pub const DEFAULT_BOUNDARY_HOUR: u32 = 4;

/// 把一个 UTC 时刻映射到逻辑日期。
///
/// 规则: 本地小时 < boundary_hour → 前一天,否则当天。
/// 即 `boundary_hour:00:00.000` 开始属于**当天**的逻辑日。
pub fn to_logical_date(utc: DateTime<Utc>, tz: Tz, boundary_hour: u32) -> NaiveDate {
    let local = utc.with_timezone(&tz);
    if local.hour() < boundary_hour {
        local.date_naive() - Duration::days(1)
    } else {
        local.date_naive()
    }
}

/// 给定逻辑日,返回它覆盖的 UTC 时间区间(左闭右开)。
///
/// 例如 boundary=4, date=2026-04-18, tz=Shanghai →
/// (2026-04-18T04:00 CST, 2026-04-19T04:00 CST) 转 UTC。
pub fn logical_date_range(
    date: NaiveDate,
    tz: Tz,
    boundary_hour: u32,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let start_local = tz
        .with_ymd_and_hms(date.year(), date.month(), date.day(), boundary_hour, 0, 0)
        .single()
        .expect("非模糊的本地时刻");
    let next = date + Duration::days(1);
    let end_local = tz
        .with_ymd_and_hms(next.year(), next.month(), next.day(), boundary_hour, 0, 0)
        .single()
        .expect("非模糊的本地时刻");
    (
        start_local.with_timezone(&Utc),
        end_local.with_timezone(&Utc),
    )
}

/// 当前时刻所在的逻辑日(基于本机系统时间转到 DEFAULT_TZ)。
pub fn current_logical_date(boundary_hour: u32) -> NaiveDate {
    to_logical_date(Utc::now(), DEFAULT_TZ, boundary_hour)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn shanghai_utc(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<Utc> {
        // 给定 Shanghai 本地时间,返回对应 UTC
        DEFAULT_TZ
            .with_ymd_and_hms(y, mo, d, h, mi, s)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn afternoon_is_today() {
        // Shanghai 10:00 → UTC 02:00;boundary=4;本地 10>4 → 当天
        let got = to_logical_date(shanghai_utc(2026, 4, 18, 10, 0, 0), DEFAULT_TZ, 4);
        assert_eq!(got, NaiveDate::from_ymd_opt(2026, 4, 18).unwrap());
    }

    #[test]
    fn early_morning_is_previous_day() {
        // Shanghai 03:00 → 本地 3<4 → 前一天
        let got = to_logical_date(shanghai_utc(2026, 4, 18, 3, 0, 0), DEFAULT_TZ, 4);
        assert_eq!(got, NaiveDate::from_ymd_opt(2026, 4, 17).unwrap());
    }

    #[test]
    fn exactly_boundary_is_today() {
        // Shanghai 04:00:00 整点 → 属于当天(闭区间起点)
        let got = to_logical_date(shanghai_utc(2026, 4, 18, 4, 0, 0), DEFAULT_TZ, 4);
        assert_eq!(got, NaiveDate::from_ymd_opt(2026, 4, 18).unwrap());
    }

    #[test]
    fn one_second_before_boundary_is_previous_day() {
        // Shanghai 03:59:59 → 前一天
        let got = to_logical_date(shanghai_utc(2026, 4, 18, 3, 59, 59), DEFAULT_TZ, 4);
        assert_eq!(got, NaiveDate::from_ymd_opt(2026, 4, 17).unwrap());
    }

    #[test]
    fn month_boundary() {
        // Shanghai 2026-05-01 02:30 → 前一月最后一天 2026-04-30
        let got = to_logical_date(shanghai_utc(2026, 5, 1, 2, 30, 0), DEFAULT_TZ, 4);
        assert_eq!(got, NaiveDate::from_ymd_opt(2026, 4, 30).unwrap());
    }

    #[test]
    fn year_boundary() {
        // Shanghai 2026-01-01 03:00 → 2025-12-31
        let got = to_logical_date(shanghai_utc(2026, 1, 1, 3, 0, 0), DEFAULT_TZ, 4);
        assert_eq!(got, NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
    }

    #[test]
    fn logical_range_is_24h() {
        let date = NaiveDate::from_ymd_opt(2026, 4, 18).unwrap();
        let (start, end) = logical_date_range(date, DEFAULT_TZ, 4);
        assert_eq!(end - start, Duration::hours(24));
    }

    #[test]
    fn logical_range_starts_at_boundary() {
        // 2026-04-18 的逻辑日 = Shanghai 2026-04-18 04:00:00 到 04-19 04:00:00
        let date = NaiveDate::from_ymd_opt(2026, 4, 18).unwrap();
        let (start, _end) = logical_date_range(date, DEFAULT_TZ, 4);
        let expected = shanghai_utc(2026, 4, 18, 4, 0, 0);
        assert_eq!(start, expected);
    }

    #[test]
    fn range_contains_afternoon_of_same_date() {
        // 验证逻辑日 2026-04-18 的范围覆盖 Shanghai 2026-04-18 15:00
        let date = NaiveDate::from_ymd_opt(2026, 4, 18).unwrap();
        let (start, end) = logical_date_range(date, DEFAULT_TZ, 4);
        let t = shanghai_utc(2026, 4, 18, 15, 0, 0);
        assert!(t >= start && t < end);
    }

    #[test]
    fn range_excludes_boundary_of_next_day() {
        // 右端点排除: 2026-04-19 04:00:00 不属于 2026-04-18
        let date = NaiveDate::from_ymd_opt(2026, 4, 18).unwrap();
        let (_start, end) = logical_date_range(date, DEFAULT_TZ, 4);
        let t = shanghai_utc(2026, 4, 19, 4, 0, 0);
        assert_eq!(t, end); // t 正好等于 end,>= end → 不属于该逻辑日
    }
}
