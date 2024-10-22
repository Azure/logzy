//! Logzy.  Simple structured log formatting tool.
extern crate chrono;
extern crate core;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate structopt;
extern crate termion;

mod args;

use chrono::prelude::*;
use serde_derive::Deserialize;
use std::{borrow::Cow, fmt, io, io::prelude::*};
use structopt::StructOpt;
use termion::color::*;

/// Struct that implements Display by printing the given string
/// using a given escape sequence, then resetting both foreground and
/// background colors
struct LogColor<'a> {
    color: Option<&'a str>,
    string: &'a str,
}

impl<'a> fmt::Display for LogColor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write at least 5 characters for the log level.
        if let Some(color) = self.color {
            write!(f, "{}{:5}{}{}", color, self.string, Fg(Reset), Bg(Reset))
        } else {
            write!(f, "{:5}", self.string)
        }
    }
}

/// Stores strings with control codes for each of the colors we use
#[derive(Debug, PartialEq)]
struct LogColors {
    critical: String,
    error: String,
    warning: String,
    info: String,
    key: String,
}

impl Default for LogColors {
    fn default() -> Self {
        LogColors {
            critical: Bg(LightRed).to_string(),
            error: Fg(LightRed).to_string(),
            warning: Fg(LightYellow).to_string(),
            info: Fg(LightGreen).to_string(),
            // Color of key in key:value pairs
            key: Fg(Cyan).to_string(),
        }
    }
}

/// Wrap the supplied string with the control codes for the given log level
fn colored_level<'a>(colors: Option<&'a LogColors>, log_level: &'a str) -> LogColor<'a> {
    let color = colors.map(|cs| match log_level {
        "CRIT" | "crit" => &cs.critical,
        "ERRO" | "ERROR" | "error" => &cs.error,
        "WARN" | "warn" => &cs.warning,
        "INFO" | "info" => &cs.info,
        _ => "",
    });

    LogColor {
        color,
        string: log_level,
    }
}

// Wrap the supplied string with the appropriate contol code for keys in
// key-value pairs
fn colored_key<'a>(colors: Option<&'a LogColors>, key: &'a str) -> LogColor<'a> {
    LogColor {
        color: colors.map(|cs| cs.key.as_str()),
        string: key,
    }
}

// Long-lived structure for pretty-printing structured logs. Does initialisation
// work up-front, and provides a format_log method that produces a value implementing
// `Display`.
struct LogRenderer {
    colors: Option<LogColors>,
}

impl Default for LogRenderer {
    fn default() -> Self {
        LogRenderer {
            colors: Some(LogColors::default()),
        }
    }
}

impl LogRenderer {
    /// Create a new LogRenderer
    fn new(colors: Option<LogColors>) -> LogRenderer {
        LogRenderer { colors }
    }

    /// Format the supplied JSON-format log to make it human-friendly
    fn format_log<'a>(&'a self, log: &'a str, concise: bool) -> LogFormatter {
        LogFormatter {
            colors: self.colors.as_ref(),
            log,
            concise,
        }
    }
}

#[derive(Deserialize)]
struct Log<'a> {
    #[serde(borrow, default = "Default::default")]
    ts: Cow<'a, str>,
    #[serde(borrow, default = "Default::default")]
    level: Cow<'a, str>,
    #[serde(borrow, default = "Default::default")]
    component: Cow<'a, str>,
    #[serde(borrow, default = "Default::default")]
    subcomponent: Cow<'a, str>,
    #[serde(borrow, default = "Default::default")]
    msg: Cow<'a, str>,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
}

/// Wrapper around a log that implements `Display` by parsing it as a JSON log
/// and printing out the appropriate fields, or printing it as-is if that isn't
/// possible.
struct LogFormatter<'a> {
    colors: Option<&'a LogColors>,
    log: &'a str,
    /// Control whether extra JSON fields should be displayed
    concise: bool,
}

impl<'a> fmt::Display for LogFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Parse the JSON format log.
        // If it fails to parse (not JSON) then return the original log unmodified.
        // A JSON log should be an object. If not, then also return the original log unmodified.
        let json_value: Log = match serde_json::from_str(self.log) {
            Ok(v) => v,
            _ => return write!(f, "{}", self.log),
        };

        let ts = DateTime::parse_from_rfc3339(&json_value.ts)
            .map_err(|_| fmt::Error)?
            .format("%Y-%m-%d %H:%M:%S%.6f")
            .to_string();
        let level = json_value.level;
        let component = json_value.component;
        let mut subcomponent = json_value.subcomponent;
        if !subcomponent.is_empty() {
            subcomponent = format!("-{}", subcomponent).into();
        }

        let msg = json_value.msg.replace('\r', " ");

        // Add the "standard" keys in a sensible consistent format
        write!(
            f,
            "{ts} {level} {component:3}{subcomponent:6} {msg}",
            ts = ts,
            level = colored_level(self.colors, &level),
            component = component,
            subcomponent = subcomponent,
            msg = msg,
        )?;

        if !self.concise {
            // Add in to the formatted log all the other keys in the JSON object
            for (key, value) in json_value.other {
                write!(f, " {}:{}", colored_key(self.colors, &key), value)?;
            }
        }

        Ok(())
    }
}

