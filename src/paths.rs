//! The spelling a read opens, which is the pathlib normalized
//! spelling of the path as written, while every refusal line
//! names the path as written. Normalization splits the path at
//! its slashes, drops empty components and dot components and
//! so a trailing slash as well, keeps parent components for the
//! operating system to resolve, keeps a leading slash anchor
//! and an exactly double leading slash as the anchor, and reads
//! an empty result with no anchor as the single dot.

use crate::pytext::PyStr;

const SLASH: u32 = 0x2F;
const DOT: u32 = 0x2E;

/// The normalized spelling of a path, as Python's pathlib
/// computes it for a POSIX path before the open.
pub fn py_path_normalized(path: &PyStr) -> PyStr {
    let points = &path.0;
    let anchor: &[u32] = if points.len() >= 2
        && points[0] == SLASH
        && points[1] == SLASH
        && (points.len() == 2 || points[2] != SLASH)
    {
        &[SLASH, SLASH]
    } else if points.first() == Some(&SLASH) {
        &[SLASH]
    } else {
        &[]
    };
    let mut parts: Vec<&[u32]> = Vec::new();
    for part in points.split(|&point| point == SLASH) {
        if part.is_empty() || part == [DOT] {
            continue;
        }
        parts.push(part);
    }
    if parts.is_empty() {
        if anchor.is_empty() {
            return PyStr(vec![DOT]);
        }
        return PyStr(anchor.to_vec());
    }
    let mut out: Vec<u32> = anchor.to_vec();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(SLASH);
        }
        out.extend_from_slice(part);
    }
    PyStr(out)
}
