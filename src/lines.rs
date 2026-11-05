//! The one line discipline every cuesheet format shares.

use crate::pytext::py_strip;

/// The stripped, non-blank lines of a recorded text, in order.
/// The text is split at newline characters only, each line is
/// stripped of Python whitespace at both ends, and a line that
/// strips to nothing is dropped.
pub fn meaningful_lines(text: &str) -> Vec<&str> {
    text.split('\n')
        .map(py_strip)
        .filter(|line| !line.is_empty())
        .collect()
}
