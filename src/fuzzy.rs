use nucleo::{Config, Matcher, Utf32Str};

use crate::github::types::Repository;

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT.match_paths()),
        }
    }

    pub fn filter(&mut self, repos: &[Repository], query: &str) -> Vec<usize> {
        if query.is_empty() {
            return (0..repos.len()).collect();
        }

        let mut needle_buf = Vec::new();
        let needle = Utf32Str::new(query, &mut needle_buf);

        let mut scored: Vec<(usize, u16)> = repos
            .iter()
            .enumerate()
            .filter_map(|(idx, repo)| {
                let mut haystack_buf = Vec::new();
                let haystack = Utf32Str::new(&repo.full_name, &mut haystack_buf);

                self.matcher
                    .fuzzy_match(haystack, needle)
                    .map(|score| (idx, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(idx, _)| idx).collect()
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}
