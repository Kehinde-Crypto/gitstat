use crate::git::RepoStats;

pub trait Analyzer {
    fn name(&self) -> &'static str;
    fn analyze(&self, stats: &[RepoStats]) -> String;
}