extern crate hyper;
extern crate libsnatch;
extern crate scraper;
extern crate json;

use hyper::client::Client;
use libsnatch::client::GetResponse;
use std::io::Read;
use scraper::{Html, Selector};


#[test]
fn json_parsing() {
    let hyper_client = Client::new();
    let host = "hypem.com".to_string();
    let url: String = "http://".to_string() + &host;
    // Get informations from arguments
    let account = "SirC".to_string(); // The use of unwrap is legit here because the argument must be entered
    let page = "1".to_string(); // The use of unwrap is legit here because the argument must be entered
    let mut html_page_content = String::new();
    let html_page = hyper_client.get_http_response(&(format!("{}/{}/{}", &url, &account, &page)))
        .expect("The server didn't answer")
        .read_to_string(&mut html_page_content);
    // Creating the parsed html page
    let html_page_content = Html::parse_document(&html_page_content) ;
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
    println!("{:#}",json);
}
