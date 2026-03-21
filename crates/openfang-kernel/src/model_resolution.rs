use openfang_runtime::agent_loop::strip_provider_prefix;
use openfang_runtime::model_catalog::ModelCatalog;
use openfang_types::agent::AgentManifest;
use openfang_types::config::DefaultModelConfig;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedModelTarget {
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) api_key_env: Option<String>,
    pub(crate) base_url: Option<String>,
}

/// Apply kernel default-model policy to a manifest being spawned.
pub(crate) fn apply_spawn_model_defaults(
    manifest: &mut AgentManifest,
    effective_default: &DefaultModelConfig,
    catalog: Option<&ModelCatalog>,
    resolve_api_key_env: impl Fn(&str) -> String,
) {
    let is_default_provider = manifest.model.provider.is_empty() || manifest.model.provider == "default";
    let is_default_model = manifest.model.model.is_empty() || manifest.model.model == "default";

    if is_default_provider && is_default_model {
        if !effective_default.provider.is_empty() {
            manifest.model.provider = effective_default.provider.clone();
        }
        if !effective_default.model.is_empty() {
            manifest.model.model = effective_default.model.clone();
        }
        if !effective_default.api_key_env.is_empty() && manifest.model.api_key_env.is_none() {
            manifest.model.api_key_env = Some(effective_default.api_key_env.clone());
        }
        if effective_default.base_url.is_some() && manifest.model.base_url.is_none() {
            manifest.model.base_url.clone_from(&effective_default.base_url);
        }
    }

    if let Some(catalog) = catalog {
        if let Some(entry) = catalog.find_model(&manifest.model.model) {
            let provider_is_default =
                manifest.model.provider.is_empty() || manifest.model.provider == "default";
            if provider_is_default || manifest.model.provider == entry.provider {
                manifest.model.provider = entry.provider.clone();
                manifest.model.model = strip_provider_prefix(&entry.id, &entry.provider);
                if manifest.model.api_key_env.is_none() {
                    manifest.model.api_key_env = Some(resolve_api_key_env(&entry.provider));
                }
            }
        }
    }

    if manifest.model.api_key_env.is_none()
        && !manifest.model.provider.is_empty()
        && manifest.model.provider != "default"
    {
        manifest.model.api_key_env = Some(resolve_api_key_env(&manifest.model.provider));
    }

    let normalized = strip_provider_prefix(&manifest.model.model, &manifest.model.provider);
    if normalized != manifest.model.model {
        manifest.model.model = normalized;
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_model_target(
    effective_default: &DefaultModelConfig,
    provider: &str,
    model: &str,
    api_key_env: Option<&String>,
    base_url: Option<&String>,
    catalog: Option<&ModelCatalog>,
    lookup_provider_url: impl Fn(&str) -> Option<String>,
    resolve_api_key_env: impl Fn(&str) -> String,
) -> ResolvedModelTarget {
    let provider_was_default = provider.is_empty() || provider == "default";
    let model_was_default = model.is_empty() || model == "default";

    let mut resolved = ResolvedModelTarget {
        provider: if provider_was_default {
            effective_default.provider.clone()
        } else {
            provider.to_string()
        },
        model: if model_was_default {
            effective_default.model.clone()
        } else {
            model.to_string()
        },
        api_key_env: api_key_env.cloned(),
        base_url: base_url.cloned(),
    };

    if let Some(catalog) = catalog {
        if let Some(entry) = catalog.find_model(&resolved.model) {
            if provider_was_default || resolved.provider == entry.provider {
                resolved.provider = entry.provider.clone();
                resolved.model = strip_provider_prefix(&entry.id, &entry.provider);
            }
        }
    }

    resolved.model = strip_provider_prefix(&resolved.model, &resolved.provider);

    if resolved.api_key_env.is_none() && !resolved.provider.is_empty() {
        if provider_was_default
            && resolved.provider == effective_default.provider
            && !effective_default.api_key_env.is_empty()
        {
            resolved.api_key_env = Some(effective_default.api_key_env.clone());
        } else {
            resolved.api_key_env = Some(resolve_api_key_env(&resolved.provider));
        }
    }

    if resolved.base_url.is_none() {
        if provider_was_default
            && resolved.provider == effective_default.provider
            && effective_default.base_url.is_some()
        {
            resolved.base_url = effective_default.base_url.clone();
        } else if !resolved.provider.is_empty() {
            resolved.base_url = lookup_provider_url(&resolved.provider);
        }
    }

    resolved
}

/// Pick a sensible default embedding model for a given provider when the user
/// configured an explicit `embedding_provider` but left `embedding_model` at the
/// default value (which is a local model name that cloud APIs wouldn't recognise).
pub(crate) fn default_embedding_model_for_provider(provider: &str) -> &'static str {
    match provider {
        "openai" => "text-embedding-3-small",
        "mistral" => "mistral-embed",
        "cohere" => "embed-english-v3.0",
        "ollama" | "vllm" | "lmstudio" => "nomic-embed-text",
        _ => "text-embedding-3-small",
    }
}

/// Infer provider from a model name when catalog lookup fails.
///
/// Uses well-known model name prefixes to map to the correct provider.
/// This is a defense-in-depth fallback — models should ideally be in the catalog.
pub(crate) fn infer_provider_from_model(model: &str) -> Option<String> {
    let lower = model.to_lowercase();
    let (prefix, has_delim) = if let Some(idx) = lower.find('/') {
        (&lower[..idx], true)
    } else if let Some(idx) = lower.find(':') {
        (&lower[..idx], true)
    } else {
        (lower.as_str(), false)
    };
    if has_delim {
        if lower.chars().filter(|&c| c == '/').count() >= 2 {
            return Some(prefix.to_string());
        }
        match prefix {
            "minimax" | "gemini" | "anthropic" | "openai" | "groq" | "deepseek" | "mistral"
            | "cohere" | "xai" | "ollama" | "together" | "fireworks" | "perplexity"
            | "cerebras" | "sambanova" | "replicate" | "huggingface" | "ai21" | "codex"
            | "claude-code" | "copilot" | "github-copilot" | "qwen" | "zhipu" | "zai"
            | "moonshot" | "openrouter" | "volcengine" | "doubao" | "dashscope" => {
                return Some(prefix.to_string());
            }
            "kimi" => {
                return Some("moonshot".to_string());
            }
            _ => {}
        }
    }
    if lower.starts_with("minimax") {
        Some("minimax".to_string())
    } else if lower.starts_with("gemini") {
        Some("gemini".to_string())
    } else if lower.starts_with("claude") {
        Some("anthropic".to_string())
    } else if lower.starts_with("gpt")
        || lower.starts_with("o1")
        || lower.starts_with("o3")
        || lower.starts_with("o4")
    {
        Some("openai".to_string())
    } else if lower.starts_with("llama")
        || lower.starts_with("mixtral")
        || lower.starts_with("qwen")
    {
        None
    } else if lower.starts_with("grok") {
        Some("xai".to_string())
    } else if lower.starts_with("deepseek") {
        Some("deepseek".to_string())
    } else if lower.starts_with("mistral")
        || lower.starts_with("codestral")
        || lower.starts_with("pixtral")
    {
        Some("mistral".to_string())
    } else if lower.starts_with("command") || lower.starts_with("embed-") {
        Some("cohere".to_string())
    } else if lower.starts_with("jamba") {
        Some("ai21".to_string())
    } else if lower.starts_with("sonar") {
        Some("perplexity".to_string())
    } else if lower.starts_with("glm") {
        Some("zhipu".to_string())
    } else if lower.starts_with("ernie") {
        Some("qianfan".to_string())
    } else if lower.starts_with("abab") {
        Some("minimax".to_string())
    } else if lower.starts_with("moonshot") || lower.starts_with("kimi") {
        Some("moonshot".to_string())
    } else {
        None
    }
}
