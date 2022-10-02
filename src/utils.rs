/*
 psrec
 Copyright 2022 Peter Pearson.
 Licensed under the Apache License, Version 2.0 (the "License");
 You may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 http://www.apache.org/licenses/LICENSE-2.0
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
 ---------
*/

// returns an Option<> tuple of the u64 value in seconds, plus a human-readable
// string representation of the number with units
pub fn convert_time_period_string_to_seconds(str_val: &str) -> Option<(u64, String)> {
    // TODO: there's probably a better way of doing this...
    let mut local_value = str_val.to_string();
    if str_val.is_empty() {
        return None;
    }
    let last_char = str_val.chars().last().unwrap();
    let value_was_unitless = !last_char.is_alphabetic();

    if !value_was_unitless {
        local_value.pop();
    }
    let parse_result = local_value.parse::<u64>();
    if parse_result.is_err() {
        return None;
    }
    let parse_result = parse_result.unwrap();

    if parse_result == 0 {
        return None;
    }

    let mut human_readable_string = String::new();

    let mut mult_to_seconds = 1;
    if !value_was_unitless {
        if last_char == 's' {
            human_readable_string = format!("{} {}", parse_result, if parse_result == 1 { "sec" } else { "secs"});
        }
        else if last_char == 'm' {
            mult_to_seconds = 60;
            human_readable_string = format!("{} {}", parse_result, if parse_result == 1 { "min" } else { "mins"});
        }
        else if last_char == 'h' {
            mult_to_seconds = 60 * 60;
            human_readable_string = format!("{} {}", parse_result, if parse_result == 1 { "hour" } else { "hours"});
        }
    }
    else {
        // assume the units are seconds
        human_readable_string = format!("{} {}", parse_result, if parse_result == 1 { "sec" } else { "secs"});
    }

    let final_time_in_secs = parse_result * mult_to_seconds;

    return Some((final_time_in_secs, human_readable_string));
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_time_period_string_01_secs() {
        assert_eq!(convert_time_period_string_to_seconds("42"), Some((42, "42 secs".into())));

        assert_eq!(convert_time_period_string_to_seconds("42s"), Some((42, "42 secs".into())));

        assert_eq!(convert_time_period_string_to_seconds("1"), Some((1, "1 sec".into())));
        assert_eq!(convert_time_period_string_to_seconds("5"), Some((5, "5 secs".into())));
    }

    #[test]
    fn test_convert_time_period_string_02_mins() {
        assert_eq!(convert_time_period_string_to_seconds("1m"), Some((60, "1 min".into())));
        assert_eq!(convert_time_period_string_to_seconds("3m"), Some((180, "3 mins".into())));
    }

    #[test]
    fn test_convert_time_period_string_03_hours() {
        assert_eq!(convert_time_period_string_to_seconds("1h"), Some((60 * 60, "1 hour".into())));
        assert_eq!(convert_time_period_string_to_seconds("3h"), Some((180 * 60, "3 hours".into())));
    }

    #[test]
    fn test_convert_time_period_string_04_invalid() {
        assert_eq!(convert_time_period_string_to_seconds("h"), None);
        assert_eq!(convert_time_period_string_to_seconds(""), None);
    }
}