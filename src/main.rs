use clap::{arg, command, ArgGroup, ColorChoice, Command};
use open_proxies::{concurrent_threads, readfile};

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();
    let inputfile = matches.get_one::<String>("input");
    let outfile = match matches.get_one::<String>("out") {
        Some(m) => m.clone(),
        None => "live.txt".to_string(),
    };
    let timeout = match matches.get_one::<u64>("timeout") {
        None => 2 as u64,
        Some(m) => m.clone(),
    };
    let threads = match matches.get_one::<u64>("threads") {
        None => 10 as u64,
        Some(m) => m.clone(),
    };
    let retry = match matches.get_one::<u64>("retrys") {
        Some(m) => m.clone(),
        None => 2 as u64,
    };
    match inputfile {
        Some(input) => {
            let inp = input.clone();
            let proxies = readfile(inp).await;
            if proxies.is_some() {
                println!("🔥 start computing! 🔥");
                concurrent_threads(Some(threads as usize), proxies.unwrap().clone(), timeout, retry as usize, Some(outfile)).await
            }
        },
        None => {
            println!("no inputfile in args!");
        }
    };
}
fn cli() -> Command {
    command!()
        .args([
            arg!(-i --input <FILENAME> "TXT file path where proxies ready to be parsed").group("options")
            .value_parser(clap::builder::NonEmptyStringValueParser::new())
            .required(true),
            arg!(-o --out <FILENAME> "file path where live proxies will be saved").group("options")
            .default_value("live.txt")
            .value_parser(clap::builder::NonEmptyStringValueParser::new())
            .required(false),
            arg!(-t --timeout <NUMBER> "single proxy compute iteration timeout in seconds").group("options")
            .default_value("2")
            .value_parser(clap::value_parser!(u64).range(1..5))
            .required(false),
            arg!(-n --threads <NUMBER> "threads number used for proxies computing").group("options")
            .default_value("10")
            .value_parser(clap::value_parser!(u64).range(2..100))
            .required(false),
            arg!(-r --retrys <NUMBER> "how many time a single proxy will be tested (>=1)").group("options")
            .default_value("2")
            .value_parser(clap::builder::NonEmptyStringValueParser::new())
            .value_parser(clap::value_parser!(u64).range(1..4))
            .required(false),
        ])
        .group(ArgGroup::new("options").multiple(true))
        .group(ArgGroup::new("usage").multiple(true))
        .next_help_heading("USAGE")
        .args([
            arg!(-a <example1> "open_proxies -i ./socks.txt -o ./live.txt -t 2 -r 2 -n 10").group("usage"),
            arg!(-b <example2> "open_proxies -i ./socks.txt -o ./live.txt").group("usage"),
        ])
        .about(r"███╗░░░███╗░█████╗░██████╗░███████╗  ░██╗░░░░░░░██╗██╗████████╗██╗░░██╗  ██╗░░░░░░█████╗░██╗░░░██╗███████╗
        ████╗░████║██╔══██╗██╔══██╗██╔════╝ ░██║░░██╗░░██║██║╚══██╔══╝██║░░██║  ██║░░░░░██╔══██╗██║░░░██║██╔════╝
        ██╔████╔██║███████║██║░░██║█████╗░░ ░╚██╗████╗██╔╝██║░░░██║░░░███████║  ██║░░░░░██║░░██║╚██╗░██╔╝█████╗░░
        ██║╚██╔╝██║██╔══██║██║░░██║██╔══╝░░ ░░████╔═████║░██║░░░██║░░░██╔══██║  ██║░░░░░██║░░██║░╚████╔╝░██╔══╝░░
        ██║░╚═╝░██║██║░░██║██████╔╝███████╗ ░░╚██╔╝░╚██╔╝░██║░░░██║░░░██║░░██║  ███████╗╚█████╔╝░░╚██╔╝░░███████╗
        ╚═╝░░░░░╚═╝╚═╝░░╚═╝╚═════╝░╚══════╝ ░░░╚═╝░░░╚═╝░░╚═╝░░░╚═╝░░░╚═╝░░╚═╝  ╚══════╝░╚════╝░░░░╚═╝░░░╚══════╝
        
        ██████╗░██╗░░░██╗  ██╗░░██╗███╗░░░███╗░█████╗░░█████╗░███████╗
        ██╔══██╗╚██╗░██╔╝  ██║░██╔╝████╗░████║██╔══██╗██╔══██╗╚════██║
        ██████╦╝░╚████╔╝░  █████═╝░██╔████╔██║╚█████╔╝██║░░██║░░███╔═╝
        ██╔══██╗░░╚██╔╝░░  ██╔═██╗░██║╚██╔╝██║██╔══██╗██║░░██║██╔══╝░░
        ██████╦╝░░░██║░░░  ██║░╚██╗██║░╚═╝░██║╚█████╔╝╚█████╔╝███████╗
        ╚═════╝░░░░╚═╝░░░  ╚═╝░░╚═╝╚═╝░░░░░╚═╝░╚════╝░░╚════╝░╚══════╝@KM8Oz")
        .color(ColorChoice::Always)
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Value {
    Bool(bool),
    String(String),
}
