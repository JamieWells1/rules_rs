use crate::err::RulesError;
use crate::parser::rules::RuleParser;
use crate::types::{TagName, TagValues};
use std::collections::HashMap;

/// Main API for the rules engine.
///
/// Provides methods for managing tags, rules, objects, and evaluating rules
/// against objects.
///
/// # Examples
/// ```ignore
/// use rules::Rules;
///
/// // Create a new Rules instance with a config directory
/// let mut rules = Rules::new("config");
///
/// // Add tags
/// rules.write_tag("colours.tags", "colour", vec!["red", "blue"])?;
///
/// // Add rules that reference those tags
/// rules.write_rule("my_rules.rules", "- colour = red")?;
///
/// // Validate a rule
/// rules.validate_rule("- colour = blue")?;
/// ```
pub struct Rules {
    /// Base directory for config files
    config_dir: String,
    /// Cached tags loaded from config files
    tags: HashMap<TagName, TagValues>,
}

impl Rules {
    /// Creates a new Rules instance with the specified config directory.
    ///
    /// # Arguments
    /// * `config_dir` - Path to the directory containing .tags, .rules, and .yaml files
    ///
    /// # Examples
    /// ```ignore
    /// let rules = Rules::new("config");
    /// ```
    pub fn new(config_dir: impl Into<String>) -> Self {
        Self {
            config_dir: config_dir.into(),
            tags: HashMap::new(),
        }
    }

    /// Loads all tags from .tags files in the config directory.
    ///
    /// This should be called after creating a new Rules instance to populate
    /// the tag definitions needed for rule validation.
    ///
    /// # Returns
    /// * `Ok(())` if tags were loaded successfully
    /// * `Err(RulesError)` if loading fails
    ///
    /// # Examples
    /// ```ignore
    /// let mut rules = Rules::new("config");
    /// rules.load_tags()?;
    /// ```
    pub fn load_tags(&mut self) -> Result<(), RulesError> {
        // TODO: Implement loading all .tags files from config_dir
        // For now, you'd need to add a function to parse .tags files
        // This would read all .tags files and populate self.tags
        todo!("Implement tag loading from directory")
    }

    /// Writes a tag to a .tags file.
    ///
    /// # Arguments
    /// * `file_name` - Name of the file (with or without .tags extension)
    /// * `tag_name` - Name of the tag (without the leading '-')
    /// * `tag_values` - Vector of values for the tag
    ///
    /// # Examples
    /// ```ignore
    /// rules.write_tag("my_tags", "colour", vec!["red", "blue"])?;
    /// ```
    pub fn write_tag(
        &mut self,
        file_name: &str,
        tag_name: impl Into<String>,
        tag_values: Vec<String>,
    ) -> Result<(), RulesError> {
        let tag_name = tag_name.into();

        // Write to file
        crate::api::write::tag::write_with_base_dir(
            file_name,
            tag_name.clone(),
            tag_values.clone(),
            &self.config_dir,
        )?;

        // Update cached tags
        self.tags.insert(tag_name, tag_values);

        Ok(())
    }

    /// Writes a rule to a .rules file.
    ///
    /// The rule is validated against the current tag definitions before writing.
    ///
    /// # Arguments
    /// * `file_name` - Name of the file (with or without .rules extension)
    /// * `rule` - The rule string to write (should start with '-')
    ///
    /// # Examples
    /// ```ignore
    /// rules.write_rule("my_rules", "- colour = red & size = large")?;
    /// ```
    pub fn write_rule(&self, file_name: &str, rule: &str) -> Result<(), RulesError> {
        crate::api::write::rule::write_with_base_dir(
            file_name,
            rule,
            self.tags.clone(),
            &self.config_dir,
        )
    }

    /// Writes an object definition to a .yaml file.
    ///
    /// # Arguments
    /// * `file_name` - Name of the file
    /// * `obj_type` - Type/category of the object
    /// * `obj` - HashMap representing the object's properties
    ///
    /// # Examples
    /// ```ignore
    /// let mut obj = HashMap::new();
    /// obj.insert("colour".to_string(), vec!["red".to_string()]);
    /// rules.write_object("objects.yaml", "shapes", obj)?;
    /// ```
    pub fn write_object(
        &self,
        file_name: &str,
        obj_type: impl Into<String>,
        obj: HashMap<String, Vec<String>>,
    ) -> Result<(), RulesError> {
        crate::api::write::object::write_with_base_dir(
            file_name,
            obj_type.into(),
            obj,
            &self.config_dir,
        )
    }

    /// Validates a rule string against the current tag definitions.
    ///
    /// This checks syntax and ensures all referenced tags and values exist.
    ///
    /// # Arguments
    /// * `rule` - The rule string to validate (should start with '-')
    ///
    /// # Returns
    /// * `Ok(())` if the rule is valid
    /// * `Err(RulesError)` with details if validation fails
    ///
    /// # Examples
    /// ```ignore
    /// rules.validate_rule("- colour = red & size = large")?;
    /// ```
    pub fn validate_rule(&self, rule: &str) -> Result<(), RulesError> {
        let parser = RuleParser::new(self.tags.clone());
        parser.validate_rule(rule)
    }

    /// Evaluates rules against an object.
    ///
    /// # Arguments
    /// * `rules_file` - Path to the .rules file
    /// * `objects_file` - Path to the .yaml file containing objects
    ///
    /// # Examples
    /// ```ignore
    /// rules.evaluate("my_rules.rules", "objects.yaml")?;
    /// ```
    pub fn evaluate(&self, rules_file: &str, objects_file: &str) -> Result<(), RulesError> {
        // Construct full paths
        let rules_path = format!("{}/{}", self.config_dir, rules_file);
        let objects_path = format!("{}/{}", self.config_dir, objects_file);

        crate::api::entry::evaluate(&rules_path, &objects_path)
    }

    /// Gets a reference to the current tag definitions.
    ///
    /// # Examples
    /// ```ignore
    /// let tags = rules.tags();
    /// println!("Available tags: {:?}", tags.keys());
    /// ```
    pub fn tags(&self) -> &HashMap<TagName, TagValues> {
        &self.tags
    }

    /// Gets the config directory path.
    pub fn config_dir(&self) -> &str {
        &self.config_dir
    }
}
