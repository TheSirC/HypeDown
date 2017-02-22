extern crate hyper;
extern crate libsnatch;
extern crate scraper;

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
    // Create the parser
    let json_class = Selector::parse("#displayList-data").expect("Initializing the parsing failed");
    // Creating a parsing fragment (for it to live long enough)
    let parser = Html::parse_fragment(&html_page_content);
    // Parsing the DOM to find the json
    let json = parser.select(&json_class);
    assert!(json)
}
