use rtpm::util::utils;
use url::Url;

#[test]
fn test_convert_to_readable_unity() {
    assert_eq!("2.4kB", utils::convert_to_readable_unity(2500));
    assert_eq!("5.7MB", utils::convert_to_readable_unity(6000000));
    assert_eq!("57.2MB", utils::convert_to_readable_unity(60000000));
}

#[test]
fn test_get_raw_url() {
    assert_eq!(
        "https://raw.githubusercontent.com/RtopRS/PluginTemplate/main/",
        utils::get_raw_url(&Url::parse("https://github.com/RtopRS/PluginTemplate/").unwrap())
            .unwrap()
            .as_str()
    );
    assert_eq!(
        "https://gitlab.com/rtoprs/RtopPluginManager/-/raw/main/",
        utils::get_raw_url(&Url::parse("https://gitlab.com/rtoprs/RtopPluginManager").unwrap())
            .unwrap()
            .as_str()
    );
    assert_eq!(
        None,
        utils::get_raw_url(&Url::parse("https://sourceforge.net/projects/android-x86/").unwrap())
    );
}
