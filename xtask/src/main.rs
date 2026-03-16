mod codegen;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("codegen");

    match cmd {
        "codegen" => {
            let check = args.iter().any(|a| a == "--check");
            codegen::run(check)
        }
        other => anyhow::bail!("unknown xtask command: {other}"),
    }
}
