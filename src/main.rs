use anyhow::Result;
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Invalid character  {0:?} at {1:?})")]
    InvalidCharacter(String, usize),
    #[error("Invalid range")]
    InvalidRange(#[from] anyhow::Error),
    #[error("Parsing Error")]
    ParsingError(),
}

struct Range {
    start: i64,
    end: i64,
}

struct ParsingResults {
    numbers: Option<Vec<i64>>,
    range: Option<Range>,
    wildcard: bool,
    periodic_range: Option<i64>,
}

impl ParsingResults {
    fn validate(&self, range_begin: i64, range_end: i64) -> Result<(), anyhow::Error> {
        if let Some(numbers) = self.numbers.as_ref() {
            for number in numbers {
                if *number < range_begin || *number > range_end {
                    return Err(anyhow::anyhow!("Invalid number"));
                }
            }
        }

        if let Some(range) = self.range.as_ref() {
            if range.start < range_begin || range.end > range_end {
                return Err(anyhow::anyhow!("Invalid range"));
            }
        }

        if let Some(periodic_range) = self.periodic_range {
            if periodic_range < range_begin || periodic_range > range_end || periodic_range <= 0 {
                return Err(anyhow::anyhow!("Invalid periodic range"));
            }
        }

        Ok(())
    }
}

fn is_day_of_week(str: &String) -> Option<i64> {
    let days_of_week = HashMap::from([
        ("Mon", 1),
        ("Tue", 2),
        ("Wed", 3),
        ("Thu", 4),
        ("Fri", 5),
        ("Sat", 6),
        ("San", 7),
    ]);

    for (day, number) in days_of_week {
        if str == day {
            return Some(number);
        }
    }

    None
}

fn parse_param(str: String) -> Result<ParsingResults, ParserError> {
    let str = str.trim().to_string();
    dbg!(str.clone());
    if str.is_empty() {
        return Err(ParserError::InvalidCharacter(str, 0));
    }

    if let Some(day) = is_day_of_week(&str) {
        return Ok(ParsingResults {
            numbers: Some(vec![day]),
            range: None,
            wildcard: false,
            periodic_range: None,
        });
    }

    let mut has_number = false;
    for chat in str.chars() {
        if chat.is_numeric() {
            has_number = true;
            break;
        }
    }

    if !has_number {
        if str.contains('-') {
            let parts: Vec<&str> = str.split('-').collect();
            if parts.len() != 2 {
                return Err(ParserError::InvalidRange(anyhow::anyhow!("Invalid range")));
            }

            if let Some(range_start) = is_day_of_week(&parts[0].to_string()) {
                if let Some(range_end) = is_day_of_week(&parts[1].to_string()) {
                    return Ok(ParsingResults {
                        range: Some(Range {
                            start: range_start,
                            end: range_end,
                        }),
                        numbers: None,
                        wildcard: false,
                        periodic_range: None,
                    });
                }
            }
        }

        if str.contains(',') {
            let parts: Vec<&str> = str.split(',').collect();
            let mut numbers = Vec::new();
            for part in parts {
                if let Some(number) = is_day_of_week(&part.to_string()) {
                    numbers.push(number);
                } else {
                    return Err(ParserError::InvalidCharacter(part.to_string(), 0));
                }
            }
            return Ok(ParsingResults {
                numbers: Some(numbers),
                range: None,
                wildcard: false,
                periodic_range: None,
            });
        }
    }

    if str == "*" {
        return Ok(ParsingResults {
            wildcard: true,
            numbers: None,
            range: None,
            periodic_range: None,
        });
    }

    if str.contains('-') {
        let parts: Vec<&str> = str.split('-').collect();
        if parts.len() != 2 {
            return Err(ParserError::InvalidRange(anyhow::anyhow!("Invalid range")));
        }
        let start = parts[0]
            .parse::<i64>()
            .map_err(|_| ParserError::InvalidRange(anyhow::anyhow!("Invalid range")))?;
        let end = parts[1]
            .parse::<i64>()
            .map_err(|_| ParserError::InvalidRange(anyhow::anyhow!("Invalid range")))?;
        let range = Range { start, end };
        return Ok(ParsingResults {
            range: Some(range),
            wildcard: false,
            numbers: None,
            periodic_range: None,
        });
    }
    if str.contains(',') {
        let parts: Vec<&str> = str.split(',').collect();
        let mut numbers = Vec::new();
        for (index, part) in parts.iter().enumerate() {
            let number = part
                .parse::<i64>()
                .map_err(|_| ParserError::InvalidCharacter(part.to_string(), index))?;
            numbers.push(number);
        }
        return Ok(ParsingResults {
            numbers: Some(numbers),
            range: None,
            wildcard: false,
            periodic_range: None,
        });
    }
    if str.starts_with("*/") {
        let mut number: i64 = 0;
        for (index, char) in str.chars().skip(2).enumerate() {
            if !char.is_ascii_digit() {
                return Err(ParserError::InvalidCharacter(char.to_string(), index));
            }
            number = number * 10 + char.to_digit(10).unwrap() as i64;
        }
        return Ok(ParsingResults {
            numbers: None,
            range: None,
            wildcard: false,
            periodic_range: Some(number),
        });
    }

    let mut number: i64 = 0;
    for (index, char) in str.chars().enumerate() {
        if !char.is_ascii_digit() {
            return Err(ParserError::InvalidCharacter(char.to_string(), index));
        }
        number = number * 10 + char.to_digit(10).unwrap() as i64;
    }

    Ok(ParsingResults {
        numbers: Some(vec![number]),
        range: None,
        wildcard: false,
        periodic_range: None,
    })
}

