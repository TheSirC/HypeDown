extern crate hyper;
extern crate libsnatch;

use hyper::client::Client;
use libsnatch::client::GetResponse;

#[test]
fn check_server_response() {
    let hyper_client = Client::new();
    let host = "hypem.com".to_string();
    let url: String = "http://".to_string() + &host;
    // Get informations from arguments
    let account = "SirC".to_string(); // The use of unwrap is legit here because the argument must be entered
    let page = "1".to_string(); // The use of unwrap is legit here because the argument must be entered
    let client_response = hyper_client.get_head_response(&(format!("{}/{}/{}", &url,&account,&page))).expect("The server didn't answer");
    assert!(client_response.status.is_success());
}
