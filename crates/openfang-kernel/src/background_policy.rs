use openfang_types::agent::{AgentEntry, AgentId, AgentState, ScheduleMode};

/// Collect restored agents whose background loops should resume at startup.
pub(crate) fn collect_background_agents(
    entries: &[AgentEntry],
) -> Vec<(AgentId, String, ScheduleMode)> {
    let mut bg_agents = Vec::new();

    for entry in entries {
        if entry.state != AgentState::Running {
            continue;
        }
        if matches!(entry.manifest.schedule, ScheduleMode::Reactive) {
            continue;
        }
        bg_agents.push((
            entry.id,
            entry.name.clone(),
            entry.manifest.schedule.clone(),
        ));
    }

    bg_agents
}