#[derive(Debug)]
struct Minutes {
    minutes: Vec<i64>,
}

impl Minutes {
    fn new(param: ParsingResults) -> Result<Self, anyhow::Error> {
        param.validate(0, 60)?;
        if param.wildcard {
            let mut minutes = Vec::new();
            for i in 0..60 {
                minutes.push(i);
            }
            return Ok(Minutes { minutes });
        }

        if let Some(range) = param.range.as_ref() {
            let mut minutes = Vec::new();
            for i in range.start..=range.end {
                minutes.push(i);
            }
            return Ok(Minutes { minutes });
        }

        if let Some(mut minutes) = param.numbers {
            minutes.sort();
            return Ok(Minutes { minutes });
        }

        if let Some(periodic_range) = param.periodic_range {
            let mut minutes = Vec::new();
            for i in (0..60).step_by(periodic_range as usize) {
                minutes.push(i);
            }
            return Ok(Minutes { minutes });
        }

        Err(anyhow::anyhow!("can't build minutes"))
    }
}

#[derive(Debug)]
struct Hours {
    hours: Vec<i64>,
}

impl Hours {
    fn new(param: ParsingResults) -> Result<Hours, anyhow::Error> {
        param.validate(0, 24)?;
        if param.wildcard {
            return Ok(Hours {
                hours: (0..24).collect(),
            });
        }

        if let Some(range) = param.range.as_ref() {
            return Ok(Hours {
                hours: (range.start..=range.end).collect(),
            });
        }

        if let Some(mut hours) = param.numbers {
            hours.sort();
            return Ok(Hours { hours });
        }

        if let Some(periodic_range) = param.periodic_range {
            return Ok(Hours {
                hours: (0..24).step_by(periodic_range as usize).collect(),
            });
        }

        Err(anyhow::anyhow!("can't build hours"))
    }
}

#[derive(Debug)]
struct DaysOfMonth {
    days: Vec<i64>,
}

impl DaysOfMonth {
    fn new(param: ParsingResults) -> Result<DaysOfMonth, anyhow::Error> {
        param.validate(1, 31)?;
        if param.wildcard {
            return Ok(DaysOfMonth {
                days: (1..=31).collect(),
            });
        }

        if let Some(range) = param.range.as_ref() {
            return Ok(DaysOfMonth {
                days: (range.start..=range.end).collect(),
            });
        }

        if let Some(mut days) = param.numbers {
            days.sort();
            return Ok(DaysOfMonth { days });
        }

        if let Some(periodic_range) = param.periodic_range {
            return Ok(DaysOfMonth {
                days: (1..=31).step_by(periodic_range as usize).collect(),
            });
        }

        Err(anyhow::anyhow!("can't build days of month"))
    }
}

#[derive(Debug)]
struct DaysOfWeek {
    days: Vec<i64>,
}
impl DaysOfWeek {
    fn new(param: ParsingResults) -> Result<DaysOfWeek, anyhow::Error> {
        param.validate(1, 7)?;
        if param.wildcard {
            return Ok(Self {
                days: (1..=7).collect(),
            });
        }

        if let Some(range) = param.range {
            if range.start > range.end {
                let start: Vec<i64> = (1..=range.end).collect();
                let mut end: Vec<i64> = (range.start..=7).collect();
                end.extend(start);
                return Ok(Self { days: end });
            }
            return Ok(Self {
                days: (range.start..=range.end).collect(),
            });
        }

        if let Some(mut days) = param.numbers {
            days.sort();
            return Ok(Self { days });
        }

        if let Some(periodic_range) = param.periodic_range {
            return Ok(Self {
                days: (1..=7).step_by(periodic_range as usize).collect(),
            });
        }

        Err(anyhow::anyhow!("can't build days of week"))
    }
}

