use crate::git::clone::clone;
use crate::git::pull::{do_fetch, do_merge};
use crate::util::structs::{RTPMConfig, RepositoryManifest};
use crate::util::utils::{read_json_file, save_json_to_file};
use colored::Colorize;
use git2::{AnnotatedCommit, Remote, Repository};
use std::fs::DirEntry;
use std::path::PathBuf;

pub fn update_repositories() {
    println!(
        ":: {}",
        "Update of all Rtop plugin repositories...\n".green().bold()
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
            "https://github.com/RtopRS/PluginsRepository/",
            &repositories_path.join("rtop"),
        );
    }

    let rtpm_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("rtpm.json");
    let mut rtpm_config: RTPMConfig = read_json_file(&rtpm_config_path);

    for repository_result in std::fs::read_dir(repositories_path).unwrap() {
        let repository: &DirEntry = repository_result.as_ref().unwrap();

        let folder_name: String = repository.file_name().into_string().unwrap();
        if !must_update_rtop && repository.file_name() == "rtop" {
            continue;
        }
        let repo_manifest: RepositoryManifest =
            read_json_file(&repository.path().join("manifest.json"));
        println!(
            ":: {}",
            format!(
                "Updating the repository: {} ({})...",
                repo_manifest.name.bold(),
                repo_manifest.url
            )
            .green()
        );

        if !rtpm_config.repositories.contains(&folder_name) {
            println!(
                ":: {}",
                "The repository is not present in the config, this one has been added."
                    .yellow()
                    .bold()
            );
            rtpm_config.repositories.push(folder_name);
        }

        let repo: Repository = Repository::open(repository.path()).unwrap();
        let mut remote: Remote = repo.find_remote("origin").unwrap();
        let fetch = do_fetch(&repo, &["main"], &mut remote).0;
        let fetch_commit: AnnotatedCommit = if let Err(error) = fetch {
            println!(
                ":: {}",
                format!(
                    "An error occurred while fetching the repository: {}",
                    error.message()
                )
                .red()
                .bold()
            );
            continue;
        } else {
            fetch.unwrap()
        };
        if let Err(error) = do_merge(&repo, "main", &fetch_commit) {
            if error.message() == "no merge base found" {
                println!(
                    ":: {}",
                    "Unable to update the repository, re-installation..."
                        .red()
                        .bold()
                );
                std::fs::remove_dir_all(repository.path()).unwrap();
                clone(&repo_manifest.url, &repository.path());
                println!(":: {}", "Repository re-installed!".green());
            } else {
                println!(
                    ":: {}",
                    format!(
                        "An error occurred while merging the repository: {}",
                        error.message()
                    )
                    .red()
                    .bold()
                );
            }
            continue;
        }
        println!(
            ":: {}",
            format!(
                "Update of the repository: {} ({}) is terminated!\n",
                repo_manifest.name.bold(),
                repo_manifest.url
            )
            .green()
        );
    }

    save_json_to_file(&rtpm_config, rtpm_config_path);

    println!(
        ":: {}",
        "Update of all Rtop plugin repositories completed!"
            .green()
            .bold()
    );
}
