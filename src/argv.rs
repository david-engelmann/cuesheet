//! The argument scan. Before clap sees the arguments, a scan
//! walks them the way the Python parser's own scan walked
//! them, CPython's argparse as the oracle's cli builds it: a
//! root parser whose one positional is the subcommand, and
//! one subparser per command with two positional paths. The
//! scan fires help where that scan fired it, runs the command
//! itself on the slots the separator distribution assigns,
//! and hands every other invocation to clap, since clap
//! refuses the same invocation in its own voice. Help text
//! and malformed invocation error text are clap's own forms;
//! every functional outcome of the walk is the oracle's.
//!
//! The distribution the walk mirrors, from the oracle's
//! parser: the pattern of the token stream carries A for an
//! argument, O for an optional, and a dash for the FIRST
//! separator token, after which every token is an argument.
//! Positionals fill in chunks split at optionals, each one
//! matching one argument with any number of separators around
//! it, greedily, in the regex order of the original matcher.
//! The slice a positional receives gives up its first
//! separator string before the value is read, so a slice of
//! exactly one separator leaves the value an empty list, and
//! the subcommand positional swallows every remaining token
//! into the subparser's hands.

use std::collections::HashMap;

use crate::pytext::{PyStr, is_negative_number};

/// One subcommand of the tool.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cmd {
    Sheet,
    Gaps,
}

impl Cmd {
    /// The name the command is invoked by.
    pub fn name(self) -> &'static str {
        match self {
            Cmd::Sheet => "sheet",
            Cmd::Gaps => "gaps",
        }
    }
}

/// What one positional slot received from the distribution.
#[derive(Clone, Debug, PartialEq)]
pub enum Slot {
    /// Exactly one value.
    One(PyStr),
    /// A list of values, empty or longer than one, which the
    /// separator family produces and the oracle dies on.
    Listed(Vec<PyStr>),
}

/// The outcome of the scan over one invocation.
pub enum Scan {
    /// Help fires for the root parser or for one subcommand.
    FireHelp(Option<Cmd>),
    /// The invocation assigns cleanly and the command runs on
    /// these slots.
    Assigned {
        cmd: Cmd,
        catalogue: Slot,
        context: Slot,
    },
    /// The scan could not cleanly assign the invocation, and
    /// the arguments go to clap as they stand, except that
    /// the token at the given index, where the walk met an
    /// ignored explicit argument refusal, is first replaced
    /// by the canonical refused form `--help=<argument>`, so
    /// clap refuses the invocation in its own voice rather
    /// than firing the help the token merely resembles.
    HandToClap { replacement: Option<(usize, PyStr)> },
}

/// Scan one invocation, given without the program name.
pub fn scan(tokens: &[PyStr]) -> Scan {
    match parse_root(tokens) {
        Walk::FireHelp(cmd) => Scan::FireHelp(cmd),
        Walk::Assigned {
            cmd,
            catalogue,
            context,
        } => Scan::Assigned {
            cmd,
            catalogue,
            context,
        },
        Walk::Refuse => Scan::HandToClap { replacement: None },
        Walk::RefuseAt { index, argument } => Scan::HandToClap {
            replacement: Some((index, argument)),
        },
    }
}

