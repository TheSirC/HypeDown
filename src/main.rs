extern crate ansi_term;
#[macro_use]
extern crate clap;
extern crate hyper;
extern crate libsnatch;
extern crate num_cpus;
extern crate json;
extern crate scraper;

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
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use scraper::{Html, Selector};
use hyper::header::SetCookie;

fn main() {

    // Parse arguments

    let argparse = App::new("HypeDown")
        .about("A downloader for the Hype Machine")
        .version(crate_version!())
        .arg(Arg::with_name("account")
            .long("account")
            .short("a")
            .required(true)
            .takes_value(true)
            .help("The account you want to download the favorites"))
        .arg(Arg::with_name("page")
            .long("page")
            .short("p")
            .required(true)
            .takes_value(true)
            .help("The number of the page in which you want to download the favorite"))
        .arg(Arg::with_name("limit")
            .long("limit")
            .short("l")
            .required(true)
            .takes_value(true)
            .help("The maximum number of track you want to download"))
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
        .get_matches();

    // Informations always used
    let host = "hypem.com".to_string();
    let url: String = "http://".to_string() + &host;
    // Get informations from arguments
    let account = argparse.value_of("account").unwrap().to_string(); // The use of unwrap is legit
    // because the argument must be entered
    let page = argparse.value_of("page").unwrap().to_string(); // The use of unwrap is legit
    // because the argument must be entered
    let limit = argparse.value_of("limit").unwrap().to_string(); // The use of unwrap is legit
    // because the argument must be entered
    let base_url = format!("{}/{}/{}", &url, &account, &page);
    let threads: usize = value_t!(argparse, "threads", usize).unwrap_or(num_cpus::get_physical());

    if argparse.is_present("debug") {
        println!("# [{}] version: {}",
                 Yellow.bold().paint("DEBUG_MODE"),
                 crate_version!());
        println!("# [{}] threads: {}",
                 Yellow.bold().paint("DEBUG_MODE"),
                 threads);
    }

    // Run HypeDown
    let hyper_client = Client::new();
    // Ask the user for a path to download the tracks
    let path_user = &(prompt_user(White.bold(), "Local path to download the tracks :"));
    let local_path = Path::new(path_user);
    // Retrieve the html page to extract the json containing the keys of the tracks
    let mut html_page_content = String::new();
    let mut server_response =
        hyper_client.get_http_response(&(format!("{}/{}/{}", &url, &account, &page)))
            .expect("The server didn't answer");
    server_response.read_to_string(&mut html_page_content);
    // Creating the cookie for later
    let cookie_request =
        hyper_client.get_http_response(&(format!("{}/{}/{}", &url, &account, &page)))
            .expect("The server didn't answer");
    let mut cookie: &Vec<String> = &Vec::new();
    if let Some(&SetCookie(ref content)) =
        server_response.headers
            .get() {
        cookie = content;
    }
    // Creating the parsed html page
    let html_page_content = Html::parse_document(&html_page_content);
    // Create the parser
    let json_class = Selector::parse("#displayList-data").expect("Initializing the parsing failed");

    // Parsing the DOM to find the json
    let unser_json = html_page_content.select(&json_class)
        .collect::<Vec<_>>()
        .iter()
        .map(|&x| x.inner_html())
        .collect::<String>();
    // Creating the serialized json
    let json = json::parse(&unser_json).expect("Failed to parse the json in the page");

    // Iterate over tracks if there is an Err we stop at 0
    for i in 0..limit.parse::<usize>().unwrap_or(1) {
        // Parsing useful informations
        // Track's id and key
        let ref id = json["tracks"][i]["id"];
        let ref key = json["tracks"][i]["key"];
        let ref base_url = format!("{}/serve/source/{}/{}", url, id, key);
        // Track's informations
        let ref artist_name = json["tracks"][i]["artist"];
        let ref song_name = json["tracks"][i]["song"];
        let ref file_name = format!("{} - {}.mp3", artist_name, song_name);

        // Sending the request to get the url to get the file
        let mut song_url_content = String::new();
        let url_song_page = hyper_client.get_json_response_using_cookie(&base_url, cookie)
            .expect("The server didn't answer")
            .read_to_string(&mut song_url_content);
        // Creating the serialized json
        let mut json_song = json::parse(&song_url_content)
            .expect("Failed to parse the json for the song");
        let url_song = json_song["url"]
            .take_string()
            .expect("No links found");
        // Using Snatch starting here, HUGE thanks to them
        // Get the first response from the server
        let client_response = hyper_client.get_head_response(&url_song)
            .expect("The server didn't answer");

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
                                 Yellow.bold()
                                     .paint("The remote content is protected by Basic Auth."));
                        let username = prompt_user(White.bold(), "Username:");
                        let password = prompt_user(White.bold(), "Password:");
                        Some(AuthorizationHeaderFactory::new(AuthorizationType::Basic,
                                                             username,
                                                             Some(password)))
                    }
                    _ => {
                        println!("{}",
                                 Red.bold()
                                     .paint(format!("[ERROR] The remote content is protected \
                                                     by {} Authorization, which is not \
                                                     supported!",
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
                hyper_client.get_head_response_using_headers(&url_song, headers)
                    .expect("First response from server failed")
            }
            None => client_response,
        };

        if local_path.exists() {
            if !argparse.is_present("force") {
                let user_input = prompt_user(Yellow.bold(),
                                             "[WARNING] The path to store the file already \
                                              exists! Do you want to override it? [y/N]");

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
                             .paint("[ERROR] Cannot get the content length of the remote \
                                     content, from the server."));
                exit(1);
            }
        };

        println!("# Remote content length: {:?} MB",
                 (remote_content_length / 1000000) as Bytes);

        let local_file = File::create(local_path.join(file_name))
            .expect("[ERROR] Cannot create a file !");

        local_file.set_len(remote_content_length)
            .expect("[ERROR] Cannot extend file to download size!");
        let out_file = OutputFileWriter::new(local_file);

        download_chunks(remote_content_length,
                        out_file,
                        threads as u64,
                        &url_song,
                        auth_header_factory);

        println!("{} Your download is available in {}",
                 Green.bold().paint("Done!"),
                 local_path.to_str().expect("Failed to print the path"));
    }
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
