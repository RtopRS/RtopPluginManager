use crate::util::structs::RTPMConfig;
use crate::util::utils::{read_json_file, save_json_to_file};
use clap::ArgMatches;
use colored::Colorize;
use std::path::PathBuf;

pub fn remove_repository(matches: &ArgMatches) {
    let repository: &str = matches.get_one::<String>("repository").unwrap_or_else(|| {
        println!("{}", "You have not filled a repository.".red().bold());
        std::process::exit(22);
    });
    let rtpm_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("rtpm.json");
    let mut rtpm_config: RTPMConfig = read_json_file(&rtpm_config_path);
    if !rtpm_config.repositories.contains(&repository.to_owned()) {
        println!(":: {}", "This repository is not installed!".red().bold());
        std::process::exit(9);
    }

    let repository_path: PathBuf = dirs::data_dir()
        .unwrap()
        .join("rtop")
        .join("repositories")
        .join(repository);
    if repository_path.exists() {
        println!(":: {}", "Deleting repository folder...".green());
        std::fs::remove_dir_all(repository_path).unwrap();
        println!(":: {}", "Repository folder deleted!".green());
    }

    let rtpm_repository_index: usize = rtpm_config
        .repositories
        .iter()
        .position(|r| r == repository)
        .unwrap();
    rtpm_config.repositories.remove(rtpm_repository_index);

    save_json_to_file(&rtpm_config, rtpm_config_path);

    println!(":: {}", "Repository removed!".green().bold());
}
