use clap::Parser;
use git_setup_rs::{Args, Result};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Create dependencies
    let profile_manager = Arc::new(git_setup_rs::profile::manager::ProfileManagerImpl::new());
    let git_wrapper = Arc::new(git_setup_rs::external::git::SystemGitWrapper::new());
    let onepassword_wrapper = Arc::new(git_setup_rs::external::onepassword::SystemOnePasswordWrapper::new());
    let gpg_wrapper = Arc::new(git_setup_rs::external::gpg::SystemGpgWrapper::new());
    let profile_detector = Arc::new(git_setup_rs::detection::AutoDetector::new());
    let fuzzy_matcher = Arc::new(git_setup_rs::matching::ProfileFuzzyMatcher::new());

    // Create command handler
    let handler = git_setup_rs::commands::handlers::CommandHandlerBuilder::new()
        .with_profile_manager(profile_manager)
        .with_git_wrapper(git_wrapper)
        .with_onepassword_wrapper(onepassword_wrapper)
        .with_gpg_wrapper(gpg_wrapper)
        .with_profile_detector(profile_detector)
        .with_fuzzy_matcher(fuzzy_matcher)
        .build()?;

    // Execute the command
    handler.execute(&args).await
}
