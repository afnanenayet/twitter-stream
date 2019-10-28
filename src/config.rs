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

const CONSUMER_API_KEY: &'static str = "Aw6ine4jxRWH6jsVQ1EWkcsqa";
const CONSUMER_SECRET_KEY: &'static str = "nRusLxGsVcdmio50RiEEqmxhORsztjRJt5ACMHkTQItUGodOpI";
const ACCESS_TOKEN: &'static str = "3070573521-aOEmeEsbcpWdcJGX7Yq1Dcl0LRcuoPd1pqqj8SM";
const ACCESS_TOKEN_SECRET: &'static str = "1iZI5H7lfdINWlfIBKp7THhnDNl1CVk3q5hMuKOWZCVAF";

/// Error types that can arise from verifying a config
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("The following keywords are not between 1 and 60 bytes in length: {keywords:?}")]
    InvalidKeywords { keywords: Vec<String> },

    #[error("The supplied API token did not authenticate correctly")]
    InvalidKey,
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
    /// all of the fields are valid. It will check each keyword and ensure that they are between 1
    /// and 60 bytes (inclusive), and that the keys for API authentication are valid.
    pub fn verify(mut self) -> Result<VerifiedConfig, ConfigError> {
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

        // TODO move this to the env, add error checking
        self.config.access_token = ACCESS_TOKEN.to_owned();
        self.config.access_token_secret = ACCESS_TOKEN_SECRET.to_owned();
        self.config.consumer_key = CONSUMER_API_KEY.to_owned();
        self.config.consumer_secret = CONSUMER_SECRET_KEY.to_owned();

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
#[derive(Clone, Debug, Default)]
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

/// Data relating to the command line interface
#[derive(Debug, StructOpt)]
pub struct CliOpts {
    /// The path to the configuration file for the Twitter API
    #[structopt(parse(from_os_str))]
    config_file: PathBuf,
}