enum Walk {
    FireHelp(Option<Cmd>),
    Assigned {
        cmd: Cmd,
        catalogue: Slot,
        context: Slot,
    },
    Refuse,
    /// An ignored explicit argument refusal at the token with
    /// this index in the walked stream, carrying the argument.
    RefuseAt {
        index: usize,
        argument: PyStr,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PatChar {
    Arg,
    Opt,
    Dash,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sep {
    Eq,
    Empty,
}

/// One classified optional token, mirroring the tuple the
/// oracle's optional parser returns.
#[derive(Clone)]
struct OptTuple {
    help: bool,
    option_string: PyStr,
    sep: Option<Sep>,
    explicit: Option<PyStr>,
}

impl OptTuple {
    fn help(option_string: PyStr, sep: Option<Sep>, explicit: Option<PyStr>) -> OptTuple {
        OptTuple {
            help: true,
            option_string,
            sep,
            explicit,
        }
    }

    fn no_action(token: &PyStr) -> OptTuple {
        OptTuple {
            help: false,
            option_string: token.clone(),
            sep: None,
            explicit: None,
        }
    }
}

enum Classified {
    Positional,
    Optional(OptTuple),
}

/// Classify one token the way the oracle's optional parser
/// classifies it for a parser whose only optional is help.
fn classify(token: &PyStr) -> Classified {
    let points = &token.0;
    if points.is_empty() || points[0] != 0x2D {
        return Classified::Positional;
    }
    if token.eq_str("-h") {
        return Classified::Optional(OptTuple::help(PyStr::from_str("-h"), None, None));
    }
    if token.eq_str("--help") {
        return Classified::Optional(OptTuple::help(PyStr::from_str("--help"), None, None));
    }
    if points.len() == 1 {
        return Classified::Positional;
    }
    if let Some(eq) = points.iter().position(|&point| point == 0x3D) {
        let head = PyStr(points[..eq].to_vec());
        if head.eq_str("-h") || head.eq_str("--help") {
            return Classified::Optional(OptTuple::help(
                head,
                Some(Sep::Eq),
                Some(PyStr(points[eq + 1..].to_vec())),
            ));
        }
    }
    let tuples = option_tuples(token);
    if tuples.len() == 1 {
        return Classified::Optional(tuples.into_iter().next().expect("one tuple"));
    }
    if !tuples.is_empty() {
        // Ambiguity cannot arise: the parser carries one long
        // option, so no token prefixes two of them.
        return Classified::Optional(tuples.into_iter().next().expect("a tuple"));
    }
    if is_negative_number(points) {
        return Classified::Positional;
    }
    if points.contains(&0x20) {
        return Classified::Positional;
    }
    Classified::Optional(OptTuple::no_action(token))
}

/// The option tuples of one token, mirroring the oracle's
/// segmentation of a token that is not an exact option string
/// and carries no recognized head before an equals sign.
fn option_tuples(token: &PyStr) -> Vec<OptTuple> {
    let points = &token.0;
    let mut out = Vec::new();
    if points.len() >= 2 && points[1] == 0x2D {
        // A long token: the part before any equals sign may
        // prefix the one long option.
        let (prefix, sep, explicit) = match points.iter().position(|&point| point == 0x3D) {
            Some(eq) => (
                PyStr(points[..eq].to_vec()),
                Some(Sep::Eq),
                Some(PyStr(points[eq + 1..].to_vec())),
            ),
            None => (token.clone(), None, None),
        };
        let registered = PyStr::from_str("--help");
        if registered.0.starts_with(&prefix.0) {
            out.push(OptTuple::help(registered, sep, explicit));
        }
    } else if points.len() >= 2 {
        // A short token: its first two code points may be the
        // one short option, with the rest as its argument.
        let short_prefix = PyStr(points[..2].to_vec());
        let short_explicit = PyStr(points[2..].to_vec());
        for registered_text in ["-h", "--help"] {
            let registered = PyStr::from_str(registered_text);
            if registered == short_prefix {
                out.push(OptTuple::help(
                    registered,
                    Some(Sep::Empty),
                    Some(short_explicit.clone()),
                ));
            } else if registered.0.starts_with(points) {
                out.push(OptTuple::help(registered, None, None));
            }
        }
    }
    out
}

/// Build the pattern of a token stream and the option tuples
/// by index. The first separator contributes the dash and
/// every later token is an argument unconditionally.
fn build_pattern(tokens: &[PyStr]) -> (Vec<PatChar>, HashMap<usize, OptTuple>) {
    let mut pattern = Vec::new();
    let mut options = HashMap::new();
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.eq_str("--") {
            pattern.push(PatChar::Dash);
            index += 1;
            while index < tokens.len() {
                pattern.push(PatChar::Arg);
                index += 1;
            }
            break;
        }
        match classify(token) {
            Classified::Positional => pattern.push(PatChar::Arg),
            Classified::Optional(tuple) => {
                options.insert(index, tuple);
                pattern.push(PatChar::Opt);
            }
        }
        index += 1;
    }
    (pattern, options)
}

/// Match one group of the pattern for a plain positional, one
/// argument with any number of separators around it, anchored
/// at the start of the slice. The argument's position is
/// forced: separators may stand before it, and only separators
/// may stand between the start and it.
fn group_candidates(pattern: &[PatChar]) -> Vec<usize> {
    let mut index = 0;
    while index < pattern.len() && pattern[index] == PatChar::Dash {
        index += 1;
    }
    if index >= pattern.len() || pattern[index] != PatChar::Arg {
        return Vec::new();
    }
    let mut trailing = 0;
    while index + 1 + trailing < pattern.len() && pattern[index + 1 + trailing] == PatChar::Dash {
        trailing += 1;
    }
    // Longest first, the order the regex engine tries them.
    (0..=trailing)
        .rev()
        .map(|taken| index + 1 + taken)
        .collect()
}

/// Match the joined patterns of the first `count` positionals
/// against the pattern slice, in the matcher's own order: the
/// longest first group that still lets the rest match a
/// prefix. Mirrors the oracle's partial match, which tries
/// every positional first, then one fewer, and so on.
fn match_groups(pattern: &[PatChar], count: usize) -> Option<Vec<usize>> {
    if count == 0 {
        return Some(Vec::new());
    }
    for span in group_candidates(pattern) {
        if let Some(mut rest) = match_groups(&pattern[span..], count - 1) {
            let mut out = vec![span];
            out.append(&mut rest);
            return Some(out);
        }
    }
    None
}

fn match_partial(pattern: &[PatChar], actions: usize) -> Vec<usize> {
    for count in (1..=actions).rev() {
        if let Some(counts) = match_groups(pattern, count) {
            return counts;
        }
    }
    Vec::new()
}

/// The span of the subcommand positional's group, which takes
/// its one argument and then every remaining pattern
/// character of whatever kind.
fn match_subcommand_group(pattern: &[PatChar]) -> Option<usize> {
    let mut index = 0;
    while index < pattern.len() && pattern[index] == PatChar::Dash {
        index += 1;
    }
    if index >= pattern.len() || pattern[index] != PatChar::Arg {
        return None;
    }
    Some(pattern.len())
}

/// Read the value of one positional from its token slice: the
/// first separator string in the slice is struck, one
/// remaining string is the value, and any other count leaves
/// a list.
fn slot_value(slice: &[PyStr]) -> Slot {
    let mut strings: Vec<PyStr> = slice.to_vec();
    if let Some(position) = strings.iter().position(|token| token.eq_str("--")) {
        strings.remove(position);
    }
    if strings.len() == 1 {
        Slot::One(strings.into_iter().next().expect("one string"))
    } else {
        Slot::Listed(strings)
    }
}

enum HelpOutcome {
    Fire,
    /// An ignored explicit argument refusal, carrying the
    /// argument the oracle's message names.
    Refuse(PyStr),
}

/// Consume the help optional the tuple names. An explicit
/// argument on the long form is always an ignored explicit
/// argument refusal. On the short form an equals argument
/// likewise refuses, as does an empty one, and a cluster
/// remainder is walked the way the original consume walks
/// it: a leading minus refuses with the whole remainder, an
/// h consumes and may turn an equals into the refusal, and
/// any other character leaves the help action standing first
/// in the queue, so the help fires.
fn consume_help(tuple: &OptTuple) -> HelpOutcome {
    let Some(explicit) = &tuple.explicit else {
        return HelpOutcome::Fire;
    };
    if !tuple.option_string.eq_str("-h") {
        return HelpOutcome::Refuse(explicit.clone());
    }
    if explicit.is_empty() || tuple.sep == Some(Sep::Eq) {
        return HelpOutcome::Refuse(explicit.clone());
    }
    let mut rest = &explicit.0[..];
    loop {
        if rest.is_empty() {
            return HelpOutcome::Fire;
        }
        match rest[0] {
            0x2D => return HelpOutcome::Refuse(PyStr(rest.to_vec())),
            0x68 => {
                rest = &rest[1..];
                if rest.first() == Some(&0x3D) {
                    return HelpOutcome::Refuse(PyStr(rest[1..].to_vec()));
                }
            }
            _ => return HelpOutcome::Fire,
        }
    }
}

enum SubWalk {
    FireHelp,
    Refuse,
    RefuseAt {
        index: usize,
        argument: PyStr,
    },
    Parsed {
        catalogue: Slot,
        context: Slot,
        extras: Vec<PyStr>,
    },
}

/// Walk one subparser's token stream over its two positionals.
fn parse_sub(tokens: &[PyStr]) -> SubWalk {
    let (pattern, options) = build_pattern(tokens);
    let mut slots: [Option<Slot>; 2] = [None, None];
    let mut next_slot = 0usize;
    let mut extras: Vec<PyStr> = Vec::new();
    let max_option: Option<usize> = options.keys().copied().max();
    let mut start = 0usize;
    while let Some(maximum) = max_option {
        if start > maximum {
            break;
        }
        let next_option = options
            .keys()
            .copied()
            .filter(|index| *index >= start)
            .min()
            .expect("an option index at or after the start");
        if start != next_option {
            let end = consume_sub_positionals(&pattern, tokens, start, &mut slots, &mut next_slot);
            if end > start {
                start = end;
                continue;
            }
            start = end;
        }
        if !options.contains_key(&start) {
            extras.extend_from_slice(&tokens[start..next_option]);
            start = next_option;
        }
        let tuple = options
            .get(&start)
            .expect("the option at the start")
            .clone();
        if !tuple.help {
            extras.push(tokens[start].clone());
            start += 1;
            continue;
        }
        match consume_help(&tuple) {
            HelpOutcome::Fire => return SubWalk::FireHelp,
            HelpOutcome::Refuse(argument) => {
                return SubWalk::RefuseAt {
                    index: start,
                    argument,
                };
            }
        }
    }
    let stop = consume_sub_positionals(&pattern, tokens, start, &mut slots, &mut next_slot);
    extras.extend_from_slice(&tokens[stop..]);
    match (slots[0].take(), slots[1].take()) {
        (Some(catalogue), Some(context)) => SubWalk::Parsed {
            catalogue,
            context,
            extras,
        },
        _ => SubWalk::Refuse,
    }
}

/// Fill subparser positionals from the chunk at the start,
/// returning the index the chunk stopped at.
fn consume_sub_positionals(
    pattern: &[PatChar],
    tokens: &[PyStr],
    start: usize,
    slots: &mut [Option<Slot>; 2],
    next_slot: &mut usize,
) -> usize {
    let remaining = 2 - *next_slot;
    if remaining == 0 {
        return start;
    }
    let counts = match_partial(&pattern[start..], remaining);
    let mut index = start;
    for (offset, count) in counts.iter().enumerate() {
        slots[*next_slot + offset] = Some(slot_value(&tokens[index..index + count]));
        index += count;
    }
    *next_slot += counts.len();
    index
}

/// Walk the root parser's token stream.
fn parse_root(tokens: &[PyStr]) -> Walk {
    let (pattern, options) = build_pattern(tokens);
    let mut state = RootState {
        seen: false,
        assigned: None,
        sub_extras: Vec::new(),
    };
    let mut extras: Vec<PyStr> = Vec::new();
    let max_option: Option<usize> = options.keys().copied().max();
    let mut start = 0usize;
    while let Some(maximum) = max_option {
        if start > maximum {
            break;
        }
        let next_option = options
            .keys()
            .copied()
            .filter(|index| *index >= start)
            .min()
            .expect("an option index at or after the start");
        if start != next_option {
            let end = match consume_root_positionals(&pattern, tokens, start, &mut state) {
                Ok(end) => end,
                Err(walk) => return walk,
            };
            if end > start {
                start = end;
                continue;
            }
            start = end;
        }
        if !options.contains_key(&start) {
            extras.extend_from_slice(&tokens[start..next_option]);
            start = next_option;
        }
        let tuple = options
            .get(&start)
            .expect("the option at the start")
            .clone();
        if !tuple.help {
            extras.push(tokens[start].clone());
            start += 1;
            continue;
        }
        match consume_help(&tuple) {
            HelpOutcome::Fire => return Walk::FireHelp(None),
            HelpOutcome::Refuse(argument) => {
                return Walk::RefuseAt {
                    index: start,
                    argument,
                };
            }
        }
    }
    let stop = match consume_root_positionals(&pattern, tokens, start, &mut state) {
        Ok(end) => end,
        Err(walk) => return walk,
    };
    extras.extend_from_slice(&tokens[stop..]);
    if !state.seen {
        return Walk::Refuse;
    }
    if !extras.is_empty() || !state.sub_extras.is_empty() {
        return Walk::Refuse;
    }
    match state.assigned {
        Some((cmd, catalogue, context)) => Walk::Assigned {
            cmd,
            catalogue,
            context,
        },
        None => Walk::Refuse,
    }
}

struct RootState {
    seen: bool,
    assigned: Option<(Cmd, Slot, Slot)>,
    sub_extras: Vec<PyStr>,
}

/// Fill the root parser's one positional from the chunk at
/// the start, which runs the subparser over the swallowed
/// slice when the chunk assigns the subcommand.
fn consume_root_positionals(
    pattern: &[PatChar],
    tokens: &[PyStr],
    start: usize,
    state: &mut RootState,
) -> Result<usize, Walk> {
    if state.seen {
        return Ok(start);
    }
    let Some(span) = match_subcommand_group(&pattern[start..]) else {
        return Ok(start);
    };
    let slice = &tokens[start..start + span];
    let name = &slice[0];
    let cmd = if name.eq_str("sheet") {
        Cmd::Sheet
    } else if name.eq_str("gaps") {
        Cmd::Gaps
    } else {
        return Err(Walk::Refuse);
    };
    state.seen = true;
    match parse_sub(&slice[1..]) {
        SubWalk::FireHelp => Err(Walk::FireHelp(Some(cmd))),
        SubWalk::Refuse => Err(Walk::Refuse),
        SubWalk::RefuseAt { index, argument } => Err(Walk::RefuseAt {
            index: start + 1 + index,
            argument,
        }),
        SubWalk::Parsed {
            catalogue,
            context,
            extras,
        } => {
            state.assigned = Some((cmd, catalogue, context));
            state.sub_extras = extras;
            Ok(start + span)
        }
    }
}
