use argh;

fn main() {
    let mut positional = vec![];
    let mut help = false;
    let mut foo = None;
    let mut a = false;
    let mut b = false;
    let mut z = 0;
    let mut v = 0;

    let result = argh::parse_argv(|arg| {
        use argh::{Arg, ParseHint};
        let mut hint = None;
        match arg {
            Arg::Pos(x) => positional.push(x.to_string()),
            Arg::Opt(x) => match x {
                "foo" | "f" | "z" => hint = Some(ParseHint::ExpectParameter),
                "help"| "h" => help = true,
                "a" => a = true,
                "b" => b = true,
                "v" => v += 1,
                _ => hint = Some(ParseHint::InvalidOption),
            },
            Arg::OptPar(x, value) => match x {
                "foo" | "f" => foo = Some(x.to_string()),
                "z" => match value.parse() {
                    Ok(i) => z = i,
                    Err(_) => hint = Some(ParseHint::InvalidValue("z must be an integer".to_string()))
                },
                _ => hint = Some(ParseHint::InvalidOption),
            },
        };
        hint
    });
    
    if let Err(e) = result {
        eprintln!("{}", e);
        return;
    }

    if help {
        println!("all [-abzv] [--foo|-f] [--help|-h] [-- args...]");
        return;
    }

    println!("foo: {:?}", foo);
    println!("a: {}", a);
    println!("b: {}", b);
    println!("z: {}", z);
    println!("v: {}", v);
    println!("positional: {}", positional.join(", "));
}
