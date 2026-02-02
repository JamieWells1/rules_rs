use crate::err::RulesError;
use crate::orchestrator::Orchestrator;

pub fn evaluate() -> Result<(), RulesError> {
    Orchestrator::run()
}

// TODO: Create base class which takes dir parameter, then all API-exposed methods belong to this class
