use anyhow::Result;
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
            if periodic_range < range_begin || periodic_range > range_end || periodic_range <=0 {
                return Err(anyhow::anyhow!("Invalid periodic range"));
            }
        }

        Ok(())
    }
}

fn parse_param(str: String) -> Result<ParsingResults, ParserError> {
    let str = str.trim().to_string();
    dbg!(str.clone());
    if str.len() == 0 {
        return Err(ParserError::InvalidCharacter(str, 0));
    }
    if str == "*" {
        return Ok(ParsingResults {
            wildcard: true,
            numbers: None,
            range: None,
            periodic_range: None,
        });
    }

    if str.contains("-") {
        let parts: Vec<&str> = str.split("-").collect();
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
    if str.contains(",") {
        let parts: Vec<&str> = str.split(",").collect();
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
            if !char.is_digit(10) {
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
        if !char.is_digit(10) {
            return Err(ParserError::InvalidCharacter(char.to_string(), index));
        }
        number = number * 10 + char.to_digit(10).unwrap() as i64;
    }
    let mut numbers = Vec::new();
    numbers.push(number);

    Ok(ParsingResults {
        numbers: Some(numbers),
        range: None,
        wildcard: false,
        periodic_range: None,
    })
}

#[derive( Debug)]
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

        if let Some(range) = param.range {
            let mut minutes = Vec::new();
            for i in range.start..=range.end {
                minutes.push(i);
            }
            return Ok(Minutes { minutes });
        }

        if let Some(minutes) = param.numbers {
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

#[derive( Debug)]
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

        if let Some(range) = param.range {
            return Ok(Hours {
                hours: (range.start..=range.end).collect(),
            });
        }

        if let Some(hours) = param.numbers {
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

#[derive( Debug)]
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

        if let Some(range) = param.range {
            return Ok(DaysOfMonth {
                days: (range.start..=range.end).collect(),
            });
        }

        if let Some(days) = param.numbers {
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

#[derive( Debug)]
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
            return Ok(Self {
                days: (range.start..=range.end).collect(),
            });
        }

        if let Some(days) = param.numbers {
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

#[derive( Debug)]
struct Months {
    months: Vec<i64>,
}

impl Months {
    fn new(param: ParsingResults) -> Result<Months, anyhow::Error> {
        param.validate(1, 12)?;
        if param.wildcard {
            return Ok(Self {
                months: (0..=12).collect(),
            });
        }

        if let Some(range) = param.range {
            return Ok(Self {
                months: (range.start..range.end).collect(),
            });
        }

        if let Some(months) = param.numbers {
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

#[derive( Debug)]
struct Command {
    command: String,
}

impl Command {
    // no validation yet
    fn new(command: String) -> Self {
        Command { command }
    }
}


#[derive( Debug)]
struct Cron {
    minute: Minutes,
    hour: Hours,
    day_of_month: DaysOfMonth,
    day_of_week: DaysOfWeek,
    month: Months,
    command: Command,
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
    let example = "*/15 0 1,15 * 1-5 /usr/bin/find";
    let result = parse(example.to_string());
    dbg!(result);
}
