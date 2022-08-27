use std::fs::DirEntry;
use crate::git::clone::clone;
use crate::git::pull::{do_fetch, do_merge};
use crate::util::structs::{RTPMConfig, RepositoryManifest};
use colored::*;
use git2::{Remote, Repository};
use std::path::PathBuf;

pub fn update_repositories() {
    println!(
        ":: {}",
        "Update of all Rtop plugin repositories...".green().bold()
    );
    let repositories_path: PathBuf = dirs::data_dir().unwrap().join("rtop").join("repositories");
    let mut must_update_rtop: bool = true;
    if !repositories_path.join("rtop").exists() {
        println!(
            ":: {}",
            "The official plugin repository is not present, start downloading it..."
                .green()
                .bold()
        );
        must_update_rtop = false;

        clone(
            "https://github.com/RtopRS/PluginsRepository/".to_owned(),
            &*repositories_path.join("rtop"),
        )
    }

    let config_path: PathBuf = dirs::config_dir()
        .unwrap()
        .join("rtop")
        .join("rtop-util.json");
    let mut rtop_config_json: RTPMConfig = serde_json::from_str(
        &std::fs::read_to_string(&config_path.clone()).unwrap_or_else(|_| "{}".to_string()),
    )
    .unwrap();

    for repository in std::fs::read_dir(repositories_path).unwrap() {
        let repository: &DirEntry = repository.as_ref().unwrap();

        let folder_name: String = repository
            .file_name()
            .into_string()
            .unwrap();
        if !must_update_rtop && repository.file_name() == "rtop" {
            continue;
        }
        let repo_manifest: RepositoryManifest = serde_json::from_str(
            &std::fs::read_to_string(&repository.path().join("manifest.json"))
                .unwrap_or_else(|_| "{}".to_string()),
        )
        .unwrap();
        println!(
            ":: {}",
            format!(
                "Updating the repository: {} ({})...",
                repo_manifest.name, repo_manifest.url
            )
            .green()
        );

        if !rtop_config_json.repositories.contains(&folder_name) {
            println!(
                ":: {}",
                "The repository is not present in the config, this one has been added."
                    .yellow()
                    .bold()
            );
            rtop_config_json.repositories.push(folder_name);
        }

        let repo: Repository = Repository::open(repository.path()).unwrap();
        let mut remote: Remote = repo.find_remote("origin").unwrap();
        let fetch_commit = do_fetch(&repo, &["main"], &mut remote).unwrap();
        do_merge(&repo, &"main", fetch_commit).unwrap();
        println!(
            ":: {}",
            format!(
                "Update of the repository: {} ({}) is terminated!",
                repo_manifest.name, repo_manifest.url
            )
            .green()
        );
    }
    std::fs::write(
        config_path,
        serde_json::to_string_pretty(&rtop_config_json).unwrap(),
    )
    .unwrap_or_else(|e| {
        println!(
            ":: {}",
            format!("An error occurred while writing to the Rtop file ({}).", e)
                .bold()
                .red()
        );
    });
    println!(
        ":: {}",
        "Update of all Rtop plugin repositories completed!"
            .green()
            .bold()
    );
}
