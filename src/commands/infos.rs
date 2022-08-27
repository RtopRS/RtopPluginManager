use crate::util::structs::{PluginManifest, RepositoryManifest};
use clap::ArgMatches;
use colored::*;
use itertools::Itertools;
use std::path::PathBuf;

fn repository_infos(repositories: Vec<String>) {
    if repositories.len() > 1 {
        println!(
            ":: {}",
            format!("Information about {} repositories\n", repositories.len())
                .yellow()
                .bold()
        );
    } else {
        println!(":: {}", "Information about repository\n".yellow().bold());
    }

    for repository in repositories {
        let repository_path: PathBuf = dirs::data_dir()
            .unwrap()
            .join("rtop")
            .join("repositories")
            .join(repository.clone());
        if !repository_path.exists() {
            println!(
                ":: {}",
                format!("The repository {} doesn't exist or is not available.", repository)
                    .red()
                    .bold()
            );
            continue;
        }

        let repo_manifest: RepositoryManifest = serde_json::from_str(
            &std::fs::read_to_string(repository_path.join("manifest.json"))
                .unwrap_or_else(|_| "{}".to_string()),
        )
        .unwrap();

        let mut fallback_url: String = repo_manifest
            .fallback_url
            .unwrap_or_else(|| "No".to_owned());
        if fallback_url.trim().is_empty() {
            fallback_url = "No".to_owned();
        }

        println!(
            "{}",
            format!(
                "{} {}\n{} {}\n{} {}\n{} {}\n",
                "Name         :".blue(),
                repo_manifest.name.yellow(),
                "Description  :".blue(),
                repo_manifest.description.yellow(),
                "URL          :".blue(),
                repo_manifest.url.yellow(),
                "Fallback URL :".blue(),
                fallback_url.yellow(),
            )
        );
    }
}

fn plugin_infos(plugins: Vec<String>) {
    if plugins.len() > 1 {
        println!(
            ":: {}",
            format!("Information about {} plugins\n", plugins.len())
                .yellow()
                .bold()
        );
    } else {
        println!(":: {}", "Information about plugin\n".yellow().bold());
    }

    for plugin in plugins {
        let plugin_path: PathBuf = dirs::data_dir()
            .unwrap()
            .join("rtop")
            .join("plugins")
            .join(plugin.clone());
        if !plugin_path.exists() {
            println!(
                ":: {}",
                format!("The plugin {} doesn't exist or is not available.", plugin)
                    .red()
                    .bold()
            );
            continue;
        }

        let plugin_manifest: PluginManifest = serde_json::from_str(
            &std::fs::read_to_string(plugin_path.join("manifest.json"))
                .unwrap_or_else(|_| "{}".to_string()),
        )
            .unwrap();

        let mut to_print: String = format!(
            "{} {}\n{} {}\n{} {}\n{} {}\n",
            "ID           :".blue(),
            plugin_manifest.id.yellow(),
            "Name         :".blue(),
            plugin_manifest.name.yellow(),
            "Description  :".blue(),
            plugin_manifest.description.yellow(),
            "Version      :".blue(),
            plugin_manifest.version.yellow(),
        );

        if let Some(author) = plugin_manifest.author {
            to_print.push_str(format!("{} {}\n", "Author       :".blue(), author.yellow()).as_str());
        } else if plugin_manifest.authors.is_some() && !plugin_manifest.authors.clone().unwrap().is_empty() {
            to_print.push_str(format!("{} {}\n", "Authors      :".blue(), plugin_manifest.authors.unwrap().join(", ").yellow()).as_str());
        }
        if let Some(license) = plugin_manifest.license {
            to_print.push_str(format!("{} {}\n", "License      :".blue(), license.yellow()).as_str());
        } else {
            to_print.push_str(format!("{} {}\n", "License      :".blue(), "No".yellow()).as_str());
        }
        if plugin_manifest.arch.is_some() && !plugin_manifest.arch.clone().unwrap().is_empty() {
            to_print.push_str(format!("{} {}\n", "Arch         :".blue(), plugin_manifest.arch.unwrap().join(", ").yellow()).as_str());
        } else {
            to_print.push_str(format!("{} {}\n", "Arch         :".blue(), "All".yellow()).as_str());
        }
        if plugin_manifest.os.is_some() && !plugin_manifest.os.clone().unwrap().is_empty() {
            to_print.push_str(format!("{} {}\n", "OS           :".blue(), plugin_manifest.os.unwrap().join(", ").yellow()).as_str());
        } else {
            to_print.push_str(format!("{} {}\n", "OS           :".blue(), "All".yellow()).as_str());
        }


        println!("{}", to_print);
    }
}

pub fn infos(sub_matches: ArgMatches) {
    let plugin_or_repository: Vec<String> = sub_matches
        .get_many::<String>("elements")
        .unwrap_or_else(|| {
            println!(
                "{}",
                "You have not filled in any plugin or repository."
                    .red()
                    .bold()
            );
            std::process::exit(22);
        })
        .map(|s| s.to_owned())
        .unique()
        .collect();

    if sub_matches
        .get_one::<bool>("repository")
        .expect("Defaulted by clap")
        .to_owned()
    {
        repository_infos(plugin_or_repository);
    } else if sub_matches
        .get_one::<bool>("plugin")
        .expect("Defaulted by clap")
        .to_owned()
    {
        plugin_infos(plugin_or_repository);
    }
}
