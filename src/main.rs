extern crate ansi_term;
#[macro_use]
extern crate clap;
extern crate hyper;
extern crate libsnatch;
extern crate num_cpus;

use ansi_term::Colour::{Green, Yellow, Red, White};
use clap::{App, Arg};
use hyper::client::Client;
use hyper::header::Headers;
use libsnatch::authorization::{AuthorizationHeaderFactory, AuthorizationType, GetAuthorizationType};
use libsnatch::Bytes;
use libsnatch::client::GetResponse;
use libsnatch::contentlength::GetContentLength;
use libsnatch::download::download_chunks;
use libsnatch::http_version::ValidateHttpVersion;
use libsnatch::write::OutputFileWriter;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::exit;

static DEFAULT_FILENAME: &'static str = "index.html";

fn main() {

    // Parse arguments

    let argparse = App::new("HypeDown")
        .about("A downloader for the Hype Machine*")
        .version(crate_version!())
        .arg(Arg::with_name("account")
            .long("account")
            .short("a")
            .takes_value(true)
            .help("The account you want to download the favorites"))
        .arg(Arg::with_name("page")
            .long("page")
            .short("p")
            .takes_value(true)
            .help("The number of the page in which you want to download the favorite"))
        .arg(Arg::with_name("limit")
            .long("limit")
            .short("l")
            .takes_value(true)
            .help("The maximum number of track you want to download"))
        .arg(Arg::with_name("file")
            .long("file")
            .short("f")
            .takes_value(true)
            .help("The local file to save the remote content file"))
        .arg(Arg::with_name("threads")
            .long("threads")
            .short("t")
            .takes_value(true)
            .help("Threads which can use to download"))
        .arg(Arg::with_name("debug")
            .long("debug")
            .short("d")
            .help("Active the debug mode"))
        .arg(Arg::with_name("force")
            .long("force")
            .help("Assume Yes to all queries and do not prompt"))
        .arg(Arg::with_name("url")
            .index(1)
            //.multiple(true)
            .required(true))
        .get_matches();

    // Get informations from arguments

    let url = argparse.value_of("url").unwrap();

    let file = argparse.value_of("file")
        .unwrap_or_else(|| url.split('/').last().unwrap_or(DEFAULT_FILENAME));

    let threads: usize = value_t!(argparse, "threads", usize).unwrap_or(num_cpus::get_physical());

    if argparse.is_present("debug") {
        println!("# [{}] version: {}",
                 Yellow.bold().paint("DEBUG_MODE"),
                 crate_version!());
        println!("# [{}] file: {}", Yellow.bold().paint("DEBUG_MODE"), file);
        println!("# [{}] threads: {}",
                 Yellow.bold().paint("DEBUG_MODE"),
                 threads);
    }

    // Run Snatch
    let hyper_client = Client::new();

    // Get the first response from the server
    let client_response = hyper_client.get_head_response(&url).unwrap();

    print!("# Waiting a response from the remote server... ");

    if !client_response.version.greater_than_http_11() {
        println!("{}",
                 Yellow.bold()
                     .paint("OK (HTTP version <= 1.0 detected)"));
    } else {
        println!("{}", Green.bold().paint("OK !"));
    }

    let auth_type = client_response.headers.get_authorization_type();
    let auth_header_factory = match auth_type {
        Some(a_type) => {
            match a_type {
                AuthorizationType::Basic => {
                    println!("{}",
                             Yellow.bold().paint("The remote content is protected by Basic Auth."));
                    let username = prompt_user(White.bold(), "Username:");
                    let password = prompt_user(White.bold(), "Password:");
                    Some(AuthorizationHeaderFactory::new(AuthorizationType::Basic,
                                                         username,
                                                         Some(password)))
                }
                _ => {
                    println!("{}",
                             Red.bold()
                                 .paint(format!("[ERROR] The remote content is protected by {} \
                                                 Authorization, which is not supported!",
                                                a_type)));
                    exit(1);
                }
            }
        }
        None => None,
    };

    let client_response = match auth_header_factory.clone() {
        Some(header_factory) => {
            let mut headers = Headers::new();
            headers.set(header_factory.build_header());
            hyper_client.get_head_response_using_headers(&url, headers).unwrap()
        }
        None => client_response,
    };

    let local_path = Path::new(&file);

    if local_path.exists() {
        if local_path.is_dir() {
            panic!(Red.bold()
                .paint("[ERROR] The local path to store the remote content is already exists, \
                        and is a directory!"));
        }
        if !argparse.is_present("force") {
            let user_input = prompt_user(Yellow.bold(),
                                         "[WARNING] The path to store the file already exists! \
                                          Do you want to override it? [y/N]");

            if !(user_input == "y" || user_input == "Y") {
                exit(0);
            }
        } else {
            println!("{}",
                     Yellow.bold()
                         .paint("[WARNING] The path to store the file already exists! \
                                 It is going to be overriden."));
        }
    }

    let remote_content_length = match client_response.headers.get_content_length() {
        Some(remote_content_length) => remote_content_length,
        None => {
            println!("{}",
                     Red.bold()
                         .paint("[ERROR] Cannot get the content length of the remote content, \
                                 from the server."));
            exit(1);
        }
    };

    println!("# Remote content length: {:?} MB",
             (remote_content_length / 1000000) as Bytes);

    let local_file = File::create(local_path).expect("[ERROR] Cannot create a file !");

    local_file.set_len(remote_content_length)
        .expect("[ERROR] Cannot extend file to download size!");
    let out_file = OutputFileWriter::new(local_file);

    download_chunks(remote_content_length,
                    out_file,
                    threads as u64,
                    &url,
                    auth_header_factory);

    println!("{} Your download is available in {}",
             Green.bold().paint("Done!"),
             local_path.to_str().unwrap());

}

fn prompt_user(style: ansi_term::Style, prompt: &str) -> String {
    print!("{} ", style.paint(prompt));
    io::stdout().flush().expect("[ERROR] Couldn't flush stdout!");

    let mut user_input = String::new();
    io::stdin()
        .read_line(&mut user_input)
        .ok()
        .expect("[ERROR] Couldn't read line!");
    String::from(user_input.trim())
}
