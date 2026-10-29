// SPDX-License-Identifier: MIT OR Apache-2.0
//! Command-line basics shared by the `emit_*_vectors` and `kiss_mint` binaries (CodeRipper OPS-011
//! `cli-basics`): `--version`/`-V` and `--help`/`-h` print and exit 0 with no side effects, and any
//! argument a binary does not declare is a usage error (exit 2) instead of being silently ignored.

/// What the command line asked for.
#[derive(Debug, PartialEq, Eq)]
pub enum Parsed {
    Version,
    Help,
    /// Every argument was a declared flag (returned in order); the binary may do its work.
    Run(Vec<String>),
    /// The first argument that is neither `--version`/`--help` nor a declared flag.
    Bad(String),
}

/// Classify `args` (without the program name) against the binary's declared `flags`.
/// Arguments are read left to right and the first decisive one wins: `--version`/`--help` before a
/// bad argument print and exit 0; a bad argument before them is the usage error.
pub fn parse(args: &[String], flags: &[&str]) -> Parsed {
    let mut seen = Vec::new();
    for a in args {
        match a.as_str() {
            "--version" | "-V" => return Parsed::Version,
            "--help" | "-h" => return Parsed::Help,
            f if flags.contains(&f) => seen.push(a.clone()),
            _ => return Parsed::Bad(a.clone()),
        }
    }
    Parsed::Run(seen)
}

/// Process entry helper: handle version/help/bad arguments (printing and exiting), and return the
/// declared flags that were given. `flags` pairs each flag with its one-line description.
pub fn basics(bin: &str, summary: &str, flags: &[(&str, &str)]) -> Vec<String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let names: Vec<&str> = flags.iter().map(|(f, _)| *f).collect();
    let usage = || {
        let mut u = format!("{bin} {}\n{summary}\n\nUSAGE: {bin} [OPTIONS]\n\nOPTIONS:\n", env!("CARGO_PKG_VERSION"));
        for (f, d) in flags {
            u.push_str(&format!("  {f}  {d}\n"));
        }
        u.push_str("  -V, --version  print the version and exit\n  -h, --help  print this help and exit\n");
        u
    };
    match parse(&args, &names) {
        Parsed::Version => {
            println!("{bin} {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
        Parsed::Help => {
            print!("{}", usage());
            std::process::exit(0);
        }
        Parsed::Bad(a) => {
            eprintln!("{bin}: unrecognized argument '{a}'\n\n{}", usage());
            std::process::exit(2);
        }
        Parsed::Run(seen) => seen,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn version_help_bad_and_declared_flags() {
        assert_eq!(parse(&v(&[]), &[]), Parsed::Run(vec![]));
        assert_eq!(parse(&v(&["--version"]), &[]), Parsed::Version);
        assert_eq!(parse(&v(&["-h"]), &["--write"]), Parsed::Help);
        assert_eq!(parse(&v(&["--bogus"]), &["--write"]), Parsed::Bad("--bogus".into()));
        assert_eq!(parse(&v(&["--write"]), &["--write"]), Parsed::Run(v(&["--write"])));
        assert_eq!(parse(&v(&["--write"]), &[]), Parsed::Bad("--write".into()));
        assert_eq!(parse(&v(&["--bogus", "--version"]), &[]), Parsed::Bad("--bogus".into()));
    }
}
