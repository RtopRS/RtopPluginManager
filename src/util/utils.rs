use std::path::PathBuf;

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
    use colored::*;
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
