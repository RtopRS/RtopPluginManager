use crate::git::clone::clone;
use crate::git::update_repositories::update_repositories;
use crate::util::structs::{PluginManifest, RepositoryPlugin, RtopConfig, RtopConfigPlugins, RTPMConfig};
use crate::util::utils::{build_cargo_project, get_raw_url, remove_plugin};
use clap::ArgMatches;
use colored::*;
use itertools::Itertools;
use std::io::Write;
use std::path::PathBuf;
use url::Url;

fn install_insecure_plugins(plugins: Vec<String>) {
    println!(
        ":: {}",
        "Be very careful, using plugins that are not in the official Rtop repos can be dangerous. Rtop is not responsible for any damage that may be caused by these plugins.".yellow().bold()
    );
    print!(":: {} ", "Do you really want to continue? (y/n)".purple());
    let _ = std::io::stdout().flush();
    let mut user_response: String = String::new();
    std::io::stdin()
        .read_line(&mut user_response)
        .expect("Did not enter a correct string");
    if !vec!["y", "yes", "ok", "o"].contains(&user_response.trim().to_lowercase().as_str()) {
        println!(":: {}", "Exiting...".blue());
        std::process::exit(0);
    }

    for plugin in plugins {
        println!(
            ":: {}",
            format!("Get the manifest for the repo: {}...", plugin).green()
        );

        let url: Url = if let Ok(url) = Url::parse(&*plugin) {
            url
        } else {
            continue;
        };

        let raw_url: Url = if let Some(url) = get_raw_url(url) {
            url
        } else {
            continue;
        };
        let manifest_url: Url = raw_url.join("manifest.json").unwrap();

        let manifest_resp = reqwest::blocking::get(manifest_url.to_owned())
            .unwrap()
            .json::<PluginManifest>();

        let plugin_manifest: PluginManifest = if let Ok(manifest) = manifest_resp {
            manifest
        } else {
            println!(":: {}", format!("The manifest of the plugin {} is wrong, please contact the author of this plugin to ask him to change it.", plugin).red().bold());
            continue;
        };
        println!(":: {}", "Manifest recovered!".green());
        let rtop_util_config_path: std::path::PathBuf = dirs::data_dir()
            .unwrap()
            .join("rtop")
            .join("plugins")
            .join(plugin_manifest.id.clone());
        let author_string: String = if plugin_manifest.authors.is_some()
            && !plugin_manifest.authors.clone().unwrap().is_empty()
        {
            plugin_manifest.authors.clone().unwrap().join(", ")
        } else if let Some(author) = plugin_manifest.author.clone() {
            author
        } else {
            "an unknown".to_owned()
        };
        if rtop_util_config_path.exists() {
            println!(":: {}", format!("The plugin {} by {} is already installed! You can use the update command to update it.", plugin_manifest.name, author_string).red());
            continue;
        } else {
            println!(
                ":: {}",
                format!(
                    "Starting the recovery of the repo for the plugin {} by {} (v{})...",
                    plugin_manifest.name, author_string, plugin_manifest.version
                )
                .green()
            );
        }
        clone(plugin, rtop_util_config_path.as_path());
        println!(
            ":: {}",
            "Launching the compilation of the plugin...\n".green()
        );

        let plugin_cargo_toml: std::path::PathBuf = dirs::data_dir()
            .unwrap()
            .join("rtop")
            .join("plugins")
            .join(plugin_manifest.id.clone())
            .join("Cargo.toml");

        build_cargo_project(plugin_cargo_toml);

        println!("\n:: {}", "Plugin compiled!".green());
        println!(":: {}", "Linking plugin to Rtop...".green());

        let rtop_config: std::path::PathBuf =
            dirs::config_dir().unwrap().join("rtop").join("config");
        if !rtop_config.exists() {
            remove_plugin(rtop_util_config_path.clone(), rtop_config.clone());
        }
        let paths =
            std::fs::read_dir(rtop_util_config_path.join("target").join("release")).unwrap();
        let mut file_path: String = String::new();
        for path in paths {
            let path_un = path.unwrap().path();
            let extension = path_un.extension();
            if let Some(extension) = extension {
                if vec!["dll", "so"].contains(&extension.to_str().unwrap()) {
                    file_path = path_un.into_os_string().into_string().unwrap();
                }
            }
        }

        let mut rtop_config_json: RtopConfig = serde_json::from_str(
            &std::fs::read_to_string(rtop_config.clone()).unwrap_or_else(|_| "{}".to_string()),
        )
        .unwrap();
        rtop_config_json.plugins.push(RtopConfigPlugins {
            path: file_path,
            provided_widgets: plugin_manifest.provided_widgets,
        });
        let rtop_config_json_prettified: String =
            serde_json::to_string_pretty(&rtop_config_json).unwrap();
        std::fs::write(rtop_config.clone(), rtop_config_json_prettified).unwrap_or_else(|e| {
            println!(
                ":: {}",
                format!("An error occurred while writing to the Rtop file ({}).", e)
                    .bold()
                    .red()
            );
            remove_plugin(rtop_util_config_path, rtop_config);
        });
        println!(":: {}", "Plugin linked to Rtop!".green());
        println!(":: {}", format!("The plugin {} is now installed! You can execute rtop-util -I {} to get info about this plugin.", plugin_manifest.name, plugin_manifest.id).green());
    }
    println!(":: {}", "Exit...".green());
}

