use crate::git::clone::clone;
use crate::util::structs::{RTPMConfig, RepositoryManifest};
use crate::util::utils::save_json_to_file;
use clap::ArgMatches;
use colored::*;
use std::path::PathBuf;

pub fn add_repository(matches: ArgMatches) {
    let repository: &str = matches.get_one::<String>("repository").unwrap_or_else(|| {
        println!("{}", "You have not filled a repository.".red().bold());
        std::process::exit(22);
    });
    let rtpm_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("rtpm.json");
    let mut rtpm_config: RTPMConfig = serde_json::from_str(
        &std::fs::read_to_string(rtpm_config_path.clone()).unwrap_or_else(|_| "{}".to_string()),
    )
    .unwrap();

    let repositories_path: PathBuf = dirs::data_dir().unwrap().join("rtop").join("repositories");
    let temp_path: PathBuf = repositories_path.join("temp");

    if temp_path.exists() {
        std::fs::remove_dir_all(&temp_path).unwrap();
    }
    std::fs::create_dir(&temp_path).unwrap();

    println!(":: {}", "Downloading the repository...".green());

    clone(repository.to_owned(), &*temp_path);

    let manifest_path: PathBuf = temp_path.join("manifest.json");

    if !manifest_path.exists() {
        println!(":: {}", "This is not a plugin repository!".red().bold());

        println!(":: {}", "Cleaning...".green());
        std::fs::remove_dir_all(temp_path).unwrap();
        println!(":: {}", "Cleaning completed!".green());

        println!(":: {}", "Exit...".blue());
        std::process::exit(22);
    }

    let repository_manifest: RepositoryManifest = serde_json::from_str(
        &std::fs::read_to_string(manifest_path).unwrap_or_else(|_| "{}".to_string()),
    )
    .unwrap();

    if rtpm_config.repositories.contains(&repository_manifest.id) {
        println!(
            ":: {}",
            "This repository is already installed!".red().bold()
        );

        println!(":: {}", "Cleaning...".green());
        std::fs::remove_dir_all(temp_path).unwrap();
        println!(":: {}", "Cleaning completed!".green());

        std::process::exit(22);
    }
    let new_path: PathBuf = repositories_path.join(repository_manifest.id.clone());

    std::fs::rename(temp_path, new_path).unwrap();

    println!(":: {}", "Linking repository to RTPM...".green());
    rtpm_config.repositories.push(repository_manifest.id);
    save_json_to_file(&rtpm_config, rtpm_config_path);
    println!(":: {}", "Plugin repository to RTPM!".green());

    println!(":: {}", "Repository added!".green().bold());
}
