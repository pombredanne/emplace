use crate::{config::Config, repo::Repo};
use anyhow::{anyhow, Context, Result};
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(not(unix))]
use std::os::windows::fs::symlink_file as symlink;
use std::{fs, path::Path};

pub fn link<T, R>(target_path: T, repository_path: R) -> Result<()>
where
    T: AsRef<Path>,
    R: AsRef<Path>,
{
    println!(
        "Converting {:?} to symbolic link {:?} in emplace repository.\n",
        target_path.as_ref(),
        repository_path.as_ref()
    );

    // Throw an error if the target path doesn't exist
    if !target_path.as_ref().exists() {
        return Err(anyhow!(
            "target file {:?} does not exist",
            target_path.as_ref()
        ));
    }

    // Get the full repository path
    let config = Config::from_default_file_or_new().context("retrieving config")?;
    let mut full_repository_path = config.folder_path();
    full_repository_path.push(repository_path.as_ref());

    // Throw an error if the repository path already exists
    if full_repository_path.exists() {
        return Err(anyhow!(
            "file {:?} already exists in emplace repository",
            full_repository_path
        ));
    }

    // Get the repository from the config
    let repo = Repo::new(config.clone(), true).context("opening repository")?;

    println!("Adding link information to configuration file");

    println!("Copying target file to emplace repository.");

    fs::copy(target_path.as_ref(), full_repository_path.as_ref()).context("copying target file to repository")?;

    println!("Removing original target file");

    fs::remove_file(target_path.as_ref()).context("removing original target file")?;

    println!("Creating symbolic link from emplace repository to target file location");

    if let Err(err) = symlink(full_repository_path, target_path) {
        fs::copy(full_repository_path, target_path).context("recovering from failed creating symbolic link")?;

        return Err(anyhow!("could not create symbolic link"));
    }

    println!("Pushing changes");

    Ok(())
}
