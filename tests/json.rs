extern crate hyper;
extern crate libsnatch;
extern crate scraper;
extern crate json;

use hyper::client::Client;
use libsnatch::client::GetResponse;
use std::io::Read;
use scraper::{Html, Selector};
use json::iterators::Entries;
use hyper::header::{Headers, Cookie, SetCookie};
use hyper::method::Method;


#[test]
fn json_parsing() {
    let hyper_client = Client::new();
    let host = "hypem.com".to_string();
    let url: String = "http://".to_string() + &host;
    // Get informations from arguments
    let account = "SirC".to_string(); // The use of unwrap is legit here
    // because the argument must be entered
    let page = "1".to_string(); // The use of unwrap is legit here
    // because the argument must be entered
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
        println!("{:?}", content);
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
    let json = json::parse(&unser_json).unwrap();
    // Creating the json containing the tracks contained in the other json (JSONception)
    let ref tracks_json = json["tracks"];
    // Accessing the tracks inside
    let i: usize = 0;
    // println!("{:#}",tracks_json[i]);
    let ref id = json["tracks"][i]["id"];
    let ref key = json["tracks"][i]["key"];
    let ref base_url = format!("{}/serve/source/{}/{}", url, id, key);
    //
    let mut song_url_content = String::new();
    let url_song_page = hyper_client.get_json_response_using_cookie(&base_url, &cookie)
        .expect("The server didn't answer")
        .read_to_string(&mut song_url_content);
    println!("{:?}", song_url_content);
    // let song_url_content = Html::parse_document(&song_url_content);
    // Creating the serialized json
    let mut json_song = json::parse(&song_url_content).unwrap();
    println!("{:?}",
             json_song["url"].take_string().expect("No links found"));
}
