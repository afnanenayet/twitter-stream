# Twitter Sentiment Analysis Streaming

[![asciicast](https://asciinema.org/a/tdQxYy8oGGLQVZjh0pfnYn2Yu.svg)](https://asciinema.org/a/tdQxYy8oGGLQVZjh0pfnYn2Yu)

## Summary

Stream tweets for given keywords in realtime and generate sentiment analysis
scores for different topics, in realtime.

## Usage

`cargo run --release -- config.yaml`

This application expects a YAML file that contains the following fields:

```yaml
# change these fields to whatever your Twitter secrets are
access_token: "secret string"
access_token_secret: "secret string"
consumer_key: "secret string"
consumer_secret: "secret string"

# don't have to change this
keywords:
  - "twitter"
  - "facebook"
  - "google"
  - "travel"
  - "art"
  - "music"
  - "photography"
  - "love"
  - "fashion"
  - "food"
```

## General Architecture

I think if we wanted to scale this service to handle more data, I would
separate out the data processing, sentiment analysis, and graphing components
of this app and have them be microservices, but given time constraints and the
size of the project, I decided to make a monolith.

The API keys / secrets necessary to authenticate with the Twitter API are
passed in as environment variables, which is a fairly standard way to handle
passing around secrets in services.

The application uses a library to interact with the Twitter API, which provides
a Tokio stream. The stream is processed one tweet at a time, which invokes a
sentiment analysis library which provides the sentiment scores for each tweet
and appends to a vector which holds the time series data for the sentiment
analysis code.

I have some logic for handling configuration and I use the typestate pattern to
verify the configs. I handle the tokio/webserver stuff in the main function,
which moves the tokio streaming logic to its own thread, and has a webserver
which serves the graph. Unfortunately I was not able to serve the graph because
I wasn't able to figure out a clean way to hold a reference to the data that
was being ingested by the tweet streaming logic in the method that handles the
GET request.

### Tokio

The meat of this project is the way we use Tokio. I implement a future that
processes a stream and converts each item into a sentiment analysis score and
prints that to STDOUT, so you have a live view of scores as they come in. I
implement a separate stream per keyword so we can evaluate each keyword
concurrently as they come in.

I also set up a separate task to handle printing values to STDOUT. I was
worried that having different futures attempt to write to the console right
after getting a sentiment score might introduce some lock contention, so I set
up an MPSC queue for a separate task to ingest so that we could print to STDOUT
without having to worry about locking STDOUT.

### Dependencies

At first I wanted to use the rust-twitter-streaming crate, but it doesn't build
properly (likely because it was using nightly `async` functions and hasn't been
updated in the last few months, so things ended up breaking and never got
updated). I settled on using the
[egg-mode](https://github.com/QuietMisdreavus/twitter-rs) crate, which does
build properly and allows you to link against rustls so we don't have to worry
about linking errors with OpenSSL (which I have dealt with in the past).

I use the `structopt` crate to handle parsing command line arguments and
automatically generate nice help messages, it's one of my favorite crates.

I'm using tokio as the execution context for my app.
