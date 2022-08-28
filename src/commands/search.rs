use crate::util::structs::{PluginManifest, RTPMConfig};
use crate::util::utils::search_plugin;
use clap::ArgMatches;
use colored::*;
use itertools::Itertools;
use std::path::PathBuf;

pub fn search(sub_matches: ArgMatches) {
    let plugins: Vec<String> = sub_matches
        .get_many::<String>("plugins")
        .unwrap_or_else(|| {
            println!(
                "{}",
                "You have not filled in any plugin or repository."
                    .red()
                    .bold()
            );
            std::process::exit(0);
        })
        .map(|s| s.to_owned())
        .unique()
        .collect();

    let rtpm_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("rtpm.json");
    let rtpm_config: RTPMConfig = serde_json::from_str(
        &std::fs::read_to_string(rtpm_config_path.clone()).unwrap_or_else(|_| "{}".to_string()),
    )
    .unwrap();

    let mut plugins_list: String = String::new();
    for plugin in plugins {
        let repository_path: PathBuf = if let Some(repository_path) = search_plugin(
            plugin.clone(),
            rtpm_config.clone(),
            rtpm_config_path.clone(),
            false,
        ) {
            repository_path
        } else {
            continue;
        };

        let plugin_manifest: PluginManifest = serde_json::from_str(
            &std::fs::read_to_string(
                repository_path
                    .join("plugins")
                    .join(format!("{}.json", plugin)),
            )
            .unwrap_or_else(|_| "{}".to_string()),
        )
        .unwrap();
        plugins_list.push_str(
            format!("{} - v{}\n", plugin_manifest.name, plugin_manifest.version).as_str(),
        );
    }
    if plugins_list.is_empty() {
        println!("{}", "No plugin was found.".red().bold())
    } else {
        println!("{}", plugins_list);
    }
}
