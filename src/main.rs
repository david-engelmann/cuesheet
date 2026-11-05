//! Command line entry point, the cuesheet binary.
//!
//! Subcommands
//!   sheet    print the sheet of one catalogue and context
//!   gaps     print the gaps view of one catalogue and context
//!
//! The argument parsing is clap, adopted under the
//! maintainer's tool choice ruling. Before clap sees the
//! arguments, a scan walks them the way the Python parser's
//! own scan walked them. It fires help where that scan fired
//! it, and where the invocation is clean it runs the command
//! itself on the slots the separator distribution assigns,
//! because the Python parser's separator reading has no clap
//! counterpart. Where the scan cannot assign the invocation
//! it hands the arguments to clap as they stand, since clap
//! refuses the same invocation in its own voice. Help text
//! and malformed invocation error text are clap's own forms.
//! The documents the tool emits and the one error line
//! discipline for its own refusals are unchanged. Cuesheet
//! reads the two files it is handed and prints a document,
//! and it never executes a cue it lists.

use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use anyhow::Error as AnyError;
use clap::{Args, CommandFactory, Parser, Subcommand};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

use cuesheet::argv::{Cmd, Scan, Slot, scan};
use cuesheet::catalogue::{Catalogue, parse_catalogue};
use cuesheet::context::{Context, parse_context};
use cuesheet::errors::CuesheetError;
use cuesheet::paths::py_path_normalized;
use cuesheet::pytext::PyStr;
use cuesheet::sheet::{build_gaps, build_sheet, format_gaps, format_sheet};

#[derive(Parser)]
#[command(
    name = "cuesheet",
    about = "Lists the actions available to an agent at one decision point, in context. Executes nothing.",
    disable_version_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(name = "sheet", infer_long_args = true)]
    Sheet(SheetArgs),
    #[command(name = "gaps", infer_long_args = true)]
    Gaps(GapsArgs),
}

#[derive(Args)]
struct SheetArgs {
    catalogue: OsString,
    context: OsString,
}

#[derive(Args)]
struct GapsArgs {
    catalogue: OsString,
    context: OsString,
}

/// The size of the Python tool's standard output buffer. A
/// print whose failure surfaces while the output still fits
/// the buffer is the failed flush family, and one that
/// surfaces at or past it is the uncaught family (CU-29).
const STDOUT_BUFFER_BYTES: usize = 8192;

/// The port's own line for a print that failed inside the
/// buffer, standing where the Python tool's interpreter
/// printed its exception ignored noise.
const FAILED_PRINT_LINE: &str = "cuesheet could not print its document";

/// The port's own line for a print that failed at or past the
/// buffer, standing where the Python tool printed a traceback.
const UNCAUGHT_PRINT_LINE: &str = "cuesheet: the document print failed";

/// The port's own line for a list standing in a slot a command
/// reads as one path, standing where the Python tool printed
/// a traceback for its type error (CU-30).
const LIST_PATH_LINE: &str =
    "cuesheet: an argument arrived as a list where a single path was expected";

/// A print failure discovered while the output still fit the
/// standard output buffer.
#[derive(Debug)]
struct FailedFlush(String);

impl std::fmt::Display for FailedFlush {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for FailedFlush {}

/// A print failure discovered at or past the buffer.
#[derive(Debug)]
struct UncaughtPrint(String);

impl std::fmt::Display for UncaughtPrint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for UncaughtPrint {}

/// Print one document to standard output through one checked
/// write, as the Python tool prints through its buffered
/// write and its exit flush.
fn print_out(text: &str) -> anyhow::Result<()> {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    lock.write_all(text.as_bytes())
        .and_then(|()| lock.flush())
        .map_err(|_| {
            if text.len() < STDOUT_BUFFER_BYTES {
                AnyError::new(FailedFlush(FAILED_PRINT_LINE.to_string()))
            } else {
                AnyError::new(UncaughtPrint(UNCAUGHT_PRINT_LINE.to_string()))
            }
        })
}

/// Write one error line to the error stream through one
/// checked write, and answer with the given status, or with
/// status 120 when the write itself fails. The text goes out
/// in the display form of the message, which is the form the
/// Python tool's error stream, with its backslashreplace
/// handler, puts on the wire.
fn answer_error_line(text: &PyStr, status: u8) -> ExitCode {
    let stderr = std::io::stderr();
    let mut lock = stderr.lock();
    match lock.write_all(text.to_display().as_bytes()) {
        Ok(()) => ExitCode::from(status),
        Err(_) => ExitCode::from(120),
    }
}

/// Print a document and answer for its print.
fn print_document(text: &str) -> ExitCode {
    match print_out(text) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => answer_print_failure(failure),
    }
}

/// Answer a failed document print.
fn answer_print_failure(failure: AnyError) -> ExitCode {
    if let Some(failed) = failure.downcast_ref::<FailedFlush>() {
        let mut line = PyStr::from_str(&failed.0);
        line.push_str("\n");
        answer_error_line(&line, 120)
    } else if let Some(uncaught) = failure.downcast_ref::<UncaughtPrint>() {
        let mut line = PyStr::from_str(&uncaught.0);
        line.push_str("\n");
        answer_error_line(&line, 1)
    } else {
        let mut line = PyStr::from_str("error: ");
        line.push_str(&failure.to_string());
        line.push_str("\n");
        answer_error_line(&line, 2)
    }
}

/// Answer a library refusal with its one error line at
/// status 2.
fn answer_refusal(error: &CuesheetError) -> ExitCode {
    let mut line = PyStr::from_str("error: ");
    line.push_pystr(&error.message());
    line.push_str("\n");
    answer_error_line(&line, 2)
}

/// Answer a clap error the way the command line answers it.
///
/// An error clap prints to standard error, a malformed
/// invocation, is printed there and answered with its own
/// exit code, and it goes through the same handled error
/// stream write as the tool's own lines, so an error stream
/// that cannot take the text answers with status 120. An
/// error clap prints to standard output is the help text, and
/// help output takes the failed write discipline of every
/// document the tool prints, because the Python tool's help
/// goes to the same buffered standard output its documents
/// do.
fn answer_clap_error(error: clap::Error) -> ExitCode {
    if error.use_stderr() {
        let code = error.exit_code();
        let text = error.render().to_string();
        let text = if text.ends_with('\n') {
            text
        } else {
            format!("{text}\n")
        };
        return answer_error_line(
            &PyStr::from_str(&text),
            if code == 0 { 2 } else { code as u8 },
        );
    }
    let text = error.render().to_string();
    print_document(&text)
}

/// Read one slot's file as text, answering a failure on the
/// spot. The read opens the normalized form of the path as
/// written, as the oracle's path object normalizes before it
/// opens. The cannot read line names the path as written, and
/// a body that is not UTF-8 text folds into the same refusal.
fn read_slot(slot: &Slot, what: &'static str) -> Result<(String, PyStr), ExitCode> {
    let raw = match slot {
        Slot::Listed(_) => {
            let mut line = PyStr::from_str(LIST_PATH_LINE);
            line.push_str("\n");
            return Err(answer_error_line(&line, 1));
        }
        Slot::One(raw) => raw,
    };
    let normalized = py_path_normalized(raw);
    let content = match read_bytes(&normalized) {
        Ok(content) => content,
        Err(_) => {
            return Err(answer_refusal(&CuesheetError::CannotRead {
                what,
                path: raw.clone(),
            }));
        }
    };
    match String::from_utf8(content) {
        Ok(text) => Ok((text, raw.clone())),
        Err(_) => Err(answer_refusal(&CuesheetError::CannotRead {
            what,
            path: raw.clone(),
        })),
    }
}

/// Read the bytes of one normalized path.
fn read_bytes(path: &PyStr) -> std::io::Result<Vec<u8>> {
    let bytes = path.to_fsencode_bytes();
    #[cfg(unix)]
    {
        std::fs::read(std::ffi::OsStr::from_bytes(&bytes))
    }
    #[cfg(not(unix))]
    {
        std::fs::read(String::from_utf8_lossy(&bytes).into_owned())
    }
}

