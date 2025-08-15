use core::error;
use core::fmt;
use core::panic::Location;
use std::path::PathBuf;

use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: Box<dyn error::Error + Send>, in_operation: Operation, loc: Option<Box<Location<'static>>>) {
            display(
                "IO error while {}{}: {}",
                in_operation,
                err,
                loc.as_ref().map_or_else(String::new, |boxed_l| format!("(in {})", boxed_l.as_ref()))
            )
            source(err.as_ref())
        }
        UnknownSshKeytype(str: String) {
            display("Unknown SSH key type: {str}")
        }
        UnsharableProject {
            display("Couldn't determine any interface to share the project to.\nHint: the SSH Keys is restricted to your local networks, which renders the key useless with no allowed source address.")
        }
        Bug(bug: Box<dyn core::any::Any + Send + 'static>) {
            display("Unhandled termination. We would appreciate a bug report for this")
            from()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    GettingProjectFile,
    JoiningProject(url::Url),
    Generic(&'static str),
    File(&'static str, PathBuf),
    Shell,
    ParseSshKeygen,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::GettingProjectFile => write!(f, "downloading project files"),
            Operation::JoiningProject(url) => write!(f, "joining project from {url}"),
            Operation::Generic(inner) => write!(f, "{inner}"),
            Operation::File(op, path) => write!(f, "{op} in {path:?}"),
            Operation::Shell => write!(f, "replacing process with git-shell"),
            Operation::ParseSshKeygen => write!(f, "unexpected output from ssh-keygen"),
        }
    }
}

impl Operation {
    #[track_caller]
    pub fn capture<E>(&self) -> impl FnOnce(E) -> Error
    where
        E: error::Error + Send + 'static,
    {
        let op = self.clone();

        let loc = if cfg!(debug_assertions) {
            Some(Box::new(*Location::caller()))
        } else {
            None
        };

        move |err| Error::Io(Box::new(err), op, loc)
    }
}