#[derive(Debug)]
struct Months {
    months: Vec<i64>,
}

impl Months {
    fn new(param: ParsingResults) -> Result<Months, anyhow::Error> {
        param.validate(1, 12)?;
        if param.wildcard {
            return Ok(Self {
                months: (1..=12).collect(),
            });
        }

        if let Some(range) = param.range {
            return Ok(Self {
                months: (range.start..=range.end).collect(),
            });
        }

        if let Some(mut months) = param.numbers {
            months.sort();
            return Ok(Self { months });
        }

        if let Some(periodic_range) = param.periodic_range {
            return Ok(Self {
                months: (1..=12).step_by(periodic_range as usize).collect(),
            });
        }

        Err(anyhow::anyhow!("can't build months"))
    }
}

#[derive(Debug)]
struct Command {
    command: String,
}

impl Command {
    // no validation yet
    fn new(command: String) -> Self {
        Command { command }
    }
}

#[derive(Debug)]
struct Cron {
    minute: Minutes,
    hour: Hours,
    day_of_month: DaysOfMonth,
    day_of_week: DaysOfWeek,
    month: Months,
    command: Command,
}

impl fmt::Display for Cron {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:14} {}", "minutes", to_string(&self.minute.minutes))?;
        writeln!(f, "{:14} {}", "hour", to_string(&self.hour.hours))?;
        writeln!(
            f,
            "{:14} {}",
            "day of month",
            to_string(&self.day_of_month.days)
        )?;
        writeln!(f, "{:14} {}", "month", to_string(&self.month.months))?;
        writeln!(
            f,
            "{:14} {}",
            "day of week",
            to_string(&self.day_of_week.days)
        )?;
        writeln!(f, "{:14} {}", "command", self.command.command)?;

        Ok(())
    }
}

fn to_string(v: &Vec<i64>) -> String {
    let mut comma_separated = String::new();

    for num in &v[0..v.len() - 1] {
        comma_separated.push_str(&num.to_string());
        comma_separated.push_str(", ");
    }

    comma_separated.push_str(&v[v.len() - 1].to_string());
    comma_separated.to_string()
}