/// Run one assigned command on its slots, in the read order
/// the Python command reads: the catalogue reads and parses
/// first, then the context reads and parses against it, and
/// the list in the context slot surfaces only at its read,
/// after the catalogue has parsed clean (CU-30).
fn run_assigned(cmd: Cmd, catalogue_slot: &Slot, context_slot: &Slot) -> ExitCode {
    let (catalogue_text, catalogue_source) = match read_slot(catalogue_slot, "catalogue") {
        Ok(pair) => pair,
        Err(code) => return code,
    };
    let catalogue: Catalogue = match parse_catalogue(&catalogue_text, &catalogue_source) {
        Ok(catalogue) => catalogue,
        Err(error) => return answer_refusal(&error),
    };
    let (context_text, context_source) = match read_slot(context_slot, "context") {
        Ok(pair) => pair,
        Err(code) => return code,
    };
    let context: Context = match parse_context(&context_text, &context_source, &catalogue) {
        Ok(context) => context,
        Err(error) => return answer_refusal(&error),
    };
    let document = match cmd {
        Cmd::Sheet => format_sheet(&context, &build_sheet(&catalogue, &context)),
        Cmd::Gaps => format_gaps(&context, &build_gaps(&catalogue, &context)),
    };
    print_document(&document)
}

fn run() -> ExitCode {
    let mut args = std::env::args_os();
    let program = args.next().unwrap_or_else(|| OsString::from("cuesheet"));
    let rest: Vec<OsString> = args.collect();
    let tokens: Vec<PyStr> = rest
        .iter()
        .map(|arg| PyStr::from_surrogateescape(&arg_bytes(arg)))
        .collect();
    match scan(&tokens) {
        Scan::FireHelp(cmd) => {
            let mut argv = vec![program];
            if let Some(cmd) = cmd {
                argv.push(OsString::from(cmd.name()));
            }
            argv.push(OsString::from("--help"));
            match Cli::try_parse_from(argv) {
                Ok(_) => ExitCode::from(0),
                Err(error) => answer_clap_error(error),
            }
        }
        Scan::Assigned {
            cmd,
            catalogue,
            context,
        } => run_assigned(cmd, &catalogue, &context),
        Scan::HandToClap { replacement } => {
            let mut handed: Vec<OsString> = rest;
            if let Some((index, argument)) = replacement {
                let mut bytes = b"--help=".to_vec();
                bytes.extend_from_slice(&argument.to_fsencode_bytes());
                if index < handed.len() {
                    handed[index] = os_string_from_bytes(bytes);
                }
            }
            match Cli::try_parse_from(std::iter::once(program).chain(handed)) {
                Ok(_) => {
                    // The scan could not assign the invocation,
                    // so the Python parser refused it. Clap's
                    // acceptance of the arguments as they stand
                    // is answered with a usage refusal, since
                    // the functional outcome is a refusal either
                    // way.
                    let usage = Cli::command().render_usage().to_string();
                    let mut line = PyStr::from_str("error: the invocation does not parse\n");
                    line.push_str(&usage);
                    line.push_str("\n");
                    answer_error_line(&line, 2)
                }
                Err(error) => answer_clap_error(error),
            }
        }
    }
}

/// Build an argument string from raw bytes.
fn os_string_from_bytes(bytes: Vec<u8>) -> OsString {
    #[cfg(unix)]
    {
        std::os::unix::ffi::OsStringExt::from_vec(bytes)
    }
    #[cfg(not(unix))]
    {
        OsString::from(String::from_utf8_lossy(&bytes).into_owned())
    }
}

/// The raw bytes of one command line argument.
fn arg_bytes(arg: &OsString) -> Vec<u8> {
    #[cfg(unix)]
    {
        arg.as_os_str().as_bytes().to_vec()
    }
    #[cfg(not(unix))]
    {
        // On a platform without byte arguments the lossy form
        // is the only form there is; the fleet runs on Unix.
        arg.to_string_lossy().into_owned().into_bytes()
    }
}

fn main() -> ExitCode {
    run()
}
