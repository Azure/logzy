/// Enum representing the allowed values of the --color flag
#[derive(Debug, PartialEq)]
enum ColorOutput {
    Never,
    Auto,
    Always,
}

impl std::str::FromStr for ColorOutput {
    type Err = String;
    fn from_str(color: &str) -> Result<Self, String> {
        match color {
            "never" => Ok(ColorOutput::Never),
            "auto" => Ok(ColorOutput::Auto),
            "always" => Ok(ColorOutput::Always),
            _ => Err(format!("Invalid color option: '{}'", color)),
        }
    }
}

#[derive(Debug, structopt::StructOpt, PartialEq)]
pub struct Cli {
    /// Enable concise output
    #[structopt(short, long)]
    pub concise: bool,
    /// Configure color mode
    #[structopt(long, default_value = "auto", possible_values = &["auto", "always", "never"])]
    color: ColorOutput,
}

impl Cli {
    /// Decide how to color the output. If the value is auto, we check
    /// whether the output is a TTY. Otherwise take the obvious values
    /// for always/never
    pub(crate) fn log_colors(&mut self) -> Option<super::LogColors> {
        // If we're set to Auto, reassign depending on the value of if atty::is
        // This ensures that subsequent calls don't need to call atty::is again.
        if let ColorOutput::Auto = self.color {
            self.color = if atty::is(atty::Stream::Stdout) {
                ColorOutput::Always
            } else {
                ColorOutput::Never
            };
        }

        match self.color {
            ColorOutput::Never => None,
            ColorOutput::Always => Some(super::LogColors::default()),
            ColorOutput::Auto => unreachable!("ColorOutput::Auto should have been reassigned"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use structopt::StructOpt;

    #[test]
    fn test_default_args() {
        let args = Cli::from_iter_safe(&["logzy"]).unwrap();
        assert_eq!(
            args,
            Cli {
                concise: false,
                color: ColorOutput::Auto,
            }
        );
    }

    #[test]
    fn test_disable_color() {
        let mut args = Cli::from_iter_safe(&["logzy", "--color", "never"]).unwrap();
        assert!(args.log_colors().is_none());
    }

    #[test]
    fn test_force_color() {
        let mut args = Cli::from_iter_safe(&["logzy", "--color", "always"]).unwrap();
        assert!(args.log_colors().is_some());
    }
}
