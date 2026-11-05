//! The refusal type of the crate, mirroring the oracle's
//! CuesheetError, of which the catalogue and context errors are
//! the two families. Every variant is a refusal the command
//! answers with one error line and exit code 2, and the text a
//! variant carries is the whole message after the error prefix.

use thiserror::Error;

use crate::pytext::PyStr;

#[derive(Debug, Error)]
pub enum CuesheetError {
    /// A file handed to the tool cannot be read, whether the
    /// operating system refuses the open or the bytes are not
    /// UTF-8 text. The line names the path as written.
    #[error("the {what} file {path} cannot be read")]
    CannotRead { what: &'static str, path: PyStr },

    /// A catalogue or a context is malformed. The message is
    /// the full oracle wording, source prefix included.
    #[error("{0}")]
    Malformed(PyStr),
}

impl CuesheetError {
    /// The full message of the refusal, as it stands after the
    /// error prefix on the refusal line.
    pub fn message(&self) -> PyStr {
        PyStr::from_str(&self.to_string())
    }
}

/// Build a malformed refusal from a formatted message.
pub fn malformed(message: String) -> CuesheetError {
    CuesheetError::Malformed(PyStr::from_str(&message))
}
