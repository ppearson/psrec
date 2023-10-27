/*
 psrec
 Copyright 2022-2023 Peter Pearson.
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

// returns an Option<> tuple of the u64 value in ms, plus a human-readable
// string representation of the number with units
pub fn convert_time_period_string_to_ms(str_val: &str) -> Option<(u64, String)> {
    // TODO: there's probably a better way of doing this...
    if str_val.is_empty() {
        return None;
    }
    let mut local_value = str_val.to_string();

    let mut unit = String::new();

    for chr in str_val.chars().rev() {
        if chr.is_alphabetic() {
            // if it's not a number, it's hopefully for the units, so either one or two chars...
            // so pop of the last char of the unreversed local copy...
            local_value.pop();
            // and accumulate the chr
            unit.push(chr);
        }
    }

    // need to reverse the unit in case it was "ms"
    unit = unit.chars().rev().collect::<String>();

    let value_was_unitless = unit.is_empty();
    let parse_result = local_value.parse::<u64>();
    if parse_result.is_err() {
        return None;
    }
    let parse_result = parse_result.unwrap();

    if parse_result == 0 {
        return None;
    }

    let human_readable_string;

    let mut mult_to_ms = 1;
    if !value_was_unitless {
        if unit == "ms" {
            human_readable_string = format!("{} {}", parse_result, "ms");
        }
        else if unit == "s" {
            mult_to_ms *= 1000;
            human_readable_string = format!("{} {}", parse_result, if parse_result == 1 { "sec" } else { "secs"});
        }
        else if unit == "m" {
            mult_to_ms *= 60 * 1000;
            human_readable_string = format!("{} {}", parse_result, if parse_result == 1 { "min" } else { "mins"});
        }
        else if unit == "h" {
            mult_to_ms *= 60 * 60 * 1000;
            human_readable_string = format!("{} {}", parse_result, if parse_result == 1 { "hour" } else { "hours"});
        }
        else {
            // otherwise, we don't know what the units are, so return None as an error
            return None;
        }
    }
    else {
        // assume the units are seconds
        mult_to_ms = 1000;
        human_readable_string = format!("{} {}", parse_result, if parse_result == 1 { "sec" } else { "secs"});
    }

    let final_time_in_ms = parse_result * mult_to_ms;

    Some((final_time_in_ms, human_readable_string))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_time_period_string_01_ms() {
        assert_eq!(convert_time_period_string_to_ms("42ms"), Some((42, "42 ms".into())));

        assert_eq!(convert_time_period_string_to_ms("100ms"), Some((100, "100 ms".into())));
    }

    #[test]
    fn test_convert_time_period_string_02_secs() {
        assert_eq!(convert_time_period_string_to_ms("42"), Some((42 * 1000, "42 secs".into())));

        assert_eq!(convert_time_period_string_to_ms("42s"), Some((42 * 1000, "42 secs".into())));

        assert_eq!(convert_time_period_string_to_ms("1"), Some((1 * 1000, "1 sec".into())));
        assert_eq!(convert_time_period_string_to_ms("5"), Some((5 * 1000, "5 secs".into())));
    }

    #[test]
    fn test_convert_time_period_string_03_mins() {
        assert_eq!(convert_time_period_string_to_ms("1m"), Some((60 * 1000, "1 min".into())));
        assert_eq!(convert_time_period_string_to_ms("3m"), Some((180 * 1000, "3 mins".into())));
    }

    #[test]
    fn test_convert_time_period_string_04_hours() {
        assert_eq!(convert_time_period_string_to_ms("1h"), Some((60 * 60 * 1000, "1 hour".into())));
        assert_eq!(convert_time_period_string_to_ms("3h"), Some((180 * 60 * 1000, "3 hours".into())));
    }

    #[test]
    fn test_convert_time_period_string_05_invalid() {
        assert_eq!(convert_time_period_string_to_ms("h"), None);
        assert_eq!(convert_time_period_string_to_ms(""), None);
        assert_eq!(convert_time_period_string_to_ms("ms"), None);
        assert_eq!(convert_time_period_string_to_ms("3345nk"), None);
    }
}