fn main_inner() -> io::Result<()> {
    // Unpack arguments
    let mut args = args::Cli::from_args();
    let concise: bool = args.concise;
    let log_colors: Option<LogColors> = args.log_colors();

    let log_renderer = LogRenderer::new(log_colors);

    let stdin = io::stdin();
    let stdout = io::stdout();

    // Lock stdin and stdout to avoid relocking on every iteration
    // This will be unlocked when we exit this function, whether in a success
    // or error scenario
    // We use a LineWriter to output data line-by-line as we produce it, rather
    // than only when the buffer is full
    let reader = io::BufReader::new(stdin.lock());
    let mut writer = io::LineWriter::new(stdout.lock());

    for line in reader.lines() {
        let line = line?;
        writeln!(writer, "{}", log_renderer.format_log(&line, concise))?;
    }

    Ok(())
}

/// Main entry point
fn main() {
    if let Err(e) = main_inner() {
        // BrokenPipe just means the output stream has been closed (e.g. if it's been
        // piped through `head`), so it's fine to exit successfully.
        if e.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("Error! e:{}", &e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn not_json() {
        let log = "This is a test log that isn't json-formatted";
        let output_log = LogRenderer::default().format_log(log, false).to_string();
        assert_eq!(log, &output_log);
    }

    #[test]
    fn is_json() {
        let message = "This is a json-formatted log";
        let log = serde_json::to_string(&json!({
            "msg": message,
            "ts": "2017-09-28T15:06:19.898587457+00:00",
            "level": "INFO",
            "component": "service1",
            "random_key": "blah",
        }))
        .unwrap();

        let output_log = LogRenderer::default().format_log(&log, false).to_string();

        let expected = format!(
            "2017-09-28 15:06:19.898587 {}INFO {}{} service1       {} {}random_key{}{}:\"blah\"",
            Fg(LightGreen),
            Fg(Reset),
            Bg(Reset),
            message,
            Fg(Cyan),
            Fg(Reset),
            Bg(Reset),
        );

        assert_eq!(output_log, expected);
    }

    #[test]
    fn millisecond_ts() {
        let log = serde_json::to_string(&json!({
            "msg": "Hello world!",
            "ts": "2017-09-28T15:06:19.898Z",
            "level": "INFO",
            "component": "service1"
        }))
        .unwrap();

        assert_eq!(
            LogRenderer::default().format_log(&log, false).to_string(),
            format!(
                "2017-09-28 15:06:19.898000 {}INFO {}{} service1       Hello world!",
                Fg(LightGreen),
                Fg(Reset),
                Bg(Reset)
            )
        );
    }

    #[test]
    fn concise_output() {
        let message = "This is a json-formatted log";
        let log = serde_json::to_string(&json!({
            "msg": message,
            "ts": "2017-09-28T15:06:19.898587457+00:00",
            "level": "INFO",
            "component": "service1",
            "random_key": "blah",
        }))
        .unwrap();

        let output_log = LogRenderer::default().format_log(&log, true).to_string();

        let expected = format!(
            "2017-09-28 15:06:19.898587 {}INFO {}{} service1       {}",
            Fg(LightGreen),
            Fg(Reset),
            Bg(Reset),
            message
        );

        assert_eq!(output_log, expected);
    }

    #[test]
    fn has_subcomponent_field() {
        let message = "This is a json-formatted log from subcomponent 'Tiny'";
        let log = serde_json::to_string(&json!({
            "msg": message,
            "ts": "2017-09-28T15:06:19.898587457+00:00",
            "level": "INFO",
            "component": "service1",
            "subcomponent": "Tiny",
            "random_key": "blah",
        }))
        .unwrap();

        let output_log = LogRenderer::default().format_log(&log, false).to_string();

        let expected = format!(
            "2017-09-28 15:06:19.898587 {}INFO {}{} service1-Tiny  {} {}random_key{}{}:\"blah\"",
            Fg(LightGreen),
            Fg(Reset),
            Bg(Reset),
            message,
            Fg(Cyan),
            Fg(Reset),
            Bg(Reset),
        );

        assert_eq!(output_log, expected);
    }
}
