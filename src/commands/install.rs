use crate::git::clone::clone;
use crate::git::update_repositories::update_repositories;
use crate::util::structs::{
    PluginManifest, RTPMConfig, RTPMConfigPluginElement, RtopConfig, RtopConfigPlugins,
};
use crate::util::utils::{build_cargo_project, get_raw_url, save_json_to_file, search_plugin};
use clap::ArgMatches;
use colored::*;
use itertools::Itertools;
use std::fs::ReadDir;
use std::io::Write;
use std::path::PathBuf;
use url::Url;

fn remove_plugin(path_to_remove: PathBuf, rtop_config: PathBuf) {
    println!(
        ":: {}",
        format!(
            "The Rtop config file: {} does not exist, you must launch Rtop before using RtopUtil.",
            rtop_config.into_os_string().into_string().unwrap()
        )
        .red()
    );
    println!(
        ":: {}",
        "Cleaning the previously installed plugin...".green()
    );
    std::fs::remove_dir_all(path_to_remove).unwrap();
    println!(":: {}", "Cleaning finished, program exit...".green());
    std::process::exit(0);
}

fn install_plugin(plugin_manifest: PluginManifest) -> bool {
    let plugin_repository_path: PathBuf = dirs::data_dir()
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
    if plugin_repository_path.exists() {
        println!(":: {}", format!("The plugin {} by {} is already installed! You can use the {} command to update it.", plugin_manifest.name, author_string, "rtpm -Sud".bold()).red());
        return false;
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
    clone(
        plugin_manifest.url.clone(),
        plugin_repository_path.as_path(),
    );
    println!(
        ":: {}",
        "Launching the compilation of the plugin...\n".green()
    );

    let plugin_cargo_toml_path: PathBuf = dirs::data_dir()
        .unwrap()
        .join("rtop")
        .join("plugins")
        .join(plugin_manifest.id.clone())
        .join("Cargo.toml");

    build_cargo_project(plugin_cargo_toml_path);

    println!("\n:: {}", "Plugin compiled!".green());
    println!(":: {}", "Linking plugin to Rtop...".green());

    let rtop_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("config");
    if !rtop_config_path.exists() {
        remove_plugin(plugin_repository_path.clone(), rtop_config_path.clone());
    }
    let paths: ReadDir =
        std::fs::read_dir(plugin_repository_path.join("target").join("release")).unwrap();
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

    let mut rtop_config: RtopConfig = serde_json::from_str(
        &std::fs::read_to_string(rtop_config_path.clone()).unwrap_or_else(|_| "{}".to_string()),
    )
    .unwrap();
    rtop_config.plugins.push(RtopConfigPlugins {
        path: file_path,
        provided_widgets: plugin_manifest.provided_widgets,
    });
    save_json_to_file(&rtop_config, rtop_config_path);
    println!(":: {}", "Plugin linked to Rtop!".green());
    println!(":: {}", "Linking plugin to RTPM...".green());
    let rtpm_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("rtpm.json");
    let mut rtpm_config: RTPMConfig = serde_json::from_str(
        &std::fs::read_to_string(rtpm_config_path.clone()).unwrap_or_else(|_| "{}".to_string()),
    )
    .unwrap();
    rtpm_config.plugins.push(RTPMConfigPluginElement {
        id: plugin_manifest.id.clone(),
        name: plugin_manifest.name.clone(),
        version: plugin_manifest.version,
        repo: plugin_manifest.url,
    });
    save_json_to_file(&rtpm_config, rtpm_config_path);
    println!(":: {}", "Plugin linked to RTPM!".green());
    println!(":: {}", format!("The plugin {} is now installed! You can execute rtpm -Ip {} to get info about this plugin.", plugin_manifest.name, plugin_manifest.id).green());
    true
}

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
        .expect("You must enter a correct answer.");
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
        install_plugin(plugin_manifest);
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
        println!(":: {}", "Starting installation of plugin...".green());
    }

    let rtpm_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("rtpm.json");
    let rtpm_config: RTPMConfig = serde_json::from_str(
        &std::fs::read_to_string(rtpm_config_path.clone()).unwrap_or_else(|_| "{}".to_string()),
    )
    .unwrap();

    for plugin in plugins {
        println!(":: {}", format!("Searching plugin {}...", plugin).green());
        let repository_path_opt: Option<PathBuf> = search_plugin(
            plugin.clone(),
            rtpm_config.clone(),
            rtpm_config_path.clone(),
            true,
        );

        let repository_path: PathBuf = if let Some(repository_path) = repository_path_opt {
            repository_path
        } else {
            println!(
                ":: {}",
                format!("I couldn't find the {} plugin.", plugin).yellow()
            );
            break;
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
        install_plugin(plugin_manifest);
    }
    // println!(":: {}", "Exit...".green());
}

pub fn install(matches: ArgMatches) {
    let mut must_println: bool = false;
    if matches
        .get_one::<bool>("update")
        .expect("Defaulted by clap")
        .to_owned()
    {
        update_repositories();
        must_println = true;
    }
    if matches
        .get_one::<bool>("upgrade")
        .expect("Defaulted by clap")
        .to_owned()
    {
        // TODO implement upgrade
        std::process::exit(9);
    }

    let plugins: Vec<String> = matches
        .get_many::<String>("plugins")
        .unwrap_or_else(|| {
            std::process::exit(0);
        })
        .map(|s| s.to_owned())
        .unique()
        .collect();

    if must_println {
        println!();
    }

    if matches
        .get_one::<bool>("unsecure-git-url")
        .expect("Defaulted by clap")
        .to_owned()
    {
        install_insecure_plugins(plugins);
    } else {
        install_plugins(plugins);
    }
}
