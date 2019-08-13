 use std::fmt;

#[derive(Debug)]
pub enum ParseState {
    Void,
    Combo,
    Dash,
    DoubleDash,
    Parameter(String),
    ForcePos
}
#[derive(Debug)]
pub enum Arg<'a> {
    Opt(&'a str),
    OptPar(&'a str, &'a str),
    Pos(&'a str)
}
pub enum ParseHint {
    InvalidOption,
    InvalidValue(String),
    ExpectParameter
}
#[derive(Debug)]
pub enum ParseError {
    InvalidOption(String),
    InvalidValue(String, String),
    InvalidHint,
    MissingParameter(String),
    UnexpectedParameter(String, String)
}
impl ParseState {
    pub fn parse(self, arg: &str, mut cb: impl FnMut(Arg) -> Option<ParseHint>) -> Result<ParseState, ParseError> {
        match self {
            ParseState::Void => {
                if arg.len() == 0 {
                    return Ok(self)
                } else if arg.starts_with("-") {
                    ParseState::Dash.parse(&arg[1..], cb)
                } else {
                    cb(Arg::Pos(arg));
                    Ok(ParseState::Void)
                }
            },
            ParseState::Dash => {
                if arg.len() == 0 {
                    cb(Arg::Pos("-"));
                    Ok(ParseState::Void)
                } else if arg.starts_with("-") {
                    ParseState::DoubleDash.parse(&arg[1..], cb)
                } else {
                    ParseState::Combo.parse(arg, cb)
                }
            }
            ParseState::DoubleDash => {
                if arg.len() == 0 {
                    Ok(ParseState::ForcePos)
                } else {
                    if let Some(split_pos) = arg.find('=') {
                        let opt = &arg[..split_pos];
                        let par = &arg[split_pos+1..];
                        match cb(Arg::Opt(opt)) {
                            None => Err(ParseError::UnexpectedParameter(opt.into(), par.into())),
                            Some(ParseHint::ExpectParameter) => { cb(Arg::OptPar(opt, par)); Ok(ParseState::Void) },
                            Some(ParseHint::InvalidOption) => Err(ParseError::InvalidOption(opt.into())),
                            Some(ParseHint::InvalidValue(msg)) => Err(ParseError::InvalidValue(opt.into(), msg))
                        }
                    } else {
                        match cb(Arg::Opt(arg)) {
                            None => Ok(ParseState::Void),
                            Some(ParseHint::ExpectParameter) => Ok(ParseState::Parameter(arg.into())),
                            Some(ParseHint::InvalidOption) => Err(ParseError::InvalidOption(arg.into())),
                            Some(ParseHint::InvalidValue(msg)) => Err(ParseError::InvalidValue(arg.into(), msg))
                        }
                    }
                }
            }
            ParseState::Combo => {
                if arg.len() == 0 {
                    Ok(ParseState::Void)
                } else {
                    match cb(Arg::Opt(&arg[0..1])) {
                        None => ParseState::Combo.parse(&arg[1..], cb),
                        Some(ParseHint::ExpectParameter) => ParseState::Parameter(arg[0..1].into()).parse(&arg[1..], cb),
                        Some(ParseHint::InvalidOption) => Err(ParseError::InvalidOption(arg[0..1].into())),
                        Some(ParseHint::InvalidValue(msg)) => Err(ParseError::InvalidValue(arg[0..1].into(), msg))
                    }
                }
            }
            ParseState::Parameter(ref opt) => {
                if arg.len() == 0 {
                    Ok(self)
                } else {
                    match cb(Arg::OptPar(opt, arg)) {
                        None => Ok(ParseState::Void),
                        Some(ParseHint::ExpectParameter) => Err(ParseError::InvalidHint),
                        Some(ParseHint::InvalidOption) => Err(ParseError::InvalidOption(arg[0..1].into())),
                        Some(ParseHint::InvalidValue(msg)) => Err(ParseError::InvalidValue(opt.into(), msg))

                    }
                }
            },
            ParseState::ForcePos => {
                cb(Arg::Pos(arg));
                Ok(ParseState::Void)
            }
        }
    }
}

pub fn parse(args: impl IntoIterator<Item=impl AsRef<str>>, mut cb: impl FnMut(Arg) -> Option<ParseHint>) -> Result<(), ParseError> {
    match args.into_iter().try_fold(ParseState::Void, move |state, arg| state.parse(arg.as_ref(), &mut cb))? {
        ParseState::Void | ParseState::ForcePos => Ok(()),
        ParseState::Parameter(opt) => Err(ParseError::MissingParameter(opt)),
        s => panic!("Invalid state after parse: {:?}", s)
    }
}

pub fn parse_argv(cb: impl FnMut(Arg) -> Option<ParseHint>) -> Result<(), ParseError> {
    parse(std::env::args(), cb)
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidOption(opt) => write!(f, "Invalid option: {}", opt),
            ParseError::MissingParameter(opt) => write!(f, "Missing parameter for {}", opt),
            ParseError::UnexpectedParameter(opt, par) => write!(f, "Unexpected parameter for {}: {}", opt, par),
            ParseError::InvalidValue(opt, msg) => write!(f, "Invalid value for {}: {}", opt, msg),
            ParseError::InvalidHint => write!(f, "Handler returned an invalid parse hint")
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn positional() {
        parse(&["filename"], |arg| {
            match arg {
                Arg::Pos("filename") => None,
                a @ _ => panic!("Invalid parameter {:?}", a)
            }
        }).expect("Parse error");
    }
    #[test]
    fn help() {
        parse(&["--help"], |arg| {
            match arg {
                Arg::Opt("help") => None,
                a @ _ => panic!("Invalid parameter {:?}", a)
            }
        }).expect("Parse error");
    }
    #[test]
    fn parameter() {
        parse(&["--foo", "bar"], |arg| {
            match arg {
                Arg::Opt("foo") => Some(ParseHint::ExpectParameter),
                Arg::OptPar("foo", arg) => {
                    assert!(arg == "bar");
                    None
                }
                a @ _ => panic!("Invalid parameter {:?}", a)
            }
        }).expect("Parse error");
    }#[test]
    fn multiple() {
        let mut got_a = false;
        let mut got_b = false;
        let mut got_c = false;
        parse(["-abc"].iter(), |arg| {
            match arg {
                Arg::Opt("a") => { got_a = true; None },
                Arg::Opt("b") => { got_b = true; None },
                Arg::Opt("c") => { got_c = true; None },
                a @ _ => panic!("Invalid parameter {:?}", a)
            }
        }).expect("Parse error");
        assert!(got_a && got_b && got_c);
    }
    #[test]
    fn multiple_with_params() {
        let mut got_a = false;
        let mut got_b = false;
        let mut got_c = false;
        parse(&["-abc", "foo"], |arg| {
            match arg {
                Arg::Opt("a") => { got_a = true; None },
                Arg::Opt("b") => { got_b = true; None },
                Arg::Opt("c") => { Some(ParseHint::ExpectParameter) },
                Arg::OptPar("c", "foo") => { got_c = true; None },
                a @ _ => panic!("Invalid parameter {:?}", a)
            }
        }).expect("Parse error");
        assert!(got_a && got_b && got_c);
    }
    #[test]
    fn multiple_with_combo_params() {
        let mut got_a = false;
        let mut got_b = false;
        let mut got_c = false;
        parse(&["-abcfoo"], |arg| {
            match arg {
                Arg::Opt("a") => { got_a = true; None },
                Arg::Opt("b") => { got_b = true; None },
                Arg::Opt("c") => { Some(ParseHint::ExpectParameter) },
                Arg::OptPar("c", "foo") => { got_c = true; None },
                a @ _ => panic!("Invalid parameter {:?}", a)
            }
        }).expect("Parse error");
        assert!(got_a && got_b && got_c);
    }
    #[test]
    fn complex() {
        let mut got_a = false;
        let mut got_b = false;
        let mut got_foobar = false;
        let mut c = None;
        let mut bag = None;
        let mut pos = vec![];

        let params = &["--bag=bad", "foo", "-abcfoo", "bar", "--foobar", "--", "--baz"];
        parse(params, |arg| {
            let mut hint = None;
            match arg {
                Arg::Opt("a") => got_a = true,
                Arg::Opt("b") => got_b = true,
                Arg::Opt("c") | Arg::Opt("bag") => hint = Some(ParseHint::ExpectParameter),
                Arg::Opt("foobar") => got_foobar = true,
                Arg::OptPar("c", value) => c = Some(value.to_owned()),
                Arg::OptPar("bag", value) => bag = Some(value.to_owned()),
                Arg::Pos(value) => pos.push(value.to_string()),
                a @ _ => panic!("Invalid parameter {:?}", a)
            };
            hint
        }).expect("Parse error");
        assert!(got_a && got_b && c == Some("foo".into()) &&
                got_foobar && bag == Some("bad".into()) &&
                pos[0] == "foo" && pos[1] == "bar" && pos[2] == "--baz");
    }
}
