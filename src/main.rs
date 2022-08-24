use clap::{Arg, ArgAction, Command};

fn main() {
    let rtop_util_config_path: std::path::PathBuf = dirs::data_dir().unwrap_or_else(|| {
        println!("Your system is not supported, please open an issue at: https://github.com/RtopRS/RtopUtil/issues/new so we can add support for your system.");
        std::process::exit(9);
    }).join("rtop").join("plugins");
    std::fs::create_dir_all(rtop_util_config_path).unwrap();

    let app: Command = Command::new("AFetch")
        .about("The official tool to simplify the management of plugins for RTop.")
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
                    Arg::new("unsecure-git-url")
                        .help("This flag allows to download a plugin from a git repo.")
                        .required_unless_present("plugins")
                        .short('z')
                        .action(ArgAction::SetTrue)
                        .takes_value(false)
                )
                .arg(
                    Arg::new("plugins")
                        .help("The plugin name or git repository URL.")
                        .takes_value(true)
                        .multiple_values(true),
                ),
        );

    match app.get_matches().subcommand() {
        Some(("install", sub_matches)) => {
            rtop_util::commands::install::install(sub_matches.clone())
        }
        _ => {}
    }
}
