use crate::util::structs::{RTPMConfig, RTPMConfigPluginElement, RtopConfig};
use crate::util::utils::save_json_to_file;
use clap::ArgMatches;
use colored::Colorize;
use itertools::Itertools;
use std::path::PathBuf;

pub fn uninstall(matches: ArgMatches) {
    let plugins: Vec<String> = matches
        .get_many::<String>("plugins")
        .unwrap_or_else(|| {
            std::process::exit(0);
        })
        .map(|s| s.to_owned())
        .unique()
        .collect();

    if plugins.len() == 1 {
        println!(":: {}", "Start uninstalling plugin...\n".green().bold());
    } else {
        println!(":: {}", "Start uninstalling plugins...\n".green().bold());
    }

    let config_dir: PathBuf = dirs::config_dir().unwrap().join("rtop");
    let rtpm_config_path: PathBuf = config_dir.join("rtpm.json");
    let mut rtpm_config: RTPMConfig = serde_json::from_str(
        &std::fs::read_to_string(rtpm_config_path.clone()).unwrap_or_else(|_| "{}".to_string()),
    )
    .unwrap();
    let rtop_config_path: PathBuf = config_dir.join("config");
    let mut rtop_config: RtopConfig = serde_json::from_str(
        &std::fs::read_to_string(rtop_config_path.clone()).unwrap_or_else(|_| "{}".to_string()),
    )
    .unwrap();
    let plugins_path: PathBuf = dirs::data_dir().unwrap().join("rtop").join("plugins");

    for plugin in plugins {
        let mut config_plugin_element_opt: Option<RTPMConfigPluginElement> = None;
        for installed_plugin in rtpm_config.plugins.clone() {
            if plugin == installed_plugin.id {
                config_plugin_element_opt = Option::from(installed_plugin);
                break;
            }
        }
        let config_plugin_element: RTPMConfigPluginElement =
            if let Some(config_plugin_element) = config_plugin_element_opt {
                config_plugin_element
            } else {
                println!(":: {}", format!("Plugin {} not found.\n", plugin).red());
                continue;
            };
        println!(
            ":: {}",
            format!("Uninstalling the plugin {}...", config_plugin_element.name).green()
        );

        let plugin_path: PathBuf = plugins_path.join(config_plugin_element.id.clone());

        println!(":: {}", "Removing plugin folder...".green());
        std::fs::remove_dir_all(plugin_path.clone()).unwrap();
        println!(":: {}", "Plugin folder removed!".green());

        println!(":: {}", "Removing plugin from RTPM config...".green());
        let rtpm_plugin_index: usize = rtpm_config
            .plugins
            .iter()
            .position(|r| r.id == config_plugin_element.id)
            .unwrap();
        rtpm_config.plugins.remove(rtpm_plugin_index);
        println!(":: {}", "Plugin removed from RTPM config!".green());

        println!(":: {}", "Removing plugin from Rtop config...".green());
        let rtop_plugin_index: usize = rtop_config
            .plugins
            .iter()
            .position(|r| {
                r.path
                    .starts_with(&plugin_path.clone().into_os_string().into_string().unwrap())
            })
            .unwrap();
        rtop_config.plugins.remove(rtop_plugin_index);
        println!(":: {}", "Plugin removed from Rtop config!".green());

        println!(
            ":: {}",
            format!("Plugin {} uninstalled!\n", config_plugin_element.name).green()
        );
    }

    save_json_to_file(&rtpm_config, rtpm_config_path);
    save_json_to_file(&rtop_config, rtop_config_path);

    println!(":: {}", "End of the uninstallation!".green().bold());
}
