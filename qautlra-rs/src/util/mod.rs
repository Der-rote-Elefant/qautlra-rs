pub mod predict;
pub mod tradedate;

use chrono::{TimeZone, Utc};

pub fn get_qadatestamp() -> i64 {
    // qainside && qifi protocol both use utc[+0] as timestamp

    let now = Utc::now();
    let timestamp = now.timestamp_nanos();
    timestamp
}
/// 将时间戳转换为格式化的日期字符串。
///
/// # 参数
/// `ts` - 以纳秒为单位的UNIX时间戳。
///
/// # 返回值
/// 返回一个格式为`"YYYY-MM-DD HH:MM:SS"`的日期字符串。
pub fn parse_datestamp(ts: i64) -> String {
    // 将时间戳转换为UTC时间，并调整时区偏移量以适应东八区（北京时间）
    let dt = Utc.timestamp_nanos(ts + 28800000000000);
    // 使用指定格式格式化时间，并转换为字符串
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn parse_fromstr_datestamp(datetime: String) -> i64 {
    match datetime.len() {
        19 => {
            Utc.datetime_from_str(&datetime, "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .timestamp_nanos()
                - 28800000000000
        }
        10 => {
            let dt = format!("{} 00:00:00", datetime);
            Utc.datetime_from_str(&dt, "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .timestamp_nanos()
                - 28800000000000
        }
        _ => 0,
    }
}
