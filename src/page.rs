use crate::song::Song;
use futures::{Future, Stream};
use hyper::{client::connect::Connect, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use parallel_getter::ParallelGetter;
use reqwest::Client as reqC;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs::File;

type Error = Box<hyper::Error>;
trait HypeAPI {
    fn get_cookie(&self, url: Uri) -> Result<hyper::header::HeaderValue, hyper::Error>;
    fn get_http_response(
        &self,
        url: Uri,
        cookie: hyper::header::HeaderValue,
    ) -> Result<String, Box<hyper::Error>>;
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
                    Ok(cookie.to_owned())
                } else {
                    panic!("Unable to find the Cookie header")
                }
            })
            .wait()
            .expect("Could unwrap the future for the cookie")
    }
    fn get_http_response(
        &self,
        url: Uri,
        cookie: hyper::header::HeaderValue,
    ) -> Result<String, Box<hyper::Error>> {
        use std::str;
        (*self)
            .request(
                Request::get(url)
                    .header("Cookie", cookie)
                    .body(hyper::Body::empty())
                    .expect("The body of the request could not be consumed"),
            )
            .and_then(|res| res.into_body().concat2())
            .and_then(|c| {
                Ok(str::from_utf8(&c)
                    .map(str::to_owned)
                    .expect("The body could not be parsed as a UTF-8 string !"))
            })
            .map_err(Error::from)
            .map_err(Box::from)
            .wait()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Page {
    // The page url
    account: String,
    // The page number where at
    page: i32,
    // The list of track retrievable on this page
    track_song: Vec<Song>,
}

impl Page {
    pub fn default() -> Page {
        Page {
            account: "popular".into(),
            page: 1,
            track_song: Vec::with_capacity(40), // Because a page is 40 songs long
        }
    }
    pub fn new(account: String) -> Self {
        Page {
            account,
            ..Page::default()
        }
    }

    fn url(&self) -> Uri {
        format!("http://hypem.com/{}/{}", self.account, self.page)
            .parse()
            .expect("Unable to parse the uri")
    }

    fn retrieve_page(&self, page_url: Uri) -> Page {
        // Initialization of the client
        let https_connector = HttpsConnector::new(4).expect("Initialization of the HTTPS failed");
        let client = Client::builder().build(https_connector);
        // Creating the cookie for later
        let mut cookie_response = client
            .get_cookie(self.url())
            .expect("The server didn't answer for the cookie");
        // Retrieve the html page to extract the json containing the keys of the tracks
        let html_page_content = client
            .get_http_response(self.url(), cookie_response)
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
            .collect::<String>();
        serde_json::from_str(&json).expect("The HTML could not be parsed a Page!")
    }

    pub fn download_songs(&self, songs: Vec<Song>) {
        let _: Vec<_> = songs.iter().map(|song| {
        let client = std::sync::Arc::new(reqC::new());
            let mut file = File::create(song.filename()).unwrap();
            let result = ParallelGetter::new(
                &song.url()
                    .expect("The song seem to have a malformed")
                    .to_string(),
                &mut file,
            )
            // Optional client to use for the request.
            .client(client)
            // Optional path to store the parts.
            .cache_path(std::path::PathBuf::from("."))
            // Number of theads to use.
            .threads(4)
            // threshold (length in bytes) to determine when multiple threads are required.
            .threshold_parallel(1 * 1024 * 1024)
            // threshold for defining when to store parts in memory or on disk.
            .threshold_memory(10 * 1024 * 1024)
            // Callback for monitoring progress.
            .callback(
                16,
                Box::new(|progress, total| {
                    println!("{} of {} KiB downloaded", progress / 1024, total / 1024);
                }),
            )
            // Commit the parallel GET requests.
            .get();
            if let Err(why) = result {
                eprintln!("errored: {}", why);
            }
        }).collect();
    }
}
