//! Python text behaviour the crate's surface depends on.
//!
//! Cuesheet's formats are defined in terms of Python string
//! operations, and its command line receives Python strings the
//! interpreter decoded with the surrogateescape error handler,
//! so the crate carries a string type that holds a Python string
//! exactly, as the sequence of its code points, lone surrogates
//! included. A command line argument can produce a lone
//! surrogate from a byte that cannot stand in UTF-8, and the
//! surrogate is observable in the refusal lines the tool prints,
//! so it may not be collapsed to a replacement character.
//! Ordering of the type is code point order, which is Python's
//! string order.

use std::fmt;

use crate::ndtable::is_decimal_digit;

/// A Python string, held as the sequence of its code points.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PyStr(pub Vec<u32>);

impl PyStr {
    /// The string a Rust string holds, code point for code point.
    /// The name mirrors the oracle's constructor on purpose. It
    /// is an infallible code point copy, not a parse, so the
    /// fallible FromStr trait is not the shape it should carry.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(text: &str) -> PyStr {
        PyStr(text.chars().map(|ch| ch as u32).collect())
    }

    /// The string Python's argument decoding produces from raw
    /// bytes, where a byte that cannot stand in UTF-8 becomes the
    /// surrogate for that byte alone and decoding resumes at the
    /// next byte.
    pub fn from_surrogateescape(bytes: &[u8]) -> PyStr {
        let mut points = Vec::new();
        let mut index = 0;
        while index < bytes.len() {
            let lead = bytes[index];
            if lead < 0x80 {
                points.push(lead as u32);
                index += 1;
                continue;
            }
            let width = match lead {
                0xC2..=0xDF => 2,
                0xE0..=0xEF => 3,
                0xF0..=0xF4 => 4,
                _ => 0,
            };
            let mut decoded = None;
            if width > 0 && index + width <= bytes.len() {
                let rest = &bytes[index + 1..index + width];
                if rest.iter().all(|byte| (0x80..=0xBF).contains(byte)) {
                    let second = rest[0];
                    let in_range = match lead {
                        0xE0 => (0xA0..=0xBF).contains(&second),
                        0xED => (0x80..=0x9F).contains(&second),
                        0xF0 => (0x90..=0xBF).contains(&second),
                        0xF4 => (0x80..=0x8F).contains(&second),
                        _ => true,
                    };
                    if in_range {
                        let mut point = (lead as u32) & (0x7F >> width);
                        for byte in rest {
                            point = (point << 6) | (*byte as u32 & 0x3F);
                        }
                        decoded = Some(point);
                    }
                }
            }
            match decoded {
                Some(point) => {
                    points.push(point);
                    index += width;
                }
                None => {
                    points.push(0xDC00 + lead as u32);
                    index += 1;
                }
            }
        }
        PyStr(points)
    }

    /// The number of code points in the string.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the string is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Whether this string is exactly the given Rust string.
    pub fn eq_str(&self, text: &str) -> bool {
        self.0.iter().copied().eq(text.chars().map(|ch| ch as u32))
    }

    /// Whether this string opens with the given Rust string.
    pub fn starts_with_str(&self, text: &str) -> bool {
        let mut points = self.0.iter();
        for ch in text.chars() {
            if points.next() != Some(&(ch as u32)) {
                return false;
            }
        }
        true
    }

    /// Append the code points of a Rust string.
    pub fn push_str(&mut self, text: &str) {
        self.0.extend(text.chars().map(|ch| ch as u32));
    }

    /// Append another Python string.
    pub fn push_pystr(&mut self, other: &PyStr) {
        self.0.extend_from_slice(&other.0);
    }

    /// The bytes the operating system is handed for this string
    /// as a path, where Python encodes a surrogateescape code
    /// point back to the byte it came from and every other code
    /// point as its UTF-8 form.
    pub fn to_fsencode_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for &point in &self.0 {
            if (0xDC00..=0xDCFF).contains(&point) {
                out.push((point - 0xDC00) as u8);
            } else if let Some(ch) = char::from_u32(point) {
                let mut buffer = [0u8; 4];
                out.extend_from_slice(ch.encode_utf8(&mut buffer).as_bytes());
            }
        }
        out
    }

    /// The string as it appears when Python writes it to its
    /// standard error, whose backslashreplace handler turns every
    /// lone surrogate into a backslash, the letter u, and four
    /// lowercase hex digits. Every refusal line the tool prints
    /// is such a write.
    pub fn to_display(&self) -> String {
        let mut out = String::new();
        for &point in &self.0 {
            if (0xD800..=0xDFFF).contains(&point) {
                out.push_str(&format!("\\u{point:04x}"));
            } else if let Some(ch) = char::from_u32(point) {
                out.push(ch);
            }
        }
        out
    }
}

impl fmt::Display for PyStr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_display())
    }
}

/// Whether the code point is whitespace to Python. The set is
/// the Unicode White_Space property, which Rust's char carries
/// as well, plus the four separator controls U+001C to U+001F
/// that Python also reads as whitespace.
pub fn is_py_space(point: u32) -> bool {
    if (0x1C..=0x1F).contains(&point) {
        return true;
    }
    match char::from_u32(point) {
        Some(ch) => ch.is_whitespace(),
        None => false,
    }
}

/// Strip Python whitespace from both ends of a line of text.
pub fn py_strip(text: &str) -> &str {
    let start = text
        .char_indices()
        .find(|(_, ch)| !is_py_space(*ch as u32))
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    let end = text
        .char_indices()
        .rev()
        .find(|(_, ch)| !is_py_space(*ch as u32))
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(0);
    if start >= end { "" } else { &text[start..end] }
}

/// Python's split of a string on whitespace runs, which drops
/// every empty piece, leading and trailing runs included.
pub fn py_split(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = text;
    loop {
        let trimmed = rest.trim_start_matches(|ch| is_py_space(ch as u32));
        if trimmed.is_empty() {
            return parts;
        }
        let end = trimmed
            .char_indices()
            .find(|(_, ch)| is_py_space(*ch as u32))
            .map(|(index, _)| index)
            .unwrap_or(trimmed.len());
        parts.push(&trimmed[..end]);
        rest = &trimmed[end..];
    }
}

/// The remainder of a line after its first whitespace run, the
/// second piece of Python's split of the line at most once on
/// whitespace. The caller has already established the line has
/// a second piece.
pub fn after_first_word(line: &str) -> &str {
    let after_word = match line.char_indices().find(|(_, ch)| is_py_space(*ch as u32)) {
        Some((index, _)) => &line[index..],
        None => return "",
    };
    after_word.trim_start_matches(|ch| is_py_space(ch as u32))
}

/// Whether the token matches the Python parser's negative
/// number pattern, which is a leading minus and then either
/// decimal digits alone or decimal digits around one dot, the
/// whole anchored at both ends, with the end anchor also
/// standing just before one trailing newline. A decimal digit
/// is any code point of Unicode category Nd.
pub fn is_negative_number(points: &[u32]) -> bool {
    let points = match points.last() {
        Some(0x0A) => &points[..points.len() - 1],
        _ => points,
    };
    let Some((&0x2D, rest)) = points.split_first() else {
        return false;
    };
    if rest.is_empty() {
        return false;
    }
    if rest.iter().all(|&point| is_decimal_digit(point)) {
        return true;
    }
    let Some(dot) = rest.iter().position(|&point| point == 0x2E) else {
        return false;
    };
    let (head, tail) = (&rest[..dot], &rest[dot + 1..]);
    !tail.is_empty()
        && head.iter().all(|&point| is_decimal_digit(point))
        && tail.iter().all(|&point| is_decimal_digit(point))
}
