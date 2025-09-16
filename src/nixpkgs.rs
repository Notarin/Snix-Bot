use crate::args::ARGS;
use git2::build::RepoBuilder;
use git2::{FetchOptions, Repository};
use log::info;
use std::path::PathBuf;
use std::sync::LazyLock;
use tempfile::env::temp_dir;
use tokio::sync::Mutex;

pub(crate) static NIXPKGS_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| temp_dir().join(PathBuf::from("nixpkgs")));
pub(crate) static NIXPKGS_REPO: LazyLock<Mutex<Option<Repository>>> =
    LazyLock::new(|| Mutex::new(None));

pub(crate) fn nixpkgs_repo() -> Repository {
    info!("Getting nixpkgs repo");
    if let Ok(repository) = Repository::open(&*NIXPKGS_PATH) {
        info!("nixpkgs is already cloned! Using that.");
        repository
    } else {
        info!("nixpkgs was not already cloned, cloning it.");
        clone_nixpkgs()
    }
}

pub(crate) fn clone_nixpkgs() -> Repository {
    info!("Starting nixpkgs clone!");
    RepoBuilder::new()
        .fetch_options(clone_options())
        .clone(&ARGS.nixpkgs_url, &NIXPKGS_PATH)
        .expect("Failed to clone nixpkgs!")
}

fn clone_options() -> FetchOptions<'static> {
    let mut clone_config: FetchOptions = FetchOptions::new();
    clone_config.depth(ARGS.clone_depth);
    clone_config
}
