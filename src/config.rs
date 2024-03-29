use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

/// This module defines the structs that handle the configuration of the app as well as routines
/// for verifying the config.
///
/// This uses the type-state pattern to allow for methods to only take explicitly verified
/// configuration structs.

/// The maximum allowed size, in bytes, of a twitter keyword
const KEYWORD_MAX_LEN: usize = 60;

/// The minimum allowed size, in bytes, of a twitter keyword
const KEYWORD_MIN_LEN: usize = 1;

/// Error types that can arise from verifying a config
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("The following keywords are not between 1 and 60 bytes in length: {keywords:?}")]
    InvalidKeywords { keywords: Vec<String> },
}

/// An unverified config
#[derive(Clone, Debug)]
pub struct Config {
    pub config: Box<TwitterConfig>,
}

/// A verified config
#[derive(Clone, Debug)]
pub struct VerifiedConfig {
    config: Box<TwitterConfig>,
}

impl Config {
    /// Verify the config and potentially promote it to a `VerifiedConfig`
    ///
    /// This method will verify the Twitter configuration and promote it to a `VerifiedConfig` if
    /// all of the fields are valid. It checks each keyword to ensure that they are between 1 and
    /// 60 bytes (inclusive).
    pub fn verify(self) -> Result<VerifiedConfig, ConfigError> {
        // The Twitter spec states that each keyword must be between 1 and 60 bytes (inclusive),
        // which we verify by mapping over each keyword and filtering for bad keywords. We also
        // clone the strings because it allows the error to list which keywords were invalid while
        // outliving the original struct, which is consumed by this method.
        let invalid_keywords: Vec<String> = self
            .config
            .keywords
            .iter()
            .map(String::clone)
            .filter(|keyword| keyword.len() < KEYWORD_MIN_LEN || keyword.len() > KEYWORD_MAX_LEN)
            .collect();
        if !invalid_keywords.is_empty() {
            return Err(ConfigError::InvalidKeywords {
                keywords: invalid_keywords,
            });
        }
        Ok(VerifiedConfig {
            config: self.config,
        })
    }
}

impl VerifiedConfig {
    /// Retrieve the internal `TwitterConfig` struct
    ///
    /// This returns an immutable reference to the Twitter config struct
    pub fn get_config(&self) -> &TwitterConfig {
        &self.config
    }
}

/// The Twitter API configuration.
///
/// This includes the necessary data for queries and authentication.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TwitterConfig {
    /// The keywords to track
    pub keywords: Vec<String>,

    /// The API token
    pub access_token: String,

    /// The API token secret
    pub access_token_secret: String,

    /// The consumer API key
    pub consumer_key: String,

    /// The consumer API secret
    pub consumer_secret: String,
}

/// An application that streams tweets from Twitter and processes them for sentiment analysis
#[derive(Debug, StructOpt)]
pub struct CliOpts {
    /// The path to the configuration YAML file
    #[structopt(parse(from_os_str))]
    pub config_file: PathBuf,
}
