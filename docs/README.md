# Token Coding Challenge

## General Architecture

I think if we wanted to scale this service to handle more data, I would
separate out the data processing, sentiment analysis, and graphing components
of this app and have them be microservices, but given time constraints and the
size of the project, I decided to make a monolith.

The API keys / secrets necessary to authenticate with the Twitter API are
passed in as environment variables, which is a fairly standard way to handle
passing around secrets in services.

### Dependencies

At first I wanted to use the rust-twitter-streaming crate, but it doesn't build
properly (likely because it was using nightly `async` functions and hasn't been
updated in the last few months, so things ended up breaking and never got
updated). I settled on using the
[egg-mode](https://github.com/QuietMisdreavus/twitter-rs) crate, which does
build properly.

I use the `structopt` crate to handle parsing command line arguments and
automatically generate nice help messages, it's one of my favorite crates.

## Summary


