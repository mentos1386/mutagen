//! Provides ignore pattern matching functionality.

#[cfg(test)]
mod tests;

use glob::{Pattern, PatternError};

struct NegatablePattern {
    pattern: Pattern,
    negated: bool,
}

impl NegatablePattern {
    fn new(pattern: &str) -> Result<NegatablePattern, PatternError> {
        // Grab the character in the pattern (grab a space if the pattern is
        // empty - it'll return false for negation).
        let first = pattern.chars().next().unwrap_or(' ');

        // Determine if the pattern is negated and extract the un-negated
        // portion.
        // HACK: We rely on '!' being a single-byte character for the call to
        // split_at.
        let (pattern, negated) = if first == '!' {
            (pattern.split_at(1).1, true)
        } else {
            (pattern, false)
        };

        // Parse and validate the pattern.
        let pattern = Pattern::new(pattern)?;

        // Success.
        Ok(NegatablePattern{pattern: pattern, negated: negated})
    }
}

pub struct Ignorer {
    patterns: Vec<NegatablePattern>,
}

impl Ignorer {
    pub fn new<I, S>(patterns: I) -> Result<Ignorer, PatternError> where
        I: IntoIterator<Item=S>,
        S: AsRef<str> {
        // Parse the patterns.
        let patterns = patterns.into_iter()
                        .map(|s| NegatablePattern::new(s.as_ref()))
                        .collect::<Result<Vec<_>, PatternError>>()?;

        // Success.
        Ok(Ignorer{patterns: patterns})
    }

    pub fn ignored<S: AsRef<str>>(&self, path: S) -> bool {
        // Nothing is initially ignored.
        let mut ignored = false;

        // Run through patterns, keeping track of the ignored state as we reach
        // more specific rules.
        for p in self.patterns.iter() {
            // If there's no match, then this rule doesn't apply.
            if !p.pattern.matches(path.as_ref()) {
                continue;
            }

            // If we have a match, then change the ignored state based on
            // whether or not the pattern is negated.
            ignored = !p.negated;
        }

        // Done.
        ignored
    }
}
