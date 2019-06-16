#[macro_use]
extern crate structopt;
mod page;
mod song;
use structopt::StructOpt;

/// A downloader for the Hype Machine
#[derive(StructOpt, Debug)]
struct CLI {
    /// The account you want to download the favorites from
    #[structopt(short = "a", long = "account", default_value = "popular")]
    account: String,

    /// If the session should be interactive or just download the previous <number> of tracks
    #[structopt(short = "i", long = "interactive")]
    interactive: bool,

    /// The offset page where the downloading you should be strat to be fetched
    #[structopt(short = "p", long = "page", default_value = "1")]
    page: u32,
}

fn main() {
    let cli_options = CLI::from_args();
}
