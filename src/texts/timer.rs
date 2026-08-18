use std::time::Duration;

/// 时间单位常量
pub const SECOND: Duration = Duration::from_secs(1);
pub const MINUTE: Duration = Duration::from_secs(60);
pub const HOUR: Duration = Duration::from_secs(3600);
pub const DAY: Duration = Duration::from_secs(86400);
pub const WEEK: Duration = Duration::from_secs(604800);

/// 获取单位的下一级单位
fn next_smaller_unit(c: char) -> Option<Duration> {
    match c {
        'w' => Some(DAY),
        'd' => Some(HOUR),
        'h' => Some(MINUTE),
        'm' => Some(SECOND),
        's' => Some(SECOND),
        _ => None,
    }
}

/// 解析时间字符串，返回 Duration
///
/// 支持单位：
/// - `s`: 秒 (seconds)
/// - `m`: 分钟 (minutes)
/// - `h`: 小时 (hours)
/// - `d`: 天 (days)
/// - `w`: 周 (weeks)
///
/// 支持组合格式，如 `1h30m`、`1d12h`、`2w3d`
///
/// 末尾无单位数字按前一个单位的下一级处理：
/// - `1h100` = 1小时 + 100分钟
/// - `1w2` = 1周 + 2天
/// - 纯数字 `100` = 100秒
///
/// # Examples
///
/// ```
/// use nothings::texts::timer::parse_duration;
/// use std::time::Duration;
///
/// assert_eq!(parse_duration("10s"), Ok(Duration::from_secs(10)));
/// assert_eq!(parse_duration("5m"), Ok(Duration::from_secs(300)));
/// assert_eq!(parse_duration("2h"), Ok(Duration::from_secs(7200)));
/// assert_eq!(parse_duration("1d"), Ok(Duration::from_secs(86400)));
/// assert_eq!(parse_duration("1w"), Ok(Duration::from_secs(604800)));
/// assert_eq!(parse_duration("1h30m"), Ok(Duration::from_secs(5400)));
/// assert_eq!(parse_duration("1d12h"), Ok(Duration::from_secs(129600)));
/// assert_eq!(parse_duration("1h100"), Ok(Duration::from_secs(9600))); // 1h + 100m
/// assert_eq!(parse_duration("1w2"), Ok(Duration::from_secs(777600))); // 1w + 2d
/// ```
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    if s.is_empty() {
        return Err("空字符串".to_string());
    }

    let mut total_secs: u64 = 0;
    let mut current_num = String::new();
    let mut last_unit: Option<char> = None;

    for c in s.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else {
            if current_num.is_empty() {
                return Err(format!("意外的字符 '{}'", c));
            }

            let num: u64 = current_num
                .parse()
                .map_err(|_| format!("无法解析数字 '{}'", current_num))?;
            current_num.clear();

            let unit_duration = match c {
                's' => SECOND,
                'm' => MINUTE,
                'h' => HOUR,
                'd' => DAY,
                'w' => WEEK,
                _ => return Err(format!("未知的时间单位 '{}'", c)),
            };

            total_secs += num * unit_duration.as_secs();
            last_unit = Some(c);
        }
    }

    // 如果最后还有数字没有单位
    if !current_num.is_empty() {
        let num: u64 = current_num
            .parse()
            .map_err(|_| format!("无法解析数字 '{}'", current_num))?;

        // 按前一个单位的下一级处理，没有前一个单位则按秒处理
        let unit_secs = if let Some(last) = last_unit {
            next_smaller_unit(last)
                .unwrap_or(SECOND)
                .as_secs()
        } else {
            1 // 默认秒
        };

        total_secs += num * unit_secs;
    }

    Ok(Duration::from_secs(total_secs))
}

