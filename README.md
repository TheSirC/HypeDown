## HypeDown

A command-line interface to download all the favorites of a user's page. The program is a port to Rust of the wonderful [HypeMachineDownloader](https://github.com/jamend/HypeMachineDownloader.git).
It's using [snatch](https://github.com/derniercri/snatch.git) to do its download and write part.

## Requirements

For this program to work, you need to have an OpenSSL installation in your PATH (due to the usage of Hyper-TLS).

### Usage :
`HypeDown --account youraccount`
### Arguments :
```
--help          Show this help
--account       Specify the account name
--page          Specify the page number
--threads       Threads which can use to download
--limit         (optional) Specify the maximum number of tracks to download
--dry           Runs the program without downloading the tracks
```

## Acknowledgements

Thanks to [@jamend](https://github.com/jamend) for [HypeMachineDownloader](https://github.com/jamend/HypeMachineDownloader.git) and  [@derniercri](https://github.com/derniercri) for [snatch](https://github.com/derniercri/snatch.git).
