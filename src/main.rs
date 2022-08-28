use clap::{Arg, ArgAction, Command};
use colored::*;
use rtpm::util::structs::RTPMConfig;

fn main() {
    let rtop_util_config_path: std::path::PathBuf = dirs::data_dir().unwrap_or_else(|| {
        println!("Your system is not supported, please open an issue at: https://github.com/RtopRS/RtopUtil/issues/new so we can add support for your system.");
        std::process::exit(9);
    }).join("rtop").join("plugins");
    std::fs::create_dir_all(rtop_util_config_path).unwrap();
    std::fs::create_dir_all(dirs::data_dir().unwrap().join("rtop").join("repositories")).unwrap();
    let config_path: std::path::PathBuf =
        dirs::config_dir().unwrap().join("rtop").join("rtpm.json");
    if !config_path.exists() {
        let config: RTPMConfig = RTPMConfig {
            repositories: Vec::new(),
            plugins: Vec::new(),
        };
        let config_prettified: String = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(config_path, config_prettified).unwrap_or_else(|e| {
            println!(
                ":: {}",
                format!("An error occurred while writing to the Rtop file ({}).", e)
                    .bold()
                    .red()
            );
        });
    }

    let app: Command = Command::new("RtopPluginManager")
        .about("The official tool to simplify the management of plugins for Rtop.")
        .version("0.0.1")
        .help_template(
            "{bin} ({version}) - The official tool to simplify the management of plugins for RTop.\n\n{usage-heading}\n{usage}\n\n{all-args}\n",
        )
        .subcommand_required(false)
        .arg_required_else_help(false)
        .subcommand(
            Command::new("install")
                .short_flag('S')
                .long_flag("install")
                .about("Install a Rtop plugin.")
                .arg(
                    Arg::new("upgrade")
                        .help("This flag allows to update all plugins.")
                        .conflicts_with_all(["plugins", "unsecure-git-url"].as_ref())
                        .short('d')
                        .action(ArgAction::SetTrue)
                        .takes_value(false)
                )
                .arg(
                    Arg::new("update")
                        .help("This flag allows to update plugins repositories.")
                        .short('u')
                        .action(ArgAction::SetTrue)
                        .takes_value(false)
                )
                .arg(
                    Arg::new("unsecure-git-url")
                        .help("This flag allows to download a plugin from a git repo.")
                        .conflicts_with("upgrade")
                        .short('z')
                        .action(ArgAction::SetTrue)
                        .takes_value(false)
                )
                .arg(
                    Arg::new("plugins")
                        .help("The plugin name or git repository URL.")
                        .conflicts_with("upgrade")
                        .takes_value(true)
                        .multiple_values(true),
                ),
        )
        .subcommand(
            Command::new("search")
                .short_flag('Q')
                .long_flag("search")
                .about("Search package in all repositories.")
                .arg(
                    Arg::new("plugins")
                        .help("The plugin(s) name.")
                        .takes_value(true)
                        .multiple_values(true),
                ),
        )
        .subcommand(
        Command::new("infos")
            .short_flag('I')
            .long_flag("infos")
            .about("Retrieve info about a repository or a plugin.")
            .arg(
                Arg::new("repository")
                    .help("This flag allows to show informations about a repository.")
                    .conflicts_with("plugin")
                    .short('r')
                    .action(ArgAction::SetTrue)
                    .takes_value(false)
            )
            .arg(
                Arg::new("plugin")
                    .help("This flag allows to show informations about a plugin.")
                    .conflicts_with("repository")
                    .short('p')
                    .action(ArgAction::SetTrue)
                    .takes_value(false)
            )
            .arg(
                Arg::new("list")
                    .help("This flag allows to list installed plugins or repositories.")
                    .short('a')
                    .action(ArgAction::SetTrue)
                    .takes_value(false)
            )
            .arg(
                Arg::new("elements")
                    .help("The plugin or repository name.")
                    .takes_value(true)
                    .multiple_values(true),
            ),
    );

    match app.get_matches().subcommand() {
        Some(("install", sub_matches)) => rtpm::commands::install::install(sub_matches.clone()),
        Some(("infos", sub_matches)) => rtpm::commands::infos::infos(sub_matches.clone()),
        Some(("search", sub_matches)) => rtpm::commands::search::search(sub_matches.clone()),
        _ => {}
    }
}