/// 解析时间字符串，返回人类可读的描述
///
/// # Examples
///
/// ```
/// use nothings::texts::timer::describe_duration;
///
/// assert_eq!(describe_duration("1h30m"), Ok("1小时30分钟".to_string()));
/// ```
pub fn describe_duration(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("空字符串".to_string());
    }

    let mut parts = Vec::new();
    let mut current_num = String::new();
    let mut last_unit: Option<char> = None;

    for c in s.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else {
            if current_num.is_empty() {
                return Err(format!("意外的字符 '{}'", c));
            }

            let unit_name = match c {
                's' => "秒",
                'm' => "分钟",
                'h' => "小时",
                'd' => "天",
                'w' => "周",
                _ => return Err(format!("未知的时间单位 '{}'", c)),
            };

            parts.push(format!("{}{}", current_num, unit_name));
            current_num.clear();
            last_unit = Some(c);
        }
    }

    if !current_num.is_empty() {
        // 按前一个单位的下一级处理
        let unit_name = if let Some(last) = last_unit {
            match last {
                'w' => "天",
                'd' => "小时",
                'h' => "分钟",
                'm' | 's' => "秒",
                _ => "秒",
            }
        } else {
            "秒"
        };
        parts.push(format!("{}{}", current_num, unit_name));
    }

    Ok(parts.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_seconds() {
        assert_eq!(parse_duration("10s"), Ok(Duration::from_secs(10)));
        assert_eq!(parse_duration("0s"), Ok(Duration::from_secs(0)));
        assert_eq!(parse_duration("60s"), Ok(Duration::from_secs(60)));
    }

    #[test]
    fn test_parse_minutes() {
        assert_eq!(parse_duration("5m"), Ok(Duration::from_secs(300)));
        assert_eq!(parse_duration("1m"), Ok(Duration::from_secs(60)));
        assert_eq!(parse_duration("0m"), Ok(Duration::from_secs(0)));
    }

    #[test]
    fn test_parse_hours() {
        assert_eq!(parse_duration("2h"), Ok(Duration::from_secs(7200)));
        assert_eq!(parse_duration("1h"), Ok(Duration::from_secs(3600)));
        assert_eq!(parse_duration("24h"), Ok(Duration::from_secs(86400)));
    }

    #[test]
    fn test_parse_days() {
        assert_eq!(parse_duration("1d"), Ok(Duration::from_secs(86400)));
        assert_eq!(parse_duration("7d"), Ok(Duration::from_secs(604800)));
        assert_eq!(parse_duration("30d"), Ok(Duration::from_secs(2592000)));
    }

    #[test]
    fn test_parse_weeks() {
        assert_eq!(parse_duration("1w"), Ok(Duration::from_secs(604800)));
        assert_eq!(parse_duration("2w"), Ok(Duration::from_secs(1209600)));
    }

    #[test]
    fn test_parse_combined() {
        assert_eq!(parse_duration("1h30m"), Ok(Duration::from_secs(5400)));
        assert_eq!(parse_duration("1d12h"), Ok(Duration::from_secs(129600)));
        assert_eq!(parse_duration("2w3d"), Ok(Duration::from_secs(1468800)));
        assert_eq!(parse_duration("1h30m10s"), Ok(Duration::from_secs(5410)));
        assert_eq!(parse_duration("1d2h3m4s"), Ok(Duration::from_secs(93784)));
    }

    #[test]
    fn test_parse_without_unit() {
        // 纯数字按秒处理
        assert_eq!(parse_duration("100"), Ok(Duration::from_secs(100)));

        // 有前一个单位时，按下一级单位处理
        // 1h100 = 1小时 + 100分钟 = 3600 + 6000 = 9600秒
        assert_eq!(parse_duration("1h100"), Ok(Duration::from_secs(9600)));

        // 1w2 = 1周 + 2天 = 604800 + 172800 = 777600秒
        assert_eq!(parse_duration("1w2"), Ok(Duration::from_secs(777600)));

        // 1d12 = 1天 + 12小时 = 86400 + 43200 = 129600秒
        assert_eq!(parse_duration("1d12"), Ok(Duration::from_secs(129600)));

        // 5m30 = 5分钟 + 30秒 = 300 + 30 = 330秒
        assert_eq!(parse_duration("5m30"), Ok(Duration::from_secs(330)));
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn test_parse_invalid_unit() {
        assert!(parse_duration("10x").is_err());
        assert!(parse_duration("10y").is_err());
        assert!(parse_duration("1M").is_err());
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(parse_duration("s10").is_err());
        assert!(parse_duration("hs").is_err());
    }

    #[test]
    fn test_describe_duration() {
        assert_eq!(describe_duration("1h30m"), Ok("1小时30分钟".to_string()));
        assert_eq!(describe_duration("2d"), Ok("2天".to_string()));
        assert_eq!(describe_duration("1w2d"), Ok("1周2天".to_string()));
        assert_eq!(describe_duration("100"), Ok("100秒".to_string()));

        // 无单位时按前一个单位的下一级
        assert_eq!(describe_duration("1h100"), Ok("1小时100分钟".to_string()));
        assert_eq!(describe_duration("1w2"), Ok("1周2天".to_string()));
        assert_eq!(describe_duration("1d12"), Ok("1天12小时".to_string()));
        assert_eq!(describe_duration("5m30"), Ok("5分钟30秒".to_string()));
    }

    #[test]
    fn test_duration_methods() {
        // 验证返回的 Duration 可以正常使用标准库方法
        let d = parse_duration("1h30m").unwrap();
        assert_eq!(d.as_secs(), 5400);
        assert_eq!(d.as_secs_f64(), 5400.0);
        assert_eq!(d.as_millis(), 5400000);
    }
}
