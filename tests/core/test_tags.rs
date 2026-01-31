use rules::err::RulesError;
use rules::parser::tags;

#[test]
fn test_validate_tag_valid() {
    let valid_tag = "- Color: Red, Blue, Green";
    assert!(tags::validate_tag(valid_tag).is_ok());
}

#[test]
fn test_validate_tag_skips_comments() {
    let comment = "# This is a comment";
    assert!(tags::validate_tag(comment).is_ok());
}

#[test]
fn test_validate_tag_skips_empty_lines() {
    let empty = "";
    assert!(tags::validate_tag(empty).is_ok());
}

#[test]
fn test_validate_tag_no_colon() {
    let invalid_tag = "- Color Red Blue";
    let result = tags::validate_tag(invalid_tag);

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("must contain a ':' separator"));
    } else {
        panic!("Expected TagParseError about missing colon");
    }
}

#[test]
fn test_validate_tag_no_dash() {
    let invalid_tag = "Color: Red, Blue";
    let result = tags::validate_tag(invalid_tag);

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("Tag must begin with '-'"));
    } else {
        panic!("Expected TagParseError about missing dash");
    }
}

#[test]
fn test_validate_tag_name_with_spaces() {
    let invalid_tag = "- Color Name: Red, Blue";
    let result = tags::validate_tag(invalid_tag);

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("Tag name cannot contain spaces"));
    } else {
        panic!("Expected TagParseError about tag name spaces");
    }
}

#[test]
fn test_validate_tag_value_with_middle_spaces() {
    let invalid_tag = "- Color: Dark Blue, Red";
    let result = tags::validate_tag(invalid_tag);

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("Tag values cannot contain spaces"));
    } else {
        panic!("Expected TagParseError about value spaces");
    }
}

#[test]
fn test_validate_tag_multiple_colons() {
    let invalid_tag = "- Color: Red: Blue";
    let result = tags::validate_tag(invalid_tag);

    assert!(result.is_err());
    if let Err(RulesError::TagParseError(msg)) = result {
        assert!(msg.contains("only one") || msg.contains("semi-colon"));
    } else {
        panic!("Expected TagParseError about multiple colons");
    }
}

#[test]
fn test_validate_tag_with_leading_whitespace() {
    let tag_with_whitespace = "  - Color: Red, Blue";
    assert!(tags::validate_tag(tag_with_whitespace).is_ok());
}

#[test]
fn test_get_name_and_values_from_tag() {
    let tag = "- Color: Red, Blue, Green";
    let result = tags::get_name_and_values_from_tag(tag);

    assert!(result.is_ok());
    if let Ok((name, values)) = result {
        assert_eq!(name, "Color");
        assert_eq!(values.len(), 3);
        assert!(values.contains(&"Red".to_string()));
        assert!(values.contains(&"Blue".to_string()));
        assert!(values.contains(&"Green".to_string()));
    }
}

#[test]
fn test_get_name_and_values_trims_whitespace() {
    let tag = "  - Color  :  Red ,  Blue  ";
    let result = tags::get_name_and_values_from_tag(tag);

    assert!(result.is_ok());
    if let Ok((name, values)) = result {
        assert_eq!(name, "Color");
        assert_eq!(values, vec!["Red".to_string(), "Blue".to_string()]);
    }
}
