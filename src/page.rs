use futures::{future, Future};
use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;
use scraper::{Html, Selector};
use page::hyper::client::connect::Connect;
use song::Song;
use std::collections::HashSet;

trait HypeAPI {
    fn get_cookie(&self, url: Uri) -> Result<hyper::header::HeaderValue, hyper::Error>;
    fn get_http_response(
        &self,
        url: Uri,
        cookie: hyper::header::HeaderValue,
    ) -> Result<hyper::header::HeaderValue, hyper::Error>;
}

impl<T> HypeAPI for Client<HttpsConnector<T>>
where
    HttpsConnector<T>: Connect,
    T: Connect + Sync + 'static,
    T::Transport: 'static,
    T::Future: 'static,
{
    fn get_cookie(&self, url: Uri) -> Result<hyper::header::HeaderValue, hyper::Error> {
        (*self)
            .get(url)
            .map(|res| {
                if let Some(cookie) = res.headers().get("SetCookie") {
                    Ok(*cookie)
                } else {
                    panic!("Unable to find the Cookie header")
                }
            }).wait()
            .expect("Could unwrap the future for the cookie")
    }
    fn get_http_response(&self, url: Uri) -> Result<hyper::header::HeaderValue, hyper::Error> {
        (*self)
            .get(url)
            .map(|res| Ok(res.into_body().concat2()))
            .wait()
            .expect("Could unwrap the future for the cookie")
    }
}

pub struct Page {
    // The page url
    account: String,
    // The page number where at
    page: i32,
    // The list of track retrievable on this page
    track_song: HashSet<Song>,
}

impl Page {
    pub fn default() -> Page {
        Page {
            account: "popular".into(),
            page: 1,
            track_song: HashSet::new(),
        }
    }
    pub fn new(account: String) -> Self {
        Page {
            account,
            ..Page::default()
        }
    }

    fn url<'a>(self) -> Uri {
        format!("http://hypem.com/{}/{}", self.account, self.page)
            .parse()
            .expect("Unable to parse the uri")
    }

    fn retrieve_page(self, page_url: Uri) -> Page {
        // Initialization of the client
        let https_connector = HttpsConnector::new(4).expect("Initialization of the HTTPS failed");
        let client = Client::builder().build(https_connector);
        // Retrieve the html page to extract the json containing the keys of the tracks
        let mut cookie_response = client
            .get_cookie(self.url())
            .expect("The server didn't answer for the cookie");
        let mut html_page_content = server_response.to_str().unwrap().to_string();
        // Creating the cookie for later
        let html_page_content = client
            .get_http_response(self.url())
            .expect("The server didn't answer the first time");
        // Creating the parsed html page
        let html_page_content = Html::parse_document(&html_page_content);
        // Create the parser
        let json_class =
            Selector::parse("#displayList-data").expect("Initializing the parsing failed");
        // Parsing the DOM to find the json
        let json = html_page_content
            .select(&json_class)
            .collect::<Vec<_>>()
            .iter()
            .map(|&x| x.inner_html())
            .collect::<String>()
            .map(json::parse)
            .expect("Failed to parse the json in the page");
        println!("{}", json);
    }

    pub fn download_songs(self, limit: usize) {
        unimplemented!()
    }
}
