pub enum ParseState<'a> {
    Void,
    Combo,
    Parameter(&'a str),
    ForcePos
}
#[derive(Debug)]
pub enum Arg<'a> {
    Opt(&'a str),
    OptPar(&'a str, &'a str),
    Pos(&'a str)
}
pub enum ParseHint {
    Next,
    ExpectParameter
}
#[derive(Debug)]
pub enum ParseError<'a> {
    MissingParameter(&'a str)
}
impl<'a> ParseState<'a> {
    pub fn parse(self, arg: &'a str, mut cb: impl FnMut(Arg) -> ParseHint) -> ParseState<'a> {
        match self {
            ParseState::Void => {
                let mut chars = arg.chars();
                let (arg_text, arg_type) = if chars.next() == Some('-') {
                    match chars.next() {
                        Some('-') => if arg.len() == 2 { return ParseState::ForcePos } else { (&arg[2..], Arg::Opt(&arg[2..])) },
                        Some(_) => return ParseState::Combo.parse(&arg[1..], cb),
                        None => (arg, Arg::Pos(arg)),
                    }
                } else {
                    (arg, Arg::Pos(arg))
                };
                let hint = cb(arg_type);
                match hint {
                    ParseHint::Next => ParseState::Void,
                    ParseHint::ExpectParameter => ParseState::Parameter(arg_text)
                }
            },
            ParseState::Combo => {
                let hint = cb(Arg::Opt(&arg[0..1]));

                if arg.len() == 1 {
                    match hint {
                        ParseHint::Next => if arg.len() == 1 { ParseState::Void } else { ParseState::Combo.parse(&arg[1..], cb) },
                        ParseHint::ExpectParameter => ParseState::Parameter(&arg[0..1])
                    }
                } else {
                    match hint {
                        ParseHint::Next => ParseState::Combo.parse(&arg[1..], cb),
                        ParseHint::ExpectParameter => ParseState::Parameter(&arg[0..1]).parse(&arg[1..], cb)
                    }
                }
            }
            ParseState::Parameter(opt) => {
                cb(Arg::OptPar(opt, arg));
                ParseState::Void 
            },
            ParseState::ForcePos => {
                cb(Arg::Pos(arg));
                ParseState::Void 
            }
        }
    }
}

pub fn parse<'a>(args: &'a [&str], mut cb: impl FnMut(Arg) -> ParseHint) -> Result<(), ParseError<'a>> {
    let end_state = args.iter().fold(ParseState::Void, move |state, arg| state.parse(arg, &mut cb));
    match end_state {
        ParseState::Void | ParseState::ForcePos => Ok(()),
        ParseState::Parameter(opt) => Err(ParseError::MissingParameter(opt)),
        ParseState::Combo => panic!("In combo state after parse!")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn positional() {
        parse(&["filename"], |arg| {
            match arg {
                Arg::Pos("filename") => ParseHint::Next,
                a @ _ => panic!("Invalid parameter {:?}", a)
            }
        }).expect("Parse error");
    }
    #[test]
    fn help() {
        parse(&["--help"], |arg| {
            match arg {
                Arg::Opt("help") => ParseHint::Next,
                a @ _ => panic!("Invalid parameter {:?}", a)
            }
        }).expect("Parse error");
    }
    #[test]
    fn parameter() {
        parse(&["--foo", "bar"], |arg| {
            match arg {
                Arg::Opt("foo") => ParseHint::ExpectParameter,
                Arg::OptPar("foo", arg) => {
                    assert!(arg == "bar");
                    ParseHint::Next
                }
                a @ _ => panic!("Invalid parameter {:?}", a)
            }
        }).expect("Parse error");
    }#[test]
    fn multiple() {
        let mut got_a = false;
        let mut got_b = false;
        let mut got_c = false;
        parse(&["-abc"], |arg| {
            match arg {
                Arg::Opt("a") => { got_a = true; ParseHint::Next },
                Arg::Opt("b") => { got_b = true; ParseHint::Next },
                Arg::Opt("c") => { got_c = true; ParseHint::Next },
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
                Arg::Opt("a") => { got_a = true; ParseHint::Next },
                Arg::Opt("b") => { got_b = true; ParseHint::Next },
                Arg::Opt("c") => { ParseHint::ExpectParameter },
                Arg::OptPar("c", "foo") => { got_c = true; ParseHint::Next },
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
                Arg::Opt("a") => { got_a = true; ParseHint::Next },
                Arg::Opt("b") => { got_b = true; ParseHint::Next },
                Arg::Opt("c") => { ParseHint::ExpectParameter },
                Arg::OptPar("c", "foo") => { got_c = true; ParseHint::Next },
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
        let mut pos = vec![];

        let params = &["foo", "-abcfoo", "bar", "--foobar", "--", "--baz"];
        parse(params, |arg| {
            match arg {
                Arg::Opt("a") => { got_a = true; ParseHint::Next },
                Arg::Opt("b") => { got_b = true; ParseHint::Next },
                Arg::Opt("c") => { ParseHint::ExpectParameter },
                Arg::Opt("foobar") => { got_foobar = true; ParseHint::Next },
                Arg::OptPar("c", value) => { c = Some(value.to_string()); ParseHint::Next },
                Arg::Pos(value) => { pos.push(value.to_string()); ParseHint::Next },
                a @ _ => panic!("Invalid parameter {:?}", a)
            }
        }).expect("Parse error");
        assert!(got_a && got_b && c == Some("foo".to_string()) &&
                got_foobar &&
                pos[0] == "foo" && pos[1] == "bar" && pos[2] == "--baz");
    }
}
