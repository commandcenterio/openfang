use openfang_types::agent::{AgentManifest, ModelConfig};
use openfang_types::config::DefaultModelConfig;
use tracing::warn;

fn json_changed<T: serde::Serialize>(left: &T, right: &T) -> bool {
    serde_json::to_string(left).ok() != serde_json::to_string(right).ok()
}

const BUNDLED_ASSISTANT_TEMPLATE: &str = include_str!("../../../agents/assistant/agent.toml");

fn fallback_default_assistant_manifest(default_model: &DefaultModelConfig) -> AgentManifest {
    AgentManifest {
        name: "assistant".to_string(),
        description: "General-purpose assistant".to_string(),
        model: ModelConfig {
            provider: default_model.provider.clone(),
            model: default_model.model.clone(),
            system_prompt: "You are a helpful AI assistant.".to_string(),
            api_key_env: if default_model.api_key_env.is_empty() {
                None
            } else {
                Some(default_model.api_key_env.clone())
            },
            base_url: default_model.base_url.clone(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Build the fresh-install default assistant manifest from the effective kernel default model.
pub(crate) fn build_default_assistant_manifest(default_model: &DefaultModelConfig) -> AgentManifest {
    let mut manifest = match toml::from_str::<AgentManifest>(BUNDLED_ASSISTANT_TEMPLATE) {
        Ok(manifest) => manifest,
        Err(error) => {
            warn!(%error, "Failed to parse bundled assistant template; falling back to minimal default assistant manifest");
            return fallback_default_assistant_manifest(default_model);
        }
    };

    manifest.model.provider = default_model.provider.clone();
    manifest.model.model = default_model.model.clone();
    manifest.model.api_key_env = if default_model.api_key_env.is_empty() {
        None
    } else {
        Some(default_model.api_key_env.clone())
    };
    manifest.model.base_url = default_model.base_url.clone();
    manifest
}

/// Refresh the bundled default assistant behavior for restored installs while
/// preserving user-specific runtime settings.
pub(crate) fn refresh_default_assistant_manifest(
    manifest: &AgentManifest,
    default_model: &DefaultModelConfig,
) -> Option<AgentManifest> {
    if manifest.name != "assistant" || !manifest.tags.iter().any(|tag| tag == "default") {
        return None;
    }

    let template = build_default_assistant_manifest(default_model);
    let mut refreshed = manifest.clone();

    refreshed.version = template.version;
    refreshed.description = template.description;
    refreshed.author = template.author;
    refreshed.module = template.module;
    refreshed.schedule = template.schedule;
    refreshed.fallback_models = template.fallback_models;
    refreshed.capabilities = template.capabilities;
    refreshed.profile = template.profile;
    refreshed.tags = template.tags;
    refreshed.generate_identity_files = template.generate_identity_files;
    refreshed.model.system_prompt = template.model.system_prompt;

    if refreshed.version == manifest.version
        && refreshed.description == manifest.description
        && refreshed.author == manifest.author
        && refreshed.module == manifest.module
        && !json_changed(&refreshed.schedule, &manifest.schedule)
        && !json_changed(&refreshed.fallback_models, &manifest.fallback_models)
        && !json_changed(&refreshed.capabilities, &manifest.capabilities)
        && !json_changed(&refreshed.profile, &manifest.profile)
        && refreshed.tags == manifest.tags
        && refreshed.generate_identity_files == manifest.generate_identity_files
        && refreshed.model.system_prompt == manifest.model.system_prompt
    {
        return None;
    }

    Some(refreshed)
}
