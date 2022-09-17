use crate::util::structs::{PluginManifest, RTPMConfig};
use crate::util::utils::{read_json_file, search_plugin};
use clap::ArgMatches;
use colored::Colorize;
use itertools::Itertools;
use std::path::PathBuf;

pub fn search(matches: &ArgMatches) {
    let plugins: Vec<String> = matches
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
        .cloned()
        .unique()
        .collect();

    let rtpm_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("rtpm.json");
    let rtpm_config: RTPMConfig = read_json_file(&rtpm_config_path);

    let mut plugins_list: String = String::new();
    for plugin in plugins {
        let repository_path: PathBuf = if let Some(repository_path) = search_plugin(
            plugin.as_str(),
            rtpm_config.clone(),
            &rtpm_config_path,
            false,
        ) {
            repository_path
        } else {
            continue;
        };

        let plugin_manifest: PluginManifest = read_json_file(
            &repository_path
                .join("plugins")
                .join(format!("{}.json", plugin)),
        );
        plugins_list.push_str(
            format!("{} - v{}\n", plugin_manifest.name, plugin_manifest.version).as_str(),
        );
    }
    if plugins_list.is_empty() {
        println!("{}", "No plugin was found.".red().bold());
    } else {
        println!("{}", plugins_list);
    }
}
