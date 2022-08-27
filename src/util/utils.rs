use colored::*;
use std::path::PathBuf;
use url::Url;

// Based on the human_bytes library of Forkbomb9: https://gitlab.com/forkbomb9/human_bytes-rs
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

pub fn remove_plugin(path_to_remove: PathBuf, rtop_config: PathBuf) {
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
            Url::parse(&*format!(
                "https://raw.githubusercontent.com/{}/{}/main/",
                url_split[0].clone(),
                url_split[1].clone()
            ))
            .unwrap(),
        ),
        "gitlab.com" => Option::from(
            Url::parse(&*format!(
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
