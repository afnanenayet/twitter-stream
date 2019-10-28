/// A wrapper for the sentiment analysis library
///
/// The `sentiment` library provides a lot of extra/unnecessary information that is not relevant
/// for this application. This module defines wrapping types for the score, for example, and
/// provides routines that are relevant to the what this application needs.

/// A struct representing a sentiment score
///
/// A sentiment score has two elements: a positivity score and a negativity score, which indicates
/// how extreme the analyzed text was in either scenario.
#[derive(Debug, Clone, Copy)]
pub struct SentimentScore {
    /// The positivity score
    pub positive: f32,
    /// The negativity score
    pub negative: f32,
}

impl From<sentiment::Analysis> for SentimentScore {
    fn from(score: sentiment::Analysis) -> Self {
        Self {
            positive: score.positive.score,
            negative: score.negative.score,
        }
    }
}

/// Analyze the sentiment of some input
///
/// This will use the `sentiment::analyze_sentiment` method to calculate a sentiment score
pub fn analyze(input: &str) -> SentimentScore {
    let score = sentiment::analyze(input.to_string());
    SentimentScore::from(score)
}
