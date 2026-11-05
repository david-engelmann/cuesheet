//! cuesheet, the library. It lists the actions an agent
//! could take right now, in context, and never executes one.
//!
//! The library carries the contracted behaviour: the
//! catalogue and context formats and their refusals, the one
//! shared evaluation the sheet and the gaps view are both
//! computed from, the path spelling a read opens, the Python
//! text discipline the formats are stated in, and the
//! argument scan that walks an invocation the way the oracle
//! parser walked it before clap ever sees it.

pub mod argv;
pub mod catalogue;
pub mod context;
pub mod errors;
pub mod lines;
pub mod ndtable;
pub mod paths;
pub mod pytext;
pub mod sheet;
