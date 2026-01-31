use rules::err::RulesError;
use rules::orchestrator::Orchestrator;

fn main() -> Result<(), RulesError> {
    Orchestrator::run()
}
