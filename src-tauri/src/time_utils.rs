use mailparse::dateparse;

pub(crate) fn received_at_sort_key(value: &str) -> i64 {
    dateparse(value)
        .ok()
        .or_else(|| parse_rfc3339_timestamp(value))
        .unwrap_or(0)
}

fn parse_rfc3339_timestamp(value: &str) -> Option<i64> {
    if value.len() < 20 {
        return None;
    }

    let year = parse_i64(value, 0, 4)?;
    let month = parse_i64(value, 5, 7)?;
    let day = parse_i64(value, 8, 10)?;
    let hour = parse_i64(value, 11, 13)?;
    let minute = parse_i64(value, 14, 16)?;
    let second = parse_i64(value, 17, 19)?;

    let timezone_start = value[19..]
        .find(|character| matches!(character, 'Z' | '+' | '-'))
        .map(|index| index + 19)?;
    let timezone = &value[timezone_start..];
    let offset_seconds = if timezone == "Z" {
        0
    } else {
        let sign = match timezone.as_bytes().first().copied()? {
            b'+' => 1,
            b'-' => -1,
            _ => return None,
        };
        let offset_hour = parse_i64(timezone, 1, 3)?;
        let offset_minute = parse_i64(timezone, 4, 6)?;
        sign * (offset_hour * 3600 + offset_minute * 60)
    };

    Some(
        days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second
            - offset_seconds,
    )
}

fn parse_i64(value: &str, start: usize, end: usize) -> Option<i64> {
    value.get(start..end)?.parse().ok()
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rfc2822_and_rfc3339_for_sorting() {
        assert!(
            received_at_sort_key("Fri, 29 May 2026 09:30:00 +0800")
                > received_at_sort_key("2026-05-29T08:00:00+08:00")
        );
    }
}
