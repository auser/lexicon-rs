use crate::app::GateAction;

pub fn run(action: GateAction) -> miette::Result<()> {
    match action {
        GateAction::Init => crate::commands::score::run_gate_init(),
    }
}
