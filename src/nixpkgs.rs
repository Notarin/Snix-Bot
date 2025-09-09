use crate::args::ARGS;
use git2::build::RepoBuilder;
use git2::{FetchOptions, Repository};
use lazy_static::lazy_static;
use log::{debug, info};
use tempfile::{TempDir, tempdir};
use tokio::sync::Mutex;

lazy_static! {
    static ref NixpkgsDir: TempDir = {
        let dir: TempDir = tempdir().expect("Failed to create dir to place nixpkgs in!");
        debug!(
            "Temp path for nixpkgs repo chosen: {}",
            dir.path().display()
        );
        dir
    };
    static ref NixpkgsPath: &'static str = {
        NixpkgsDir
            .path()
            .to_str()
            .expect("For some bizarre reason, the tempdir for nixpkgs was not UTF-8")
    };
    pub(crate) static ref NixpkgsRepo: Mutex<Repository> = Mutex::new(clone_nixpkgs());
}

pub fn clone_nixpkgs() -> Repository {
    info!("Starting nixpkgs clone!");
    RepoBuilder::new()
        .fetch_options(clone_options())
        .clone(&ARGS.nixpkgs_url, NixpkgsDir.path())
        .expect("Failed to clone nixpkgs!")
}

fn clone_options() -> FetchOptions<'static> {
    let mut clone_config: FetchOptions = FetchOptions::new();
    clone_config.depth(ARGS.clone_depth as i32);
    clone_config
}
