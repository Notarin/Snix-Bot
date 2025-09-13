use crate::args::ARGS;
use git2::build::RepoBuilder;
use git2::{FetchOptions, Repository};
use lazy_static::lazy_static;
use log::info;
use std::path::PathBuf;
use tempfile::env::temp_dir;
use tokio::sync::Mutex;

lazy_static! {
    pub(crate) static ref NixpkgsPath: PathBuf = temp_dir().join(PathBuf::from("nixpkgs"));
    pub(crate) static ref NixpkgsRepo: Mutex<Option<Repository>> = Mutex::new(None);
}

pub(crate) fn nixpkgs_repo() -> Repository {
    info!("Getting nixpkgs repo");
    match Repository::open(&*NixpkgsPath) {
        Ok(repository) => {
            info!("nixpkgs is already cloned! Using that.");
            repository
        }
        Err(_) => {
            info!("nixpkgs was not already cloned, cloning it.");
            clone_nixpkgs()
        }
    }
}

pub(crate) fn clone_nixpkgs() -> Repository {
    info!("Starting nixpkgs clone!");
    RepoBuilder::new()
        .fetch_options(clone_options())
        .clone(&ARGS.nixpkgs_url, &NixpkgsPath)
        .expect("Failed to clone nixpkgs!")
}

fn clone_options() -> FetchOptions<'static> {
    let mut clone_config: FetchOptions = FetchOptions::new();
    clone_config.depth(ARGS.clone_depth as i32);
    clone_config
}
