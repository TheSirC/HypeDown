#[macro_use]
extern crate clap;
extern crate actix_web;

use clap::{App, Arg};
use actix_web::{actix,client};

fn main() {
    // Parse arguments

    let argparse = App::new("HypeDown")
        .about("A downloader for the Hype Machine")
        .version(crate_version!())
        .arg(
            Arg::with_name("account")
                .long("account")
                .short("a")
                .required(true)
                .takes_value(true)
                .help("The account you want to download the favorites"),
        )
        .arg(
            Arg::with_name("page")
                .long("page")
                .short("p")
                .takes_value(true)
                .help("The number of the page in which you want to download the favorite"),
        )
        .arg(
            Arg::with_name("limit")
                .long("limit")
                .short("l")
                .takes_value(true)
                .help("The maximum number of track you want to download"),
        )
        .arg(
            Arg::with_name("force")
                .long("force")
                .help("Assume Yes to all queries and do not prompt"),
        )
        .arg(
            Arg::with_name("dry-run")
                .long("dry")
                .short("d")
                .help("Runs the program without downloading the tracks"),
        )
        .get_matches();

        
}
