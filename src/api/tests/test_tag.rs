use crate::api::write::tag::write_with_base_dir;
use crate::err::RulesError;
use std::fs;
use std::path::Path;

const TEST_CONFIG_DIR: &str = "src/api/tests/test_config";

fn setup_and_cleanup_test_file(file_name: &str) {
    // Ensure test config directory exists
    let _ = fs::create_dir_all(TEST_CONFIG_DIR);

    // Clean up any existing test file
    let path = format!("{}/{}", TEST_CONFIG_DIR, file_name);
    if Path::new(&path).exists() {
        let _ = fs::remove_file(&path);
    }
}

fn cleanup_test_file(file_name: &str) {
    let path = format!("{}/{}", TEST_CONFIG_DIR, file_name);
    if Path::new(&path).exists() {
        let _ = fs::remove_file(&path);
    }
}

#[test]
fn test_write_tag_creates_file() {
    let file_name = "test_create.tags";
    setup_and_cleanup_test_file(file_name);

    let result = write_with_base_dir(
        file_name,
        "colour".to_string(),
        vec!["red".to_string(), "blue".to_string()],
        TEST_CONFIG_DIR,
    );

    assert!(result.is_ok());
    assert!(Path::new(&format!("{}/{}", TEST_CONFIG_DIR, file_name)).exists());

    cleanup_test_file(file_name);
}

#[test]
fn test_write_tag_normalizes_filename() {
    let file_name_without_ext = "test_normalize";
    let file_name_with_ext = "test_normalize.tags";
    setup_and_cleanup_test_file(file_name_with_ext);

    let result = write_with_base_dir(
        file_name_without_ext,
        "colour".to_string(),
        vec!["red".to_string()],
        TEST_CONFIG_DIR,
    );

    assert!(result.is_ok());
    assert!(Path::new(&format!("{}/{}", TEST_CONFIG_DIR, file_name_with_ext)).exists());

    cleanup_test_file(file_name_with_ext);
}

#[test]
fn test_write_tag_validates_empty_name() {
    let file_name = "test_empty_name.tags";
    setup_and_cleanup_test_file(file_name);

    let result = write_with_base_dir(
        file_name,
        "".to_string(),
        vec!["red".to_string()],
        TEST_CONFIG_DIR,
    );

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("cannot be empty"));
    } else {
        panic!("Expected TagParseError about empty name");
    }

    cleanup_test_file(file_name);
}

#[test]
fn test_write_tag_validates_empty_values() {
    let file_name = "test_empty_values.tags";
    setup_and_cleanup_test_file(file_name);

    let result = write_with_base_dir(
        file_name,
        "colour".to_string(),
        vec![],
        TEST_CONFIG_DIR,
    );

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("at least one value"));
    } else {
        panic!("Expected TagParseError about empty values");
    }

    cleanup_test_file(file_name);
}

#[test]
fn test_write_tag_validates_spaces_in_name() {
    let file_name = "test_spaces_name.tags";
    setup_and_cleanup_test_file(file_name);

    let result = write_with_base_dir(
        file_name,
        "colour name".to_string(),
        vec!["red".to_string()],
        TEST_CONFIG_DIR,
    );

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("cannot contain spaces"));
    } else {
        panic!("Expected TagParseError about spaces in name");
    }

    cleanup_test_file(file_name);
}

#[test]
fn test_write_tag_validates_spaces_in_values() {
    let file_name = "test_spaces_value.tags";
    setup_and_cleanup_test_file(file_name);

    let result = write_with_base_dir(
        file_name,
        "colour".to_string(),
        vec!["dark red".to_string()],
        TEST_CONFIG_DIR,
    );

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("cannot contain spaces"));
        assert!(msg.contains("dark red"));
    } else {
        panic!("Expected TagParseError about spaces in value");
    }

    cleanup_test_file(file_name);
}

#[test]
fn test_write_tag_appends_values_to_existing_tag() {
    let file_name = "test_append.tags";
    setup_and_cleanup_test_file(file_name);

    // Write initial tag
    write_with_base_dir(
        file_name,
        "colour".to_string(),
        vec!["red".to_string(), "blue".to_string()],
        TEST_CONFIG_DIR,
    )
    .unwrap();

    // Append more values to same tag
    write_with_base_dir(
        file_name,
        "colour".to_string(),
        vec!["green".to_string(), "yellow".to_string()],
        TEST_CONFIG_DIR,
    )
    .unwrap();

    // Read file and verify
    let content = fs::read_to_string(&format!("{}/{}", TEST_CONFIG_DIR, file_name)).unwrap();
    assert!(content.contains("red"));
    assert!(content.contains("blue"));
    assert!(content.contains("green"));
    assert!(content.contains("yellow"));

    // Should only have one line for colour
    let colour_lines: Vec<&str> = content.lines().filter(|l| l.contains("colour")).collect();
    assert_eq!(colour_lines.len(), 1);

    cleanup_test_file(file_name);
}

#[test]
fn test_write_tag_adds_multiple_tags() {
    let file_name = "test_multiple.tags";
    setup_and_cleanup_test_file(file_name);

    // Write first tag
    write_with_base_dir(
        file_name,
        "colour".to_string(),
        vec!["red".to_string(), "blue".to_string()],
        TEST_CONFIG_DIR,
    )
    .unwrap();

    // Write second tag
    write_with_base_dir(
        file_name,
        "size".to_string(),
        vec!["small".to_string(), "large".to_string()],
        TEST_CONFIG_DIR,
    )
    .unwrap();

    // Read file and verify both tags exist
    let content = fs::read_to_string(&format!("{}/{}", TEST_CONFIG_DIR, file_name)).unwrap();
    assert!(content.contains("colour"));
    assert!(content.contains("size"));
    assert!(content.contains("red"));
    assert!(content.contains("small"));

    cleanup_test_file(file_name);
}

#[test]
fn test_write_tag_creates_config_dir() {
    let test_dir = "src/api/tests/test_config_creation";

    // Remove test directory if it exists
    if Path::new(test_dir).exists() {
        let _ = fs::remove_dir_all(test_dir);
    }

    let file_name = "test_dir.tags";

    let result = write_with_base_dir(
        file_name,
        "colour".to_string(),
        vec!["red".to_string()],
        test_dir,
    );
    assert!(result.is_ok());
    assert!(Path::new(test_dir).exists());

    // Clean up test directory
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_write_tag_formats_correctly() {
    let file_name = "test_format.tags";
    setup_and_cleanup_test_file(file_name);

    write_with_base_dir(
        file_name,
        "colour".to_string(),
        vec!["red".to_string(), "blue".to_string(), "green".to_string()],
        TEST_CONFIG_DIR,
    )
    .unwrap();

    let content = fs::read_to_string(&format!("{}/{}", TEST_CONFIG_DIR, file_name)).unwrap();

    // Should be formatted as "- colour: red, blue, green"
    assert!(content.contains("- colour: red, blue, green"));

    cleanup_test_file(file_name);
}
