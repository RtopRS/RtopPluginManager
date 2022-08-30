use crate::git::pull::{do_fetch, do_merge};
use crate::util::structs::{PluginManifest, RTPMConfig};
use crate::util::utils::{save_json_to_file, search_plugin};
use colored::*;
use git2::{Remote, Repository};
use std::fs::DirEntry;
use std::path::PathBuf;

pub fn update_packages() {
    println!(":: {}", "Update of all Rtop plugins...\n".green().bold());
    let plugins_path: PathBuf = dirs::data_dir().unwrap().join("rtop").join("plugins");
    let rtpm_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("rtpm.json");
    let mut rtpm_config: RTPMConfig = serde_json::from_str(
        &std::fs::read_to_string(&rtpm_config_path).unwrap_or_else(|_| "{}".to_string()),
    )
    .unwrap();

    for plugin in std::fs::read_dir(plugins_path).unwrap() {
        let plugin: &DirEntry = plugin.as_ref().unwrap();
        let folder_name: String = plugin.file_name().into_string().unwrap();
        let plugin_index: usize = if let Some(plugin_index) =
            rtpm_config.plugins.iter().position(|r| r.id == folder_name)
        {
            plugin_index
        } else {
            continue;
        };
        let plugin_manifest_path: PathBuf = if rtpm_config.plugins[plugin_index].plugin_type == 0 {
            if let Some(repository_path) = search_plugin(
                folder_name.clone(),
                rtpm_config.clone(),
                rtpm_config_path.clone(),
                false,
            ) {
                repository_path
                    .join("plugins")
                    .join(format!("{}.json", folder_name))
            } else {
                println!(
                    ":: {}",
                    format!(
                        "The plugin {} doesn't exist or is not available.",
                        folder_name
                    )
                    .red()
                    .bold()
                );
                continue;
            }
        } else {
            plugin.path().join("manifest.json")
        };

        let plugin_manifest: PluginManifest = serde_json::from_str(
            &std::fs::read_to_string(plugin_manifest_path.clone())
                .unwrap_or_else(|_| "{}".to_string()),
        )
        .unwrap();
        println!(
            ":: {}",
            format!(
                "Updating the plugin: {} ({})...",
                plugin_manifest.name.bold(),
                plugin_manifest.url
            )
            .green()
        );

        let git_repository: Repository = Repository::open(plugin.path()).unwrap();
        let mut remote: Remote = git_repository.find_remote("origin").unwrap();
        let fetch_commit = do_fetch(&git_repository, &["main"], &mut remote).unwrap();
        do_merge(&git_repository, "main", fetch_commit).unwrap();
        println!(
            ":: {}",
            format!(
                "Update of the plugin: {} ({}) is terminated!\n",
                plugin_manifest.name.bold(),
                plugin_manifest.url
            )
            .green()
        );
        let new_plugin_manifest: PluginManifest = serde_json::from_str(
            &std::fs::read_to_string(plugin_manifest_path).unwrap_or_else(|_| "{}".to_string()),
        )
        .unwrap();
        if plugin_manifest.version != new_plugin_manifest.version {
            rtpm_config.plugins[plugin_index].version = new_plugin_manifest.version;
        }
    }
    save_json_to_file(&rtpm_config, rtpm_config_path);
    println!(
        ":: {}",
        "Update of all Rtop plugins completed!".green().bold()
    );
}
