use clap::{Arg, ArgAction, Command};
use rtpm::util::structs::RTPMConfig;
use rtpm::util::utils::save_json_to_file;
use std::path::PathBuf;

fn main() {
    let rtop_data_dir: PathBuf = dirs::data_dir().unwrap_or_else(|| {
        println!("Your system is not supported, please open an issue at: https://github.com/RtopRS/RtopPluginManager/issues/new so we can add support for your system.");
        std::process::exit(9);
    }).join("rtop");

    std::fs::create_dir_all(rtop_data_dir.join("plugins")).unwrap();
    std::fs::create_dir_all(rtop_data_dir.join("repositories")).unwrap();

    let rtop_config_dir: PathBuf = dirs::config_dir().unwrap().join("rtop");
    std::fs::create_dir_all(&rtop_config_dir).unwrap();

    let config_path: PathBuf = rtop_config_dir.join("rtpm.json");
    if !config_path.exists() {
        let config: RTPMConfig = RTPMConfig {
            repositories: Vec::new(),
            plugins: Vec::new(),
        };
        save_json_to_file(&config, config_path);
    }

    let app: Command = Command::new("RtopPluginManager")
        .about("The official tool to simplify the management of plugins for Rtop.")
        .version("0.0.1")
        .help_template(
            "{bin} ({version}) - The official tool to simplify the management of plugins for Rtop.\n\n{usage-heading}\n{usage}\n\n{all-args}\n",
        )
        .subcommand_required(false)
        .arg_required_else_help(true)
        .author("Rtop Development Team")
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
            Command::new("uninstall")
                .short_flag('U')
                .long_flag("uninstall")
                .about("Uninstall a Rtop plugin.")
                .arg(
                    Arg::new("plugins")
                        .help("The plugin(s) name.")
                        .takes_value(true)
                        .multiple_values(true),
                ),
        )
        .subcommand(
            Command::new("add-repository")
                .short_flag('A')
                .long_flag("add-repository")
                .about("Add custom Rtop plugin repository.")
                .arg(
                    Arg::new("repository")
                        .help("The repository URL.")
                        .takes_value(true)
                        .multiple_values(false),
                ),
        )
        .subcommand(
            Command::new("remove-repository")
                .short_flag('R')
                .long_flag("remove-repository")
                .about("Remove custom Rtop plugin repository.")
                .arg(
                    Arg::new("repository")
                        .help("The repository URL.")
                        .takes_value(true)
                        .multiple_values(false),
                ),
        )
        .subcommand(
        Command::new("infos")
            .short_flag('I')
            .long_flag("infos")
            .about("Retrieve infos about a repository or a plugin.")
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
        Some(("install", matches)) => rtpm::commands::install::install(matches.clone()),
        Some(("infos", matches)) => rtpm::commands::infos::infos(matches.clone()),
        Some(("search", matches)) => rtpm::commands::search::search(matches.clone()),
        Some(("uninstall", matches)) => rtpm::commands::uninstall::uninstall(matches.clone()),
        Some(("add-repository", matches)) => {
            rtpm::commands::add_repository::add_repository(matches.clone())
        }
        Some(("remove-repository", matches)) => {
            rtpm::commands::remove_repository::remove_repository(matches.clone())
        }
        _ => {}
    }
}
