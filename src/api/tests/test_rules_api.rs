use crate::Rules;
use std::fs;
use std::path::Path;

fn setup_test_env(test_name: &str) -> String {
    let test_dir = format!("src/api/tests/test_config/{}", test_name);
    let _ = fs::create_dir_all(&test_dir);

    let tags_file = format!("{}/test.tags", test_dir);
    let tags_content = "# Test tags\n- colour: red, blue, green\n- shape: circle, square, rectangle\n- size: small, medium, large";
    fs::write(&tags_file, tags_content).unwrap();
    test_dir
}

fn cleanup_test_env(test_dir: &str) {
    if Path::new(test_dir).exists() {
        let _ = fs::remove_dir_all(test_dir);
    }
}

#[test]
fn test_rules_api_load_and_validate() {
    let test_dir = setup_test_env("test_load_validate");

    let mut rules = Rules::new(&test_dir);
    rules.load_tags().unwrap();

    assert!(rules.validate_rule("- colour = red").is_ok());
    assert!(rules.validate_rule("- shape = circle").is_ok());
    assert!(rules.validate_rule("- size = large").is_ok());

    assert!(
        rules
            .validate_rule("- colour = red & shape ! circle")
            .is_ok()
    );
    assert!(
        rules
            .validate_rule("- colour = blue | size = small")
            .is_ok()
    );

    assert!(rules.validate_rule("- colour = red, blue").is_ok());
    assert!(rules.validate_rule("- colour = red, blue, green").is_ok());

    assert!(
        rules
            .validate_rule("- (colour = red, blue) & size = large")
            .is_ok()
    );

    assert!(rules.validate_rule("- invalid = value").is_err());

    assert!(rules.validate_rule("- colour = purple").is_err());

    cleanup_test_env(&test_dir);
}

#[test]
fn test_rules_api_case_insensitive_loading() {
    let test_dir = setup_test_env("test_case_insensitive");

    let mut rules = Rules::new(&test_dir);
    rules.load_tags().unwrap();

    rules.debug_tags();

    assert!(rules.validate_rule("- colour = red").is_ok());
    assert!(rules.validate_rule("- colour = blue").is_ok());
    assert!(rules.validate_rule("- shape = circle").is_ok());

    cleanup_test_env(&test_dir);
}

#[test]
fn test_rules_api_write_methods() {
    let test_dir = setup_test_env("test_write_methods");

    let mut rules = Rules::new(&test_dir);
    rules.load_tags().unwrap();

    rules
        .write_tag("api_test", "material", vec!["wood", "metal", "plastic"])
        .unwrap();

    assert!(rules.validate_rule("- material = wood").is_ok());

    rules
        .write_rule("api_test", "- colour = red & size = large")
        .unwrap();

    cleanup_test_env(&test_dir);
}