fn parse(str: String) -> Result<Cron, anyhow::Error> {
    let parts = str
        .split_whitespace()
        .map(|f| f.to_string())
        .collect::<Vec<String>>();
    if parts.len() != 6 {
        return Err(anyhow::anyhow!(
            "invalid number of parts in cron expression"
        ));
    }
    let mut parts = parts.iter();
    let minute = Minutes::new(parse_param(parts.next().unwrap().to_string())?)?;
    dbg!(&minute);
    let hour = Hours::new(parse_param(parts.next().unwrap().to_string())?)?;
    dbg!(&hour);
    let day_of_month = DaysOfMonth::new(parse_param(parts.next().unwrap().to_string())?)?;
    dbg!(&day_of_month);
    let month = Months::new(parse_param(parts.next().unwrap().to_string())?)?;
    dbg!(&month);
    let day_of_week = DaysOfWeek::new(parse_param(parts.next().unwrap().to_string())?)?;
    dbg!(&day_of_week);

    let command = Command::new(parts.next().unwrap().to_string());
    dbg!(&command);
    Ok(Cron {
        minute,
        hour,
        day_of_month,
        day_of_week,
        month,
        command,
    })
}
fn main() {
    let arg = std::env::args().nth(1).unwrap();
    match parse(arg) {
        Ok(cron) => println!("{}", cron),
        Err(e) => println!("{}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_from_task() -> Result<(), Box<dyn std::error::Error>> {
        let example = "*/15 0 1,15 * 1-5 /usr/bin/find";
        let result = parse(example.to_string())?;

        assert_eq!(result.minute.minutes, vec![0, 15, 30, 45]);
        assert_eq!(result.hour.hours, vec![0]);
        assert_eq!(result.day_of_month.days, vec![1, 15]);
        assert_eq!(
            result.month.months,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
        );
        assert_eq!(result.day_of_week.days, vec![1, 2, 3, 4, 5]);
        assert_eq!(result.command.command, "/usr/bin/find");
        Ok(())
    }
    #[test]
    fn test_all_stars() -> Result<(), Box<dyn std::error::Error>> {
        let example = "* * * * * /usr/bin/find";
        let result = parse(example.to_string())?;

        assert_eq!(result.minute.minutes, (0..=59).collect::<Vec<i64>>());
        assert_eq!(result.hour.hours, (0..=23).collect::<Vec<i64>>());
        assert_eq!(result.day_of_month.days, (1..=31).collect::<Vec<i64>>());
        assert_eq!(result.month.months, (1..=12).collect::<Vec<i64>>());
        assert_eq!(result.day_of_week.days, (1..=7).collect::<Vec<i64>>());
        assert_eq!(result.command.command, "/usr/bin/find");
        Ok(())
    }

    #[test]
    fn test_concrete_values() -> Result<(), Box<dyn std::error::Error>> {
        let example = "1,2 1,2 1,2 1,2 1,2 /usr/bin/find";
        let result = parse(example.to_string())?;
        let right = vec![1, 2];
        assert_eq!(result.minute.minutes, right);
        assert_eq!(result.hour.hours, right);
        assert_eq!(result.day_of_month.days, right);
        assert_eq!(result.month.months, right);
        assert_eq!(result.day_of_week.days, right);
        assert_eq!(result.command.command, "/usr/bin/find");
        Ok(())
    }
    #[test]
    fn test_concrete_values_order() -> Result<(), Box<dyn std::error::Error>> {
        let example = "2,1 2,1 2,1 2,1 2,1 /usr/bin/find";
        let result = parse(example.to_string())?;
        let right = vec![1, 2];
        assert_eq!(result.minute.minutes, right);
        assert_eq!(result.hour.hours, right);
        assert_eq!(result.day_of_month.days, right);
        assert_eq!(result.month.months, right);
        assert_eq!(result.day_of_week.days, right);
        assert_eq!(result.command.command, "/usr/bin/find");
        Ok(())
    }
    #[test]
    fn test_ranges() -> Result<(), Box<dyn std::error::Error>> {
        let example = "1-2 1-2 1-2 1-2 1-2 /usr/bin/find";
        let result = parse(example.to_string())?;
        let right = vec![1, 2];
        assert_eq!(result.minute.minutes, right);
        assert_eq!(result.hour.hours, right);
        assert_eq!(result.day_of_month.days, right);
        assert_eq!(result.month.months, right);
        assert_eq!(result.day_of_week.days, right);
        assert_eq!(result.command.command, "/usr/bin/find");
        Ok(())
    }

    #[test]
    fn test_period_values() -> Result<(), Box<dyn std::error::Error>> {
        let example = "*/15 */2 */2 */2 */2 /usr/bin/find";
        let result = parse(example.to_string())?;
        assert_eq!(result.minute.minutes, vec![0, 15, 30, 45]);
        assert_eq!(
            result.hour.hours,
            vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22]
        );
        assert_eq!(
            result.day_of_month.days,
            vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31]
        );
        assert_eq!(result.month.months, vec![1, 3, 5, 7, 9, 11]);
        assert_eq!(result.day_of_week.days, vec![1, 3, 5, 7]);
        assert_eq!(result.command.command, "/usr/bin/find");
        Ok(())
    }
    #[test]
    fn test_bad_expressions() {
        let examples = vec![
            "*/15 */2 */2 */2 */2",
            "*/15 */2 */2 */2 */2 hello world",
            "*/0 */2 */2 */2 */2 /usr/bin/find",
            "*/15 */0 */2 */2 */2 /usr/bin/find",
            "*/15 */2 */0 */2 */2 /usr/bin/find",
            "*/15 */2 */2 */0 */2 /usr/bin/find",
            "*/15 */2 */2 */2 */0 /usr/bin/find",
            "*/-1 */2 */2 */2 */2 /usr/bin/find",
            "*/15 */-2 */2 */2 */2 /usr/bin/find",
            "*/15 */2 */-2 */2 */2 /usr/bin/find",
            "*/15 */2 */2 */-2 */2 /usr/bin/find",
            "*/15 */2 */2 */2 */-2 /usr/bin/find",
            "0,98 */2 */2 */2 */2 /usr/bin/find",
            "0-98 */2 */2 */2 */2 /usr/bin/find",
            "*/2 0-98 */2 */2 */2 /usr/bin/find",
            "*/2 0,98 */2 */2 */2 /usr/bin/find",
            "*/2 0,1 982,1 */2 */2 /usr/bin/find",
        ];
        for example in examples {
            let result = parse(example.to_string());
            assert!(result.is_err(), "{}", example.to_string());
        }
    }
}
