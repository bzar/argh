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
    InvalidOption,
    ExpectParameter
}
#[derive(Debug)]
pub enum ParseError<'a> {
    InvalidOption(&'a str),
    MissingParameter(&'a str),
    UnexpectedParameter(&'a str, &'a str)
}
impl<'a> ParseState<'a> {
    pub fn parse(self, arg: &'a str, mut cb: impl FnMut(Arg<'a>) -> Option<ParseHint>) -> Result<ParseState<'a>, ParseError> {
        if arg.len() == 0 {
            return Ok(self)
        }

        match self {
            ParseState::Void => {
                let mut chars = arg.chars();
                if chars.next() == Some('-') {
                    match chars.next() {
                        Some('-') if arg.len() == 2 => Ok(ParseState::ForcePos),
                        Some('-') => {
                            if let Some(split_pos) = arg.find('=') {
                                let opt = &arg[2..split_pos];
                                let par = &arg[split_pos+1..];
                                match cb(Arg::Opt(opt)) {
                                    None => Err(ParseError::UnexpectedParameter(opt, par)),
                                    Some(ParseHint::ExpectParameter) => { cb(Arg::OptPar(opt, par)); Ok(ParseState::Void) },
                                    Some(ParseHint::InvalidOption) => Err(ParseError::InvalidOption(opt))
                                }
                            } else {
                                match cb(Arg::Opt(&arg[2..])) {
                                    None => Ok(ParseState::Void),
                                    Some(ParseHint::ExpectParameter) => Ok(ParseState::Parameter(&arg[2..])),
                                    Some(ParseHint::InvalidOption) => Err(ParseError::InvalidOption(&arg[2..]))
                                }
                            }
                        },
                        Some(_) => ParseState::Combo.parse(&arg[1..], cb),
                        None => { cb(Arg::Pos(arg)); Ok(ParseState::Void) },
                    }
                } else {
                    cb(Arg::Pos(arg));
                    Ok(ParseState::Void)
                }
            },
            ParseState::Combo => {
                let hint = cb(Arg::Opt(&arg[0..1]));

                match (hint, arg.len()) {
                    (None, 1) => if arg.len() == 1 { Ok(ParseState::Void) } else { ParseState::Combo.parse(&arg[1..], cb) },
                    (Some(ParseHint::ExpectParameter), 1) => Ok(ParseState::Parameter(&arg[0..1])),
                    (None, _) => ParseState::Combo.parse(&arg[1..], cb),
                    (Some(ParseHint::ExpectParameter), _) => ParseState::Parameter(&arg[0..1]).parse(&arg[1..], cb),
                    (Some(ParseHint::InvalidOption), _) => Err(ParseError::InvalidOption(&arg[0..1]))
                }
            }
            ParseState::Parameter(opt) => {
                if let Some(split_pos) = opt.find('=') {
                    cb(Arg::OptPar(&opt[..split_pos], &opt[split_pos+1..]));
                    ParseState::Void.parse(arg, cb)

                } else {
                    cb(Arg::OptPar(opt, arg));
                    Ok(ParseState::Void) 
                }
            },
            ParseState::ForcePos => {
                cb(Arg::Pos(arg));
                Ok(ParseState::Void)
            }
        }
    }
}

pub fn parse<'a>(args: &'a [&str], mut cb: impl FnMut(Arg<'a>) -> Option<ParseHint>) -> Result<(), ParseError<'a>> {
    match args.iter().try_fold(ParseState::Void, move |state, arg| state.parse(arg, &mut cb))? {
        ParseState::Void | ParseState::ForcePos => Ok(()),
        ParseState::Parameter(opt) => Err(ParseError::MissingParameter(opt)),
        ParseState::Combo => panic!("In combo state after parse!")
    }
}

pub fn parse_string_vec<'a>(args: &'a Vec<String>, mut cb: impl FnMut(Arg<'a>) -> Option<ParseHint>) -> Result<(), ParseError<'a>> {
    match args.iter().try_fold(ParseState::Void, move |state, arg| state.parse(arg, &mut cb))? {
        ParseState::Void | ParseState::ForcePos => Ok(()),
        ParseState::Parameter(opt) => Err(ParseError::MissingParameter(opt)),
        ParseState::Combo => panic!("In combo state after parse!")
    }
}
pub fn parse_string_iterator<'a>(mut args: impl Iterator<Item=&'a String>, mut cb: impl FnMut(Arg<'a>) -> Option<ParseHint>) -> Result<(), ParseError<'a>> {
    match args.try_fold(ParseState::Void, move |state, arg| state.parse(arg, &mut cb))? {
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
        parse(&["-abc"], |arg| {
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
                Arg::OptPar("c", value) => c = Some(value),
                Arg::OptPar("bag", value) => bag = Some(value),
                Arg::Pos(value) => pos.push(value),
                a @ _ => panic!("Invalid parameter {:?}", a)
            };
            hint
        }).expect("Parse error");
        assert!(got_a && got_b && c == Some("foo") &&
                got_foobar && bag == Some("bad") &&
                pos[0] == "foo" && pos[1] == "bar" && pos[2] == "--baz");
    }
}
