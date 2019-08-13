use argh;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut positional = vec![];
    let mut help = false;
    let mut foo = None;
    let mut a = false;
    let mut b = false;
    let mut z = false;
    let mut v = 0;
    //let result = argh::parse_string_iterator(args.iter().by_ref(), |arg| {
    let result = argh::parse_string_iterator(std::env::args(), |arg| {
        let mut hint = None;
        match arg {
            argh::Arg::Pos(x) => positional.push(x),
            argh::Arg::Opt("foo") | argh::Arg::Opt("f") => hint = Some(argh::ParseHint::ExpectParameter),
            argh::Arg::OptPar("foo", x) | argh::Arg::OptPar("f", x) => foo = Some(x),
            argh::Arg::Opt("help") | argh::Arg::Opt("h") => help = true,
            argh::Arg::Opt("a") => a = true,
            argh::Arg::Opt("b") => b = true,
            argh::Arg::Opt("z") => z = true,
            argh::Arg::Opt("v") => v += 1,
            argh::Arg::Opt(_) | argh::Arg::OptPar(_, _) => hint = Some(argh::ParseHint::InvalidOption),

        };
        hint
    });
    if let Err(e) = result {
        match e {
            argh::ParseError::InvalidOption(opt) => eprintln!("Invalid option: {}", opt),
            argh::ParseError::MissingParameter(opt) => eprintln!("Missing parameter for {}", opt),
            argh::ParseError::UnexpectedParameter(opt, par) => eprintln!("Unexpected parameter for {}: {}", opt, par)
        }
        return;
    }

    if help {
        println!("all [-abzv] [--foo|-f] [--help|-h]");
        return;
    }

    println!("foo: {:?}", foo);
    println!("a: {}", a);
    println!("b: {}", b);
    println!("z: {}", z);
    println!("v: {}", v);
    println!("positional: {}", positional.join(", "));
}
