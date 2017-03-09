//! Provides infrastructure for prompting.

/// The name of the environment variable that the prompting program should look
/// for to determine if it is being executed in prompt mode.
pub const PROMPTER_ENVIRONMENT_VARIABLE: &'static str = "MUTAGEN_PROMPTER";

/// The name of the environment variable from which the prompting program should
/// read the base-64 encoded message.
pub const PROMPTER_MESSAGE_BASE64_ENVIRONMENT_VARIABLE: &'static str = "MUTAGEN_PROMPTER_MESSAGE_BASE64";
