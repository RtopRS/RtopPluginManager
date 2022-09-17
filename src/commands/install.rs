use crate::git::clone::clone;
use crate::git::update_repositories::update_repositories;
use crate::git::updates_packages::update_packages;
use crate::util::structs::{
    PluginManifest, RTPMConfig, RTPMConfigPluginElement, RtopConfig, RtopConfigPlugins,
};
use crate::util::utils::{
    build_cargo_project, contain_clap_arg, get_raw_url, read_json_file, save_json_to_file,
    search_plugin, user_input_choice, verify_device_specification,
};
use clap::ArgMatches;
use colored::Colorize;
use itertools::Itertools;
use std::fs::ReadDir;
use std::path::PathBuf;
use url::Url;

fn install_plugin(plugin_manifest: PluginManifest, plugin_type: i8) -> bool {
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

    if !verify_device_specification(&plugin_manifest) {
        println!(
            ":: {}",
            "The author of this plugin has excluded your OS or architecture from the compatibility list.".yellow().bold()
        );
        print!(
            ":: {} ",
            "You can still continue if you wish (the compilation of the plugin may fail) (y/n)"
                .purple()
        );
        if !user_input_choice() {
            println!(":: {}", "Exiting...".blue());
            std::process::exit(0);
        }
    }

    if plugin_repository_path.exists() {
        println!(":: {}", format!("The plugin {} by {} is already installed! You can use the {} command to update it.", plugin_manifest.name, author_string, "rtpm -Sud".bold()).red());
        return false;
    }
    println!(
        ":: {}",
        format!(
            "Starting the recovery of the repo for the plugin {} by {} (v{})...",
            plugin_manifest.name, author_string, plugin_manifest.version
        )
        .green()
    );
    clone(&plugin_manifest.url, plugin_repository_path.as_path());
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

    let cargo_build: bool = build_cargo_project(&plugin_cargo_toml_path);
    if !cargo_build {
        println!(":: {}", "An error occurred during compilation!".red());
        println!(
            ":: {}",
            "Cleaning the previously installed plugin...".green()
        );
        std::fs::remove_dir_all(plugin_repository_path).unwrap();
        println!(":: {}", "Cleaning finished, program exit...".green());
        std::process::exit(0);
    }

    println!("\n:: {}", "Plugin compiled!".green());
    println!(":: {}", "Linking plugin to Rtop...".green());

    let rtop_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("config.json");
    if !rtop_config_path.exists() {
        println!(
            ":: {}",
            format!(
                "The Rtop config file: {} does not exist, you must launch Rtop before using RtopPluginManager.",
                rtop_config_path.into_os_string().into_string().unwrap()
            )
                .red()
        );
        println!(
            ":: {}",
            "Cleaning the previously installed plugin...".green()
        );
        std::fs::remove_dir_all(plugin_repository_path).unwrap();
        println!(":: {}", "Cleaning finished, program exit...".green());
        std::process::exit(0);
    }
    let paths: ReadDir =
        std::fs::read_dir(plugin_repository_path.join("target").join("release")).unwrap();
    let mut file_path: String = String::new();
    for path in paths {
        let path_un = path.unwrap().path();
        let extension = path_un.extension();
        if let Some(extension_name) = extension {
            if vec!["dll", "so"].contains(&extension_name.to_str().unwrap()) {
                file_path = path_un.into_os_string().into_string().unwrap();
            }
        }
    }

    let mut rtop_config: RtopConfig = read_json_file(&rtop_config_path);
    rtop_config.plugins.push(RtopConfigPlugins {
        name: plugin_manifest.id.clone(),
        path: file_path,
    });
    save_json_to_file(&rtop_config, rtop_config_path);
    println!(":: {}", "Plugin linked to Rtop!".green());
    println!(":: {}", "Linking plugin to RTPM...".green());
    let rtpm_config_path: PathBuf = dirs::config_dir().unwrap().join("rtop").join("rtpm.json");
    let mut rtpm_config: RTPMConfig = read_json_file(&rtpm_config_path);
    rtpm_config.plugins.push(RTPMConfigPluginElement {
        id: plugin_manifest.id.clone(),
        name: plugin_manifest.name.clone(),
        version: plugin_manifest.version,
        repo: plugin_manifest.url,
        plugin_type,
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
    if !user_input_choice() {
        println!(":: {}", "Exiting...".blue());
        std::process::exit(0);
    }

    for plugin in plugins {
        println!(
            ":: {}",
            format!("Get the manifest for the repo: {}...", plugin).green()
        );

        let url: Url = if let Ok(url) = Url::parse(&plugin) {
            url
        } else {
            continue;
        };
        let raw_url: Url = if let Some(temp_url) = get_raw_url(&url) {
            temp_url
        } else {
            continue;
        };
        let manifest_url: Url = raw_url.join("manifest.json").unwrap();

        let manifest_resp = reqwest::blocking::get(manifest_url.clone())
            .unwrap()
            .json::<PluginManifest>();

        let plugin_manifest: PluginManifest = if let Ok(manifest) = manifest_resp {
            manifest
        } else {
            println!(":: {}", format!("The manifest of the plugin {} is wrong, please contact the author of this plugin to ask him to change it.", plugin).red().bold());
            continue;
        };
        println!(":: {}", "Manifest recovered!".green());
        install_plugin(plugin_manifest, 1);
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
    let rtpm_config: RTPMConfig = read_json_file(&rtpm_config_path);

    for plugin in plugins {
        println!(":: {}", format!("Searching plugin {}...", plugin).green());
        let repository_path_opt: Option<PathBuf> = search_plugin(
            plugin.as_str(),
            rtpm_config.clone(),
            &rtpm_config_path,
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
        let plugin_manifest: PluginManifest = read_json_file(
            &repository_path
                .join("plugins")
                .join(format!("{}.json", plugin)),
        );
        install_plugin(plugin_manifest, 0);
    }
    // println!(":: {}", "Exit...".green());
}

pub fn install(matches: &ArgMatches) {
    let must_println: bool = if contain_clap_arg("update", matches) {
        update_repositories();
        true
    } else {
        false
    };

    if contain_clap_arg("upgrade", matches) {
        update_packages();
        std::process::exit(0);
    }

    let plugins: Vec<String> = matches
        .get_many::<String>("plugins")
        .unwrap_or_else(|| {
            std::process::exit(0);
        })
        .cloned()
        .unique()
        .collect();

    if must_println {
        println!();
    }

    if contain_clap_arg("unsecure-git-url", matches) {
        install_insecure_plugins(plugins);
    } else {
        install_plugins(plugins);
    }
}
