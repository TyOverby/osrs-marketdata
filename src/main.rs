mod collector;
mod serve;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    match args.get(1).map(|s|s.as_str())  {
        | None => Err(format!("must provide argument").into()),
        | Some ("collect") => collector::main(),
        | Some ("serve") => serve::main(),
        | Some (other) => Err(format!("{} is not recognized", other).into()),
    }
}
