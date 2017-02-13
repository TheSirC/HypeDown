extern crate hyper;
extern crate libsnatch;
extern crate scraper;

use hyper::client::Client;
use libsnatch::client::GetResponse;


#[test]
fn test_name() {
    let hyper_client = Client::new();
    let host = "hypem.com".to_string();
    let url: String = "http://".to_string() + &host;
    // Get informations from arguments
    let account = "SirC".to_string(); // The use of unwrap is legit here because the argument must be entered
    let page = "1".to_string(); // The use of unwrap is legit here because the argument must be entered
    let html_page = hyper_client.get_http_response(&(format!("{}/{}/{}", &url, &account, &page)))
        .expect("The server didn't answer");
    // Create the parser
    let json_class = Selector::parse("#displayList-data").expect("Initializing the parsing failed");
    // Parsing the DOM to find the json
    let json = Html::parse_fragment(html_page).select(&json_class).expect("Parsing failed");
    assert!(json)
}
