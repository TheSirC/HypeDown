use authorization::AuthorizationHeaderFactory;
use Bytes;
use write::{OutputFileWriter, OutputChunkWriter};
use client::GetResponse;
use hyper::client::Client;
use hyper::error::Error;
use hyper::header::{ByteRangeSpec, Headers, Range};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use pbr::{MultiBar, Pipe, ProgressBar, Units};
use response::CheckResponseStatus;
use std::cmp::min;
use std::io::Read;
use std::thread;
use std::time::{Instant, Duration};

/// Constant to represent the length of the buffer to download
/// the remote content
const DOWNLOAD_BUFFER_BYTES: usize = 1024 * 64;

/// Constant to represent the refresh interval (in milliseconds)
/// for the CLI
const PROGRESS_UPDATE_INTERVAL_MILLIS: u64 = 500;

/// Represents a range between two Bytes types
#[derive(Debug, PartialEq)]
struct RangeBytes(Bytes, Bytes);

/// Function to get the current chunk length, based on the chunk index.
fn get_chunk_length(chunk_index: u64,
                    content_length: Bytes,
                    global_chunk_length: Bytes)
                    -> Option<RangeBytes> {

    if content_length == 0 || global_chunk_length == 0 {
        return None;
    }

    let b_range: Bytes = chunk_index * global_chunk_length;

    if b_range >= (content_length - 1) {
        return None;
    }

    let e_range: Bytes = min(content_length - 1,
                             ((chunk_index + 1) * global_chunk_length) - 1);

    Some(RangeBytes(b_range, e_range))

}


/// Function to get the HTTP header to send to the file server, for a chunk (specified by its index)
fn get_header_from_index(chunk_index: u64,
                         content_length: Bytes,
                         global_chunk_length: Bytes)
                         -> Option<(Headers, RangeBytes)> {

    get_chunk_length(chunk_index, content_length, global_chunk_length).map(|range| {
        let mut header = Headers::new();
        header.set(Range::Bytes(vec![ByteRangeSpec::FromTo(range.0, range.1)]));
        (header, RangeBytes(range.0, range.1 - range.0))
    })
}


/// Function to get from the server the content of a chunk.
/// This function returns a Result type - Bytes if the content of the header is accessible, an Error type otherwise.
fn download_a_chunk(http_client: &Client,
                    http_header: Headers,
                    mut chunk_writer: OutputChunkWriter,
                    url: &str,
                    mpb: &mut ProgressBar<Pipe>)
                    -> Result<Bytes, Error> {

    let mut body = http_client.get_http_response_using_headers(url, http_header).unwrap();
    if !body.check_partialcontent_status() {
        return Err(Error::Status);
    }
    let mut bytes_buffer = [0; DOWNLOAD_BUFFER_BYTES];
    let mut sum_bytes = 0;

    let progress_update_interval = Duration::from_millis(PROGRESS_UPDATE_INTERVAL_MILLIS);
    let mut last_progress_bytes = 0;
    let mut last_progress_time = Instant::now() - progress_update_interval;

    while let Ok(n) = body.read(&mut bytes_buffer) {
        if n == 0 {
            return Ok(sum_bytes);
        }

        chunk_writer.write(sum_bytes, &bytes_buffer[0..n]);

        sum_bytes += n as u64;

        // Update the CLI
        if Instant::now().duration_since(last_progress_time) > progress_update_interval {
            last_progress_time = Instant::now();
            let progress_bytes_delta = sum_bytes - last_progress_bytes;
            last_progress_bytes = sum_bytes;
            mpb.add(progress_bytes_delta);
        }
    }
    mpb.add(sum_bytes - last_progress_bytes);
    return Ok(0u64);
}

/// Function to download each chunk of a remote content (given by its URL).
/// This function takes as parameters:
/// * the remote content length,
/// * a mutable reference to share between threads, which contains each chunk,
/// * the number of chunks that contains the remote content,
/// * the URL of the remote content server.
pub fn download_chunks(content_length: u64,
                       mut out_file: OutputFileWriter,
                       nb_chunks: u64,
                       url: &str,
                       authorization_header_factory: Option<AuthorizationHeaderFactory>) {

    // let mut downloaded_chunks: Arc<Mutex<Chunks>> =
    //     Arc::new(Mutex::new(Chunks::with_capacity(nb_chunks as usize)));
    let global_chunk_length: u64 = (content_length / nb_chunks) + 1;
    let mut jobs = vec![];

    let mut mpb = MultiBar::new();
    mpb.println(&format!("Downloading {} chunks: ", nb_chunks));

    for chunk_index in 0..nb_chunks {

        let (mut http_header, RangeBytes(chunk_offset, chunk_length)) =
            get_header_from_index(chunk_index, content_length, global_chunk_length).unwrap();
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        let hyper_client = Client::with_connector(connector);
        let url_clone = String::from(url);
        if let Some(auth_header_factory) = authorization_header_factory.clone() {
            http_header.set(auth_header_factory.build_header());
        }

        // Progress bar customization
        let mut mp = mpb.create_bar(chunk_length);
        mp.tick_format("▏▎▍▌▋▊▉██▉▊▋▌▍▎▏");
        mp.format("|#--|");
        mp.show_tick = true;
        mp.show_speed = true;
        mp.show_percent = true;
        mp.show_counter = false;
        mp.show_time_left = true;
        mp.set_units(Units::Bytes);
        mp.message(&format!("Chunk {} ", chunk_index));


        let chunk_writer = out_file.get_chunk_writer(chunk_offset);
        jobs.push(thread::spawn(move || match download_a_chunk(&hyper_client,
                                                                    http_header,
                                                                    chunk_writer,
                                                                    &url_clone,
                                                                    &mut mp) {
            Ok(bytes_written) => {
                if bytes_written > 0 {
                    mp.finish();
                } else {
                    panic!("The downloaded chunk {} is empty", chunk_index);
                }
            }
            Err(error) => {
                panic!("Cannot download the chunk {}, due to error {}",
                       chunk_index,
                       error);
            }
        }));
    }

    mpb.listen();

    for child in jobs {
        let _ = child.join();
    }

}
