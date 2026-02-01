// Business logic orchestration (state machine)
use crate::types;
use std::collections::HashMap;

use crate::{err::RulesError, parser::rules::Parser, parser::tags};

#[derive(Default)]
pub struct Orchestrator {
    m_tags: HashMap<types::TagName, types::TagValues>,
    m_subrules: HashMap<types::SubRuleNumber, types::SubRule>,
}

impl Orchestrator {
    fn map_tags(&mut self) -> Result<(), RulesError> {
        let tags: Vec<types::Tag> = tags::parse_tags()?;
        for tag in tags {
            self.m_tags.insert(tag.name, tag.values);
        }

        Ok(())
    }

    fn map_subrules(&mut self) -> Result<(), RulesError> {
        let dnf_subrules: Vec<types::SubRule> = Parser::parse_rules(self.m_tags.clone())?;
        for (i, subrule) in dnf_subrules.into_iter().enumerate() {
            self.m_subrules.insert(i as i32, subrule);
        }

        Ok(())
    }

    pub fn run() -> Result<(), RulesError> {
        let mut orch: Orchestrator = Orchestrator::default();

        // Parsing and storage
        orch.map_tags()?;

        // Parsing, convertion to DNF and storage
        orch.map_subrules()?;

        Ok(())
    }
}
