use openfang_types::agent::{AgentManifest, ModelConfig};
use openfang_types::config::DefaultModelConfig;

/// Build the fresh-install default assistant manifest from the effective kernel default model.
pub(crate) fn build_default_assistant_manifest(default_model: &DefaultModelConfig) -> AgentManifest {
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
