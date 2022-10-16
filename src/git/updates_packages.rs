use crate::git::clone::clone;
use crate::git::pull::{do_fetch, do_merge};
use crate::util::structs::{PluginManifest, RTPMConfig, RtopConfig};
use crate::util::utils::{
    build_cargo_project, read_json_file, save_json_to_file, search_plugin, user_input_choice,
};
use colored::Colorize;
use git2::{AnnotatedCommit, Remote, Repository};
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

pub fn update_packages() {
    println!(":: {}", "Update of all Rtop plugins...\n".green().bold());
    let plugins_path: PathBuf = dirs::data_dir().unwrap().join("rtop").join("plugins");
    let base_rtop_path: PathBuf = dirs::config_dir().unwrap().join("rtop");
    let rtop_config_path: PathBuf = base_rtop_path.join("config.json");
    let mut rtop_config: RtopConfig = read_json_file(&rtop_config_path);
    let rtpm_config_path: PathBuf = base_rtop_path.join("rtpm.json");
    let mut rtpm_config: RTPMConfig = read_json_file(&rtpm_config_path);

    for plugin_result in std::fs::read_dir(plugins_path.clone()).unwrap() {
        let plugin: &DirEntry = plugin_result.as_ref().unwrap();
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
                folder_name.as_str(),
                rtpm_config.clone(),
                &rtpm_config_path,
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

        let plugin_manifest: PluginManifest = read_json_file(&plugin_manifest_path);
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
        let fetch = do_fetch(&git_repository, &["main"], &mut remote);
        let need_compilation: bool;
        let fetch_commit: AnnotatedCommit = if fetch.0.is_err() {
            println!(
                ":: {}",
                format!(
                    "An error occurred while fetching the plugin: {}",
                    fetch.0.err().unwrap().message()
                )
                .red()
                .bold()
            );
            continue;
        } else {
            need_compilation = fetch.1;
            fetch.0.unwrap()
        };
        if !need_compilation {
            println!();
            continue;
        }

        if let Err(error) = do_merge(&git_repository, "main", &fetch_commit) {
            if error.message() == "no merge base found" {
                println!(
                    ":: {}",
                    "Unable to update the plugin, re-installation..."
                        .red()
                        .bold()
                );
                std::fs::remove_dir_all(plugin.path()).unwrap();
                clone(&plugin_manifest.url, &plugin.path());
                println!(":: {}", "Plugin re-installed!".green());
            } else {
                println!(
                    ":: {}",
                    format!(
                        "An error occurred while merging the plugin: {}",
                        error.message()
                    )
                    .red()
                    .bold()
                );
            }
            continue;
        }

        println!(":: {}", "Plugin updated, compilation...".green());

        let plugin_cargo_toml_path: PathBuf = dirs::data_dir()
            .unwrap()
            .join("rtop")
            .join("plugins")
            .join(plugin_manifest.id.clone())
            .join("Cargo.toml");

        println!(":: {}", "Backup previous executable...".green());

        let rtop_plugin_index: usize = rtop_config
            .plugins
            .iter()
            .position(|r| {
                r.path.starts_with(
                    &plugin
                        .path()
                        .join("target")
                        .into_os_string()
                        .into_string()
                        .unwrap(),
                )
            })
            .unwrap();
        let shared_lib_path: String = rtop_config.plugins[rtop_plugin_index].path.clone();
        let shared_lib_temp_path: PathBuf = plugins_path
            .clone()
            .join(Path::new(&shared_lib_path).file_name().unwrap());
        std::fs::copy(shared_lib_path.clone(), shared_lib_temp_path.clone()).unwrap();

        let cargo_build: bool = build_cargo_project(&plugin_cargo_toml_path);
        if !cargo_build {
            println!(":: {}", "An error occurred during compilation!".red());
            print!(
                ":: {} ",
                "Do you want to keep the old version anyway? (y/n)".purple()
            );
            if user_input_choice() {
                println!(":: {}", "Recovery of the plugin backup...".green());
                std::fs::rename(shared_lib_temp_path, shared_lib_path).unwrap();
                println!(":: {}", "Backup recovered!".green());
            } else {
                println!(
                    ":: {}",
                    "Cleaning the previously installed plugin...".green()
                );
                let rtpm_plugin_index: usize = rtpm_config
                    .plugins
                    .iter()
                    .position(|r| r.id == plugin_manifest.id)
                    .unwrap();
                rtpm_config.plugins.remove(rtpm_plugin_index);

                let rtop_plugin_index: usize = rtop_config
                    .plugins
                    .iter()
                    .position(|r| {
                        r.path.starts_with(
                            &plugin
                                .path()
                                .join("target")
                                .into_os_string()
                                .into_string()
                                .unwrap(),
                        )
                    })
                    .unwrap();
                rtop_config.plugins.remove(rtop_plugin_index);
                std::fs::remove_dir_all(plugin.path()).unwrap();
                save_json_to_file(&rtpm_config, rtpm_config_path.clone());
                save_json_to_file(&rtop_config, rtop_config_path.clone());
                std::fs::remove_file(shared_lib_temp_path).unwrap();
            }
            continue;
        }

        println!(
            ":: {}",
            format!(
                "Update of the plugin: {} ({}) is terminated!\n",
                plugin_manifest.name.bold(),
                plugin_manifest.url
            )
            .green()
        );
        let new_plugin_manifest: PluginManifest = read_json_file(&plugin_manifest_path);
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
