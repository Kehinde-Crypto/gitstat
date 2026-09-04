use crate::analyzer::Analyzer;
use crate::git::RepoStats;
use std::collections::HashMap;

pub struct ContributorAnalyzer {
    pub top_n: usize,
}

impl Analyzer for ContributorAnalyzer {
    fn name(&self) -> &'static str {
        "Top Contributors"
    }

    fn analyze(&self, stats: &[RepoStats]) -> String {
        let mut global_authors: HashMap<String, usize> = HashMap::new();

        for repo in stats {
            for (author, count) in &repo.authors {
                *global_authors.entry(author.clone()).or_insert(0) += count;
            }
        }

        let mut sorted_authors: Vec<(String, usize)> = global_authors.into_iter().collect();
        sorted_authors.sort_by(|a, b| b.1.cmp(&a.1));

        let mut output = String::new();
        output.push_str("Top Contributors:\n");

        for (author, count) in sorted_authors.into_iter().take(self.top_n) {
            output.push_str(&format!("  {}: {} commits\n", author, count));
        }

        output
    }
}