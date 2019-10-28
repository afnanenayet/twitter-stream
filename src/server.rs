use crate::config::VerifiedConfig;
use crate::sentiment::{analyze, SentimentScore};

/// The module representing the server
///
/// This module contains methods and structs pertaining to the constantly server the logic that the
/// server is going to dispatch, as well as things like maintaining the server state and
/// implementing graceful shutdown routines.
use anyhow::Result;
use egg_mode::stream::{filter, StreamMessage};
use futures::Stream;
use std::time::SystemTime;
use tokio::runtime::current_thread::block_on_all;

// TODO move the score/statistics stuff to its own module
// TODO move the access token stuff to the env

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

/// A struct representing the state of the server
///
/// This contains the sentiment scores that have been collected for each topic
///
/// Note: this struct contains a reference to the strings of the topics. The configuration struct
/// should outlive the `Server` struct, the keys for the score map hold a reference to.
pub struct Server {
    /// The processed datapoints for sentiment analysis
    scores: ScoreSeries,

    /// A valid config for the Twitter API
    config: VerifiedConfig,
}

impl Server {
    /// Construct a new server and initialize its state variables
    pub fn new(config: VerifiedConfig) -> Self {
        Self {
            scores: Vec::new(),
            config,
        }
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

        // Direct the stream to filter for keywords
        let stream = filter()
            .track(cfg.keywords.clone().into_iter())
            .start(&token);

        // Process each tweet as they come in
        block_on_all(stream.for_each(|m| {
            if let StreamMessage::Tweet(tweet) = m {
                let score = analyze(&tweet.text);

                // Add the timestamp to the score
                let datapoint = ScoreTimestamp {
                    score,
                    timestamp: std::time::SystemTime::now(),
                };
                self.scores.push(datapoint);
            };
            futures::future::ok(())
        }))?;
        Ok(())
    }
}
