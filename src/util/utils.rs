use crate::util::structs::{RTPMConfig, RepositoryPlugin};
use colored::*;
use serde::Serialize;
use std::path::PathBuf;
use url::Url;

// Based on the human_bytes library of Forkbomb9: https://gitlab.com/forkbomb9/human_bytes-rs.
pub fn convert_to_readable_unity<T: Into<f64>>(size: T) -> String {
    const SUFFIX: [&str; 9] = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let size_converted: f64 = size.into();
    if size_converted <= 0.0_f64 {
        return "0 B".to_owned();
    }
    let base: f64 = size_converted.log10() / 1024_f64.log10();
    let mut result: String = format!("{:.1}", 1024_f64.powf(base - base.floor()))
        .trim_end_matches(".0")
        .to_owned();
    result.push_str(SUFFIX[base.floor() as usize]);
    result
}

pub fn build_cargo_project(toml_path: PathBuf) {
    use cargo::core::{compiler::CompileMode, Workspace};
    use cargo::ops::CompileOptions;
    use cargo::util::interning::InternedString;
    use cargo::Config;

    let config: Config = Config::default().unwrap();
    let workspace: Workspace = Workspace::new(toml_path.as_ref(), &config).unwrap();
    let mut compile_options: CompileOptions =
        CompileOptions::new(&config, CompileMode::Build).unwrap();
    compile_options.build_config.requested_profile = InternedString::new("release");
    cargo::ops::compile(&workspace, &compile_options).unwrap();
}

pub fn get_raw_url(url: Url) -> Option<Url> {
    let url_host: &str = url.host_str().unwrap();
    let url_path: &str = url.path();
    let url_split: Vec<String> = url_path
        .split('/')
        .filter(|&s| !s.is_empty())
        .collect::<Vec<&str>>()
        .into_iter()
        .map(|s| s.to_owned())
        .collect();

    match url_host {
        "github.com" => Option::from(
            Url::parse(&format!(
                "https://raw.githubusercontent.com/{}/{}/main/",
                url_split[0].clone(),
                url_split[1].clone()
            ))
            .unwrap(),
        ),
        "gitlab.com" => Option::from(
            Url::parse(&format!(
                "https://gitlab.com/{}/{}/-/raw/main/",
                url_split[0].clone(),
                url_split[1].clone()
            ))
            .unwrap(),
        ),
        _ => {
            println!(":: {}", "Currently, only GitHub and GitLab are supported for external plugins. You can open an issue on: https://github.com/RtopRS/RtopUtil/issues/new so I can add another site.".bold().red());
            None
        }
    }
}
pub fn search_plugin(
    plugin_name: String,
    mut rtpm_config: RTPMConfig,
    rtpm_config_path: PathBuf,
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
            save_json_to_file(&rtpm_config, rtpm_config_path.clone());
            continue;
        }
        let repository_plugins: RepositoryPlugin = serde_json::from_str(
            &std::fs::read_to_string(path.join("plugins.json"))
                .unwrap_or_else(|_| "{}".to_string()),
        )
        .unwrap();
        if repository_plugins.plugins.contains(&plugin_name) {
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
