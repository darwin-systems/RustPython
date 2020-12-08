use core::fmt;

#[derive(Clone, Copy)]
pub enum Mode {
    Program,
    Statement,
}

impl core::str::FromStr for Mode {
    type Err = ModeParseError;
    fn from_str(s: &str) -> Result<Self, ModeParseError> {
        match s {
            "exec" | "single" => Ok(Mode::Program),
            "eval" => Ok(Mode::Statement),
            _ => Err(ModeParseError { _priv: () }),
        }
    }
}

#[derive(Debug)]
pub struct ModeParseError {
    _priv: (),
}

impl fmt::Display for ModeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"mode should be "exec", "eval", or "single""#)
    }
}
