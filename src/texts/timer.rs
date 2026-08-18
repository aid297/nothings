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

pub trait Timer {
    fn with_text(s: &str) -> Result<TimerImpl, String>;
    fn get_duration(&self)->Duration;
    fn to_chinese(&self) -> Result<String, String>;
}
pub struct TimerImpl{
    duration: Duration,
    text: String,
}

impl Default for TimerImpl {
    fn default() -> Self{
        TimerImpl{duration: Duration::from_secs(0),text:"".to_string()}
    }
}

impl Timer for TimerImpl{
    fn with_text(s: &str) -> Result<TimerImpl, String> {
        let mut t = TimerImpl::default();
        t.text = s.to_string();
        
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
        
        t.duration = Duration::from_secs(total_secs);
        
        Ok(t)
    }
    
    fn get_duration(&self)->Duration{
        self.duration
    }
    
    fn to_chinese(&self) -> Result<String, String> {
        if self.text.is_empty() {
            return Err("空字符串".to_string());
        }
        
        let mut parts = Vec::new();
        let mut current_num = String::new();
        let mut last_unit: Option<char> = None;
        
        for c in self.text.chars() {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_seconds() {
        assert_eq!(TimerImpl::with_text("10s").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(10));
        assert_eq!(TimerImpl::with_text("0s").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(0));
        assert_eq!(TimerImpl::with_text("60s").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(60));
    }

    #[test]
    fn test_parse_minutes() {
        assert_eq!(TimerImpl::with_text("5m").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(300));
        assert_eq!(TimerImpl::with_text("1m").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(60));
        assert_eq!(TimerImpl::with_text("0m").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(0));
    }

    #[test]
    fn test_parse_hours() {
        assert_eq!(TimerImpl::with_text("2h").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(7200));
        assert_eq!(TimerImpl::with_text("1h").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(3600));
        assert_eq!(TimerImpl::with_text("24h").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(86400));
    }

    #[test]
    fn test_parse_days() {
        assert_eq!(TimerImpl::with_text("1d").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(86400));
        assert_eq!(TimerImpl::with_text("7d").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(604800));
        assert_eq!(TimerImpl::with_text("30d").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(2592000));
    }

    #[test]
    fn test_parse_weeks() {
        assert_eq!(TimerImpl::with_text("1w").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(604800));
        assert_eq!(TimerImpl::with_text("2w").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(1209600));
    }

    #[test]
    fn test_parse_combined() {
        assert_eq!(TimerImpl::with_text("1h30m").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(5400));
        assert_eq!(TimerImpl::with_text("1d12h").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(129600));
        assert_eq!(TimerImpl::with_text("2w3d").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(1468800));
        assert_eq!(TimerImpl::with_text("1h30m10s").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(5410));
        assert_eq!(TimerImpl::with_text("1d2h3m4s").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(93784));
    }

    #[test]
    fn test_parse_without_unit() {
        // 纯数字按秒处理
        assert_eq!(TimerImpl::with_text("100").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(100));

        // 有前一个单位时，按下一级单位处理
        // 1h100 = 1小时 + 100分钟 = 3600 + 6000 = 9600秒
        assert_eq!(TimerImpl::with_text("1h100").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(9600));

        // 1w2 = 1周 + 2天 = 604800 + 172800 = 777600秒
        assert_eq!(TimerImpl::with_text("1w2").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(777600));

        // 1d12 = 1天 + 12小时 = 86400 + 43200 = 129600秒
        assert_eq!(TimerImpl::with_text("1d12").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(129600));

        // 5m30 = 5分钟 + 30秒 = 300 + 30 = 330秒
        assert_eq!(TimerImpl::with_text("5m30").unwrap_or(TimerImpl::default()).get_duration(), Duration::from_secs(330));
    }

    #[test]
    fn test_parse_empty() {
        assert!(TimerImpl::with_text("").is_err());
    }

    #[test]
    fn test_parse_invalid_unit() {
        assert!(TimerImpl::with_text("10x").is_err());
        assert!(TimerImpl::with_text("10y").is_err());
        assert!(TimerImpl::with_text("1M").is_err());
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(TimerImpl::with_text("s10").is_err());
        assert!(TimerImpl::with_text("hs").is_err());
    }

    #[test]
    fn test_describe_duration() {
        assert_eq!(TimerImpl::with_text("1h30m").unwrap_or(TimerImpl::default()).to_chinese(), Ok("1小时30分钟".to_string()));
        assert_eq!(TimerImpl::with_text("2d").unwrap_or(TimerImpl::default()).to_chinese(), Ok("2天".to_string()));
        assert_eq!(TimerImpl::with_text("1w2d").unwrap_or(TimerImpl::default()).to_chinese(), Ok("1周2天".to_string()));
        assert_eq!(TimerImpl::with_text("100").unwrap_or(TimerImpl::default()).to_chinese(), Ok("100秒".to_string()));

        // 无单位时按前一个单位的下一级
        assert_eq!(TimerImpl::with_text("1h100").unwrap_or(TimerImpl::default()).to_chinese(), Ok("1小时100分钟".to_string()));
        assert_eq!(TimerImpl::with_text("1w2").unwrap_or(TimerImpl::default()).to_chinese(), Ok("1周2天".to_string()));
        assert_eq!(TimerImpl::with_text("1d12").unwrap_or(TimerImpl::default()).to_chinese(), Ok("1天12小时".to_string()));
        assert_eq!(TimerImpl::with_text("5m30").unwrap_or(TimerImpl::default()).to_chinese(), Ok("5分钟30秒".to_string()));
    }

    #[test]
    fn test_duration_methods() {
        // 验证返回的 Duration 可以正常使用标准库方法
        let d = TimerImpl::with_text("1h30m").unwrap();
        assert_eq!(d.get_duration().as_secs(), 5400);
        assert_eq!(d.get_duration().as_secs_f64(), 5400.0);
        assert_eq!(d.get_duration().as_millis(), 5400000);
    }
}
