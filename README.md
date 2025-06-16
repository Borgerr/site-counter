# Site Counter

A multi-threaded webscraper with different configuration options.

## Interface

Site Counter is currently a CLI tool with the following options:

A webscraper that fetches websites and counts their references from others visited, and outputs all this data into an archive.

```
Usage: site_counter [OPTIONS] <START_URL>

Arguments:
  <START_URL>  URL to start off with. Must include protocol, URL, and any optional path.

Options:
  -n, --num-workers <NUM_WORKERS>
          Number of maximum worker threads.
  -t, --tmpfs-size <ARCHIVE_SIZE>
          Maximum size of the produced archive, in KB.
  -d, --destination <DESTINATION_ZIPFILE>
          Where to place the result archive.
  -i, --is-bfs
          Crawling algorithm is breadth first instead of depth first when set.
  -v
          Give verbose output at runtime about which URLs are visited, whether or not responses were received, etc.
  -h, --help
          Print help
  -V, --version
          Print version
```

