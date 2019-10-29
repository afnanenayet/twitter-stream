use crate::config::VerifiedConfig;
use crate::sentiment::{analyze, SentimentScore};
use futures::future::{lazy, Future};
use futures::Async;

/// The module representing the server
///
/// This module contains methods and structs pertaining to the constantly server the logic that the
/// server is going to dispatch, as well as things like maintaining the server state and
/// implementing graceful shutdown routines.
use anyhow::Result;
use egg_mode::stream::{filter, StreamMessage, TwitterStream};
use futures::Stream;
use std::time::SystemTime;

/// A sentiment score paired with a timestamp
///
/// This contains a sentiment score and an associated timestamp. This is the most basic unit of
/// data that is ingested for the time-series sentiment analysis.
#[derive(Debug, Copy, Clone)]
struct ScoreTimestamp {
    /// The sentiment score
    pub score: SentimentScore,

    /// The time when the sentiment score was recorded
    pub timestamp: SystemTime,
}

/// A type representing sentiment analysis time-series data
///
/// This is a convenient type alias for a vector of timestamps. A time series is contiguous, so
/// these scores should be pushed to the vector in order.
type ScoreSeries = Vec<ScoreTimestamp>;

/// A future that gets the sentiment score for a tweet
///
/// This is a helper future that collects sentiment scores
struct ScoreProcessor {
    /// The scores for a particular stream
    pub scores: ScoreSeries,

    /// The topic that this particular stream processor corresponds to
    pub topic: String,

    /// The Twitter stream that's being consumed (or any stream that serves the proper struct)
    pub stream: TwitterStream,
}

impl ScoreProcessor {
    /// Create a new score processor with a given stream and topic
    fn new(topic: String, stream: TwitterStream) -> Self {
        Self {
            scores: Vec::new(),
            topic,
            stream,
        }
    }
}

impl Future for ScoreProcessor {
    type Item = ();
    type Error = ();

    /// Generate a sentiment analysis score for a tweet, if it's ready
    ///
    /// This method will poll the underlying stream to see if a tweet is ready to be analyzed. If
    /// it is, then the future will process the tweet and generate a sentiment analysis score.
    // Allowing unreachable code disables the warning that the Rust compiler throws at the bottom
    // of the loop, which is intentional.
    #[allow(unreachable_code)]
    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        loop {
            // We can't use the `try_ready` macro because it assumes that the Item and Error associated
            // types are identical for the stream that's consumed and the current future
            let value = match self.stream.poll() {
                Ok(Async::Ready(value)) => value,
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(_) => return Err(()), // equivalent to `.map_err(|_| ())`
            };

            // We silently ignore things from the stream that aren't tweets that are ready for
            // consumption
            if let Some(StreamMessage::Tweet(tweet)) = value {
                let score = analyze(&tweet.text);

                // Add the timestamp to the score
                let datapoint = ScoreTimestamp {
                    score,
                    timestamp: std::time::SystemTime::now(),
                };
                self.scores.push(datapoint);
                println!("{}: {:?}", self.topic, score);
            };
        }
        // This gives a hint to the compiler that this code is unreacable. Futures are supposed to
        // run to completion, but this one does not. Admittedly I'm abusing Futures a bit, but this
        // is the easiest way to take advantage of Tokio's runtime and process code at the same
        // time. This flag allows LLVM to heavily optimize the above loop.
        unreachable!();
    }
}

/// A struct representing the state of the server
///
/// This contains the sentiment scores that have been collected for each topic
///
/// Note: this struct contains a reference to the strings of the topics. The configuration struct
/// should outlive the `Server` struct, the keys for the score map hold a reference to.
pub struct Server {
    /// A valid config for the Twitter API
    config: VerifiedConfig,
}

impl Server {
    /// Construct a new server and initialize its state variables
    pub fn new(config: VerifiedConfig) -> Self {
        Self { config }
    }

    /// Start the webserver and stream data from the Twitter API.
    ///
    /// This starts the webserver for the application, which processed incoming tweets and also serves
    /// the graph (as necessary).
    pub fn run(&mut self) -> Result<()> {
        let cfg = self.config.get_config();
        // We have to clone the strings from the config because the Twitter library needs to own the
        // strings, which is not the most efficient.
        let con_token =
            egg_mode::KeyPair::new(cfg.consumer_key.clone(), cfg.consumer_secret.clone());
        let access_token =
            egg_mode::KeyPair::new(cfg.access_token.clone(), cfg.access_token_secret.clone());
        let token = egg_mode::Token::Access {
            consumer: con_token,
            access: access_token,
        };

        // Create a stream for each keyword so we can track sentiment scores for each stream
        let streams: Vec<ScoreProcessor> = cfg
            .keywords
            .clone()
            .iter()
            .map(|keyword| {
                let stream = filter().track(vec![keyword]).start(&token);
                ScoreProcessor::new(keyword.clone(), stream)
            })
            .collect();

        // Spawn a stream/future for each keyword concurrently
        tokio::run(lazy(|| {
            for stream in streams {
                tokio::spawn(lazy(move || stream));
            }
            Ok(())
        }));
        Ok(())
    }
}
