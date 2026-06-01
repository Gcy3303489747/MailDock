use base64::{engine::general_purpose, Engine as _};
use encoding_rs::Encoding;

pub(crate) fn normalize_display_text(value: impl AsRef<str>) -> String {
    repair_utf8_decoded_as_gbk(&clean_text(
        decode_rfc2047_words(value.as_ref()).unwrap_or_else(|| value.as_ref().to_owned()),
    ))
}

pub(crate) fn normalize_body_text(value: impl AsRef<str>) -> String {
    let repaired = repair_utf8_decoded_as_gbk(value.as_ref());
    let normalized = repaired.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = Vec::new();
    let mut blank_count = 0;

    for line in normalized.lines() {
        let trimmed_end = line.trim_end();

        if trimmed_end.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 && !lines.is_empty() {
                lines.push(String::new());
            }
            continue;
        }

        blank_count = 0;
        lines.push(trimmed_end.to_owned());
    }

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    lines.join("\n").trim().to_owned()
}

pub(crate) fn clean_text(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .replace("\r\n", "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_rfc2047_words(value: &str) -> Option<String> {
    let mut output = String::new();
    let mut remainder = value;
    let mut decoded_any = false;
    let mut previous_was_encoded = false;

    while let Some(start) = remainder.find("=?") {
        let before = &remainder[..start];
        if !(previous_was_encoded && before.trim().is_empty()) {
            output.push_str(before);
        }

        let encoded_remainder = &remainder[start + 2..];
        let Some(first_separator) = encoded_remainder.find('?') else {
            output.push_str(&remainder[start..]);
            return decoded_any.then_some(output);
        };
        let after_first_separator = first_separator + 1;
        let Some(second_separator_offset) = encoded_remainder[after_first_separator..].find('?')
        else {
            output.push_str(&remainder[start..]);
            return decoded_any.then_some(output);
        };
        let second_separator = after_first_separator + second_separator_offset;
        let payload_start = second_separator + 1;
        let Some(end_offset) = encoded_remainder[payload_start..].find("?=") else {
            output.push_str(&remainder[start..]);
            return decoded_any.then_some(output);
        };
        let end = payload_start + end_offset;

        let encoded_word = &encoded_remainder[..end];
        if let Some(decoded_word) = decode_rfc2047_word(encoded_word) {
            output.push_str(&decoded_word);
            decoded_any = true;
            previous_was_encoded = true;
        } else {
            output.push_str("=?");
            output.push_str(encoded_word);
            output.push_str("?=");
            previous_was_encoded = false;
        }

        remainder = &encoded_remainder[end + 2..];
    }

    output.push_str(remainder);
    decoded_any.then_some(output)
}

fn decode_rfc2047_word(encoded_word: &str) -> Option<String> {
    let mut parts = encoded_word.splitn(3, '?');
    let charset = parts.next()?.trim();
    let encoding = parts.next()?.trim();
    let payload = parts.next()?.trim();

    let bytes = match encoding.to_ascii_lowercase().as_str() {
        "b" => decode_base64_payload(payload)?,
        "q" => decode_rfc2047_q_payload(payload)?,
        _ => return None,
    };

    decode_charset(charset, &bytes)
}

fn decode_base64_payload(payload: &str) -> Option<Vec<u8>> {
    let cleaned = payload
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    if cleaned.is_empty() {
        return Some(Vec::new());
    }

    general_purpose::STANDARD
        .decode(&cleaned)
        .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(&cleaned))
        .or_else(|_| {
            let padding = (4 - cleaned.len() % 4) % 4;
            let mut padded = cleaned.clone();
            padded.extend(std::iter::repeat('=').take(padding));
            general_purpose::STANDARD.decode(padded)
        })
        .ok()
}

fn decode_rfc2047_q_payload(payload: &str) -> Option<Vec<u8>> {
    let bytes = payload.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            byte if byte.is_ascii_whitespace() => {
                index += 1;
            }
            b'_' => {
                decoded.push(b' ');
                index += 1;
            }
            b'=' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
                decoded.push(u8::from_str_radix(hex, 16).ok()?);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }

    Some(decoded)
}

fn decode_charset(charset: &str, bytes: &[u8]) -> Option<String> {
    if charset.eq_ignore_ascii_case("utf-8") || charset.eq_ignore_ascii_case("utf8") {
        return String::from_utf8(bytes.to_vec()).ok();
    }

    let encoding = Encoding::for_label(charset.as_bytes())?;
    let (decoded, _, had_errors) = encoding.decode(bytes);

    if had_errors {
        return None;
    }

    Some(decoded.into_owned())
}

fn repair_utf8_decoded_as_gbk(value: &str) -> String {
    let (encoded, _, had_errors) = encoding_rs::GBK.encode(value);
    if had_errors {
        return value.to_owned();
    }

    String::from_utf8(encoded.into_owned()).unwrap_or_else(|_| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_mixed_plain_text_and_q_encoded_header() {
        let value = "www.donkX.net邮箱验=?utf-8?Q?=E8=AF=81=E7=A0=81?=";

        assert_eq!(normalize_display_text(value), "www.donkX.net邮箱验证码");
    }

    #[test]
    fn decodes_q_encoded_header_with_folded_whitespace() {
        let value = "www.donkX.net邮箱验=?utf-8?Q?=E8=AF=81\r\n =E7=A0=81?=";

        assert_eq!(normalize_display_text(value), "www.donkX.net邮箱验证码");
    }
}
