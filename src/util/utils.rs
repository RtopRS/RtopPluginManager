use crate::util::structs::PluginManifest;
use crate::util::structs::{RTPMConfig, RepositoryPlugin};
use colored::Colorize;
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use url::Url;

// Based on the human_bytes library of Forkbomb9: https://gitlab.com/forkbomb9/human_bytes-rs.
pub fn convert_to_readable_unity<T: Into<f64>>(size: T) -> String {
    const SUFFIX: [&str; 9] = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let size_converted: f64 = size.into();
    if size_converted <= 0.0_f64 {
        return "0 B".to_owned();
    }
    let base: f64 = size_converted.log(1024_f64);
    let mut result: String = format!("{:.1}", 1024_f64.powf(base - base.floor()))
        .trim_end_matches(".0")
        .to_owned();
    result.push_str(SUFFIX[base.floor() as usize]);
    result
}

pub fn build_cargo_project(toml_path: &PathBuf) -> bool {
    use cargo::core::{compiler::CompileMode, Workspace};
    use cargo::ops::CompileOptions;
    use cargo::util::interning::InternedString;
    use cargo::Config;

    let config: Config = Config::default().unwrap();
    let workspace: Workspace = Workspace::new(toml_path.as_ref(), &config).unwrap();
    let mut compile_options: CompileOptions =
        CompileOptions::new(&config, CompileMode::Build).unwrap();
    compile_options.build_config.requested_profile = InternedString::new("release");
    return cargo::ops::compile(&workspace, &compile_options).is_ok();
}

pub fn get_raw_url(url: &Url) -> Option<Url> {
    let url_host: &str = url.host_str().unwrap();
    let url_path: &str = url.path();
    let url_split: Vec<&str> = url_path.split('/').filter(|&s| !s.is_empty()).collect();

    match url_host {
        "github.com" => Option::from(
            Url::parse(&format!(
                "https://raw.githubusercontent.com/{}/{}/main/",
                url_split[0], url_split[1]
            ))
            .unwrap(),
        ),
        "gitlab.com" => Option::from(
            Url::parse(&format!(
                "https://gitlab.com/{}/{}/-/raw/main/",
                url_split[0], url_split[1]
            ))
            .unwrap(),
        ),
        _ => {
            println!(":: {}", "Currently, only GitHub and GitLab are supported for external plugins. You can open an issue on: https://github.com/RtopRS/RtopPluginManager/issues/new so I can add another site.".bold().red());
            None
        }
    }
}
pub fn search_plugin(
    plugin_name: &str,
    mut rtpm_config: RTPMConfig,
    rtpm_config_path: &Path,
    print_if_found: bool,
) -> Option<PathBuf> {
    let mut repository_path_opt: Option<PathBuf> = None;
    for repository in rtpm_config.repositories.clone() {
        let path: PathBuf = dirs::data_dir()
            .unwrap()
            .join("rtop")
            .join("repositories")
            .join(repository.clone());
        if !path.exists() {
            println!(
                ":: {}",
                format!(
                    "The repository {} is not or no longer present, I delete it.",
                    repository
                )
                .yellow()
            );
            let index: usize = rtpm_config
                .repositories
                .iter()
                .position(|r| r == &repository)
                .unwrap();
            rtpm_config.repositories.remove(index);
            save_json_to_file(&rtpm_config, rtpm_config_path.to_path_buf());
            continue;
        }
        let repository_plugins: RepositoryPlugin = read_json_file(&path.join("plugins.json"));
        if repository_plugins.plugins.contains(&plugin_name.to_owned()) {
            if print_if_found {
                println!(
                    ":: {}",
                    format!("Plugin found in the repository {}!", repository.bold()).green()
                );
            }
            repository_path_opt = Option::from(path);
            break;
        }
    }
    repository_path_opt
}

pub fn save_json_to_file<T>(json: &T, path: PathBuf)
where
    T: ?Sized + Serialize,
{
    std::fs::write(path.clone(), serde_json::to_string_pretty(&json).unwrap()).unwrap_or_else(
        |e| {
            println!(
                ":: {}",
                format!(
                    "An error occurred while writing to the {} file ({}).",
                    path.into_os_string().into_string().unwrap(),
                    e
                )
                .bold()
                .red()
            );
        },
    );
}

pub fn read_json_file<T>(path: &PathBuf) -> T
where
    for<'a> T: serde::Deserialize<'a>,
{
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|_| "{}".to_owned()))
        .unwrap()
}

pub fn verify_device_specification(plugin_manifest: &PluginManifest) -> bool {
    if let Some(os) = &plugin_manifest.os {
        return !(!os.is_empty() && !os.contains(&std::env::consts::OS.to_owned()));
    }

    if let Some(arch) = &plugin_manifest.arch {
        return !(!arch.is_empty() && !arch.contains(&std::env::consts::ARCH.to_owned()));
    }

    true
}

pub fn contain_clap_arg(name: &str, matches: &clap::ArgMatches) -> bool {
    matches
        .get_one::<bool>(name)
        .unwrap_or_else(|| {
            println!(
                "{}",
                "A clap error occurred, please try again.".red().bold()
            );
            std::process::exit(22);
        })
        .to_owned()
}

pub fn user_input_choice() -> bool {
    drop(std::io::stdout().flush());
    let mut user_response: String = String::new();
    std::io::stdin()
        .read_line(&mut user_response)
        .unwrap_or_else(|_| {
            println!(
                "{}",
                "An error occurred while reading the user input."
                    .red()
                    .bold()
            );
            std::process::exit(22);
        });
    vec!["y", "yes", "ok", "o"].contains(&user_response.trim().to_lowercase().as_str())
}
