use crate::git::clone::clone;
use crate::util::structs::{PluginManifest, RtopConfig, RtopConfigPlugins};
use crate::util::utils::{build_cargo_project, remove_plugin};
use clap::ArgMatches;
use colored::*;
use std::io::Write;
use url::Url;

#[derive(Clone, Debug)]
struct RepoInfos {
    raw_url: Url,
}

fn install_plugin_by_insecure(plugins: Vec<String>) {
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

        let url: Url = if let Ok(url) = Url::parse(&*plugin) { url } else { continue };
        let url_host: &str = url.host_str().unwrap();
        let url_path: &str = url.path();
        let url_split: Vec<String> = url_path
            .split('/')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<&str>>()
            .into_iter()
            .map(|s| s.to_owned())
            .collect();

        let repo_infos: RepoInfos = match url_host {
            "github.com" => RepoInfos {
                raw_url: Url::parse(&*format!(
                    "https://raw.githubusercontent.com/{}/{}/main/",
                    url_split[0].clone(),
                    url_split[1].clone()
                ))
                .unwrap(),
            },
            "gitlab.com" => RepoInfos {
                raw_url: Url::parse(&*format!(
                    "https://gitlab.com/{}/{}/-/raw/main/",
                    url_split[0].clone(),
                    url_split[1].clone()
                ))
                .unwrap(),
            },
            _ => {
                println!(":: {}", "Currently, only GitHub and GitLab are supported. You can open an issue on: https://github.com/RtopRS/RtopUtil/issues/new so I can add another site.".bold().red());
                continue;
            }
        };
        let manifest_url: Url = repo_infos.raw_url.join("manifest.json").unwrap();

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

pub fn install(sub_matches: ArgMatches) {
    let plugins: Vec<String> = sub_matches
        .get_many::<String>("plugins")
        .unwrap_or_else(|| {
            println!("{}", "An unknown error has occurred.".red().bold());
            std::process::exit(9);
        })
        .map(|s| s.to_owned())
        .collect();

    if sub_matches
        .get_one::<bool>("unsecure-git-url")
        .expect("Defaulted by clap")
        .to_owned()
    {
        install_plugin_by_insecure(plugins);
    }
}
