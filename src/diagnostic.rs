use std::fmt;
use std::ops::Range;

pub struct DiagnosticPosition<'a> {
    pub path: &'a String,
    // 0-indexed, inclusive-exclusive
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

impl<'a> fmt::Display for DiagnosticPosition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(start_line) = self.start_line {
            // We _could_ just always show "a.sh:4-4" when the line range only consists of one line, but
            // "a.sh:4" is much more obvious at first glance; c.f. the GH permalink format.
            if let Some(end_line) = self.end_line {
                write!(f, "{}:{}-{}", self.path, start_line + 1, end_line)
            } else {
                write!(f, "{}:{}", self.path, start_line + 1)
            }
        } else {
            write!(f, "{}", self.path)
        }
    }
}

// Diagnostics should always be tied to the location where we want the user to
// make a change, i.e. if a.sh contains a "if change ... then change b.sh", a.sh
// has been changed but b.sh has not, then the diagnostic should be tied to b.sh.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Diagnostic {
    pub path: String,
    // 0-indexed, inclusive-exclusive
    // NB: I don't love this representation, but it doesn't make a big difference to me
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {}",
            DiagnosticPosition {
                path: &self.path,
                start_line: self.start_line,
                end_line: self.end_line,
            },
            self.message
        )
    }
}