fn install_plugins(plugins: Vec<String>) {
    if plugins.len() > 1 {
        println!(
            ":: {}",
            format!("Starting installation of {} plugins...", plugins.len()).green()
        );
    } else {
        println!(
            ":: {}",
            "Starting installation of plugin...".green()
        );
    }

    let mut rtop_config_json: RTPMConfig = serde_json::from_str(
        &std::fs::read_to_string(dirs::config_dir().unwrap().join("rtop").join("rtop-util.json")).unwrap_or_else(|_| "{}".to_string()),
    ).unwrap();

    for plugin in plugins {
        println!(
            ":: {}",
            format!("Searching plugin {}...", plugin).green()
        );
        let mut repository_path_opt: Option<PathBuf> = None;
        for repository in rtop_config_json.repositories.clone() {
            let path: PathBuf = dirs::data_dir().unwrap().join("rtop").join("repositories").join(repository.clone());
            if !path.exists() {
                 println!(":: {}", "The repository shit is not or no longer present, I delete it.".yellow());
                let index: usize = rtop_config_json.repositories.iter().position(|r| r == &repository).unwrap();
                rtop_config_json.repositories.remove(index);
            }
            let repository_plugins: RepositoryPlugin = serde_json::from_str(
                &std::fs::read_to_string(path.join("plugins.json")).unwrap_or_else(|_| "{}".to_string()),
            ).unwrap();
            if repository_plugins.plugins.contains(&plugin) {
                println!(
                    ":: {}",
                    format!("Plugin found in the repository {}!", repository).green()
                );
                repository_path_opt = Option::from(path);
                break
            }
        }
        let repository_path: PathBuf = if let Some(repository_path) = repository_path_opt {
            repository_path
        } else {
            println!(
                ":: {}",
                format!("I couldn't find the {} plugin.", plugin).yellow()
            );
            break
        };


    //     let plugin_manifest: PluginManifest = if let Ok(manifest) = manifest_resp {
    //         manifest
    //     } else {
    //         println!(":: {}", format!("The manifest of the plugin {} is wrong, please contact the author of this plugin to ask him to change it.", plugin).red().bold());
    //         continue;
    //     };
    //     println!(":: {}", "Manifest recovered!".green());
    //     let rtop_util_config_path: std::path::PathBuf = dirs::data_dir()
    //         .unwrap()
    //         .join("rtop")
    //         .join("plugins")
    //         .join(plugin_manifest.id.clone());
    //     let author_string: String = if plugin_manifest.authors.is_some()
    //         && !plugin_manifest.authors.clone().unwrap().is_empty()
    //     {
    //         plugin_manifest.authors.clone().unwrap().join(", ")
    //     } else if let Some(author) = plugin_manifest.author.clone() {
    //         author
    //     } else {
    //         "an unknown".to_owned()
    //     };
    //     if rtop_util_config_path.exists() {
    //         println!(":: {}", format!("The plugin {} by {} is already installed! You can use the update command to update it.", plugin_manifest.name, author_string).red());
    //         continue;
    //     } else {
    //         println!(
    //             ":: {}",
    //             format!(
    //                 "Starting the recovery of the repo for the plugin {} by {} (v{})...",
    //                 plugin_manifest.name, author_string, plugin_manifest.version
    //             )
    //                 .green()
    //         );
    //     }
    //     clone(plugin, rtop_util_config_path.as_path());
    //     println!(
    //         ":: {}",
    //         "Launching the compilation of the plugin...\n".green()
    //     );
    //
    //     let plugin_cargo_toml: std::path::PathBuf = dirs::data_dir()
    //         .unwrap()
    //         .join("rtop")
    //         .join("plugins")
    //         .join(plugin_manifest.id.clone())
    //         .join("Cargo.toml");
    //
    //     build_cargo_project(plugin_cargo_toml);
    //
    //     println!("\n:: {}", "Plugin compiled!".green());
    //     println!(":: {}", "Linking plugin to Rtop...".green());
    //
    //     let rtop_config: std::path::PathBuf =
    //         dirs::config_dir().unwrap().join("rtop").join("config");
    //     if !rtop_config.exists() {
    //         remove_plugin(rtop_util_config_path.clone(), rtop_config.clone());
    //     }
    //     let paths =
    //         std::fs::read_dir(rtop_util_config_path.join("target").join("release")).unwrap();
    //     let mut file_path: String = String::new();
    //     for path in paths {
    //         let path_un = path.unwrap().path();
    //         let extension = path_un.extension();
    //         if let Some(extension) = extension {
    //             if vec!["dll", "so"].contains(&extension.to_str().unwrap()) {
    //                 file_path = path_un.into_os_string().into_string().unwrap();
    //             }
    //         }
    //     }
    //
    //     let mut rtop_config_json: RtopConfig = serde_json::from_str(
    //         &std::fs::read_to_string(rtop_config.clone()).unwrap_or_else(|_| "{}".to_string()),
    //     )
    //         .unwrap();
    //     rtop_config_json.plugins.push(RtopConfigPlugins {
    //         path: file_path,
    //         provided_widgets: plugin_manifest.provided_widgets,
    //     });
    //     let rtop_config_json_prettified: String =
    //         serde_json::to_string_pretty(&rtop_config_json).unwrap();
    //     std::fs::write(rtop_config.clone(), rtop_config_json_prettified).unwrap_or_else(|e| {
    //         println!(
    //             ":: {}",
    //             format!("An error occurred while writing to the Rtop file ({}).", e)
    //                 .bold()
    //                 .red()
    //         );
    //         remove_plugin(rtop_util_config_path, rtop_config);
    //     });
    //     println!(":: {}", "Plugin linked to Rtop!".green());
    //     println!(":: {}", format!("The plugin {} is now installed! You can execute rtop-util -I {} to get info about this plugin.", plugin_manifest.name, plugin_manifest.id).green());
    }
    // println!(":: {}", "Exit...".green());
}

pub fn install(sub_matches: ArgMatches) {
    if sub_matches
        .get_one::<bool>("update")
        .expect("Defaulted by clap")
        .to_owned()
    {
        update_repositories();
    }
    if sub_matches
        .get_one::<bool>("upgrade")
        .expect("Defaulted by clap")
        .to_owned()
    {
        // TODO implement upgrade
        std::process::exit(9);
    }

    let plugins: Vec<String> = sub_matches
        .get_many::<String>("plugins")
        .unwrap_or_else(|| {
            println!("{}", "You have not filled in any plugins.".red().bold());
            std::process::exit(22);
        })
        .map(|s| s.to_owned())
        .unique()
        .collect();

    if sub_matches
        .get_one::<bool>("unsecure-git-url")
        .expect("Defaulted by clap")
        .to_owned()
    {
        install_insecure_plugins(plugins);
    } else {
        install_plugins(plugins);
    }
}
