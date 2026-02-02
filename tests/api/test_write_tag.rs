use rules::{err::RulesError, write_tag};
use std::fs;

#[test]
fn test_write_tag_new_tag() -> Result<(), RulesError> {
    // Setup: Create a test file
    let test_file = "test_tags.tags";
    let test_path = format!("config/{}", test_file);

    let initial_content = "# Test tags file\n\n- Color: Red, Blue";
    fs::write(&test_path, initial_content).unwrap();

    // Execute: Add a new tag
    write_tag(
        test_file,
        "Size".to_string(),
        vec!["Small".to_string(), "Large".to_string()],
    )?;

    // Verify: Check the file contains the new tag
    let result = fs::read_to_string(&test_path).unwrap();
    assert!(result.contains("- Size: Small, Large"));

    // Cleanup
    fs::remove_file(&test_path).unwrap();

    Ok(())
}

#[test]
fn test_write_tag_existing_tag() -> Result<(), RulesError> {
    // Setup
    let test_file = "test_tags_existing.tags";
    let test_path = format!("config/{}", test_file);

    let initial_content = "- Color: Red, Blue";
    fs::write(&test_path, initial_content).unwrap();

    // Execute: Add values to existing tag
    write_tag(test_file, "Color".to_string(), vec!["Green".to_string()])?;

    // Verify
    let result = fs::read_to_string(&test_path).unwrap();
    assert!(result.contains("Red, Blue, Green"));

    // Cleanup
    fs::remove_file(&test_path).unwrap();

    Ok(())
}

#[test]
fn test_write_tag_invalid_tag_no_colon() {
    let test_file = "test_invalid_no_colon.tags";
    let test_path = format!("config/{}", test_file);

    // Create a file with an invalid tag (no colon)
    let initial_content = "- Color Red Blue";
    fs::write(&test_path, initial_content).unwrap();

    // Try to write to this file - should fail during validation
    let result = write_tag(test_file, "Size".to_string(), vec!["Small".to_string()]);

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("must contain a ':' separator"));
    } else {
        panic!("Expected TagParseError.");
    }

    // Cleanup
    fs::remove_file(&test_path).unwrap();
}

#[test]
fn test_write_tag_invalid_tag_no_dash() {
    let test_file = "test_invalid_no_dash.tags";
    let test_path = format!("config/{}", test_file);

    // Create a file with an invalid tag (no leading dash)
    let initial_content = "Color: Red, Blue";
    fs::write(&test_path, initial_content).unwrap();

    let result = write_tag(test_file, "Size".to_string(), vec!["Small".to_string()]);

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("Tag must begin with '-'"));
    } else {
        panic!("Expected TagParseError with '-' message.");
    }

    // Cleanup
    fs::remove_file(&test_path).unwrap();
}

#[test]
fn test_write_tag_invalid_tag_name_with_spaces() {
    let test_file = "test_invalid_name_spaces.tags";
    let test_path = format!("config/{}", test_file);

    // Create a file with tag name containing spaces
    let initial_content = "- Color Name: Red, Blue";
    fs::write(&test_path, initial_content).unwrap();

    let result = write_tag(test_file, "Size".to_string(), vec!["Small".to_string()]);

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("Tag name cannot contain spaces."));
    } else {
        panic!("Expected TagParseError about tag name spaces.");
    }

    // Cleanup
    fs::remove_file(&test_path).unwrap();
}

#[test]
fn test_write_tag_invalid_value_with_middle_spaces() {
    let test_file = "test_invalid_value_spaces.tags";
    let test_path = format!("config/{}", test_file);

    // Create a file with value containing middle spaces
    let initial_content = "- Color: Dark Blue, Red";
    fs::write(&test_path, initial_content).unwrap();

    let result = write_tag(test_file, "Size".to_string(), vec!["Small".to_string()]);

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("Tag values cannot contain spaces."));
    } else {
        panic!("Expected TagParseError about value spaces.");
    }

    // Cleanup
    fs::remove_file(&test_path).unwrap();
}

#[test]
fn test_write_tag_multiple_colons() {
    let test_file = "test_multiple_colons.tags";
    let test_path = format!("config/{}", test_file);

    // Create a file with multiple colons
    let initial_content = "- Color: Red: Blue";
    fs::write(&test_path, initial_content).unwrap();

    let result = write_tag(test_file, "Size".to_string(), vec!["Small".to_string()]);

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("only one") || msg.contains("semi-colon"));
    } else {
        panic!("Expected TagParseError about multiple colons.");
    }

    // Cleanup
    fs::remove_file(&test_path).unwrap();
}

#[test]
fn test_write_tag_file_not_found() {
    let result = write_tag(
        "nonexistent_file.tags",
        "Color".to_string(),
        vec!["Red".to_string()],
    );

    assert!(result.is_err());
    // Should be an IO error or custom error about file not found
}
