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

    /// The maximum number of track you want to download
    #[structopt(short = "l", long = "limit", default_value = "1")]
    limit: i32,

    /// Runs the program without downloading the tracks
    #[structopt(short = "d", long = "dry-run")]
    dry_run: bool,

    /// Assume Yes to all queries and do not prompt
    #[structopt(short = "f", long = "force")]
    force: bool,
}

fn main() {
    let cli_options = CLI::from_args();
}
