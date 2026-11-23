use crate::domain::ports::UsageProviderPort;
use crate::domain::value_objects::ProviderKind;
use crate::infrastructure::providers::amazon_bedrock::AmazonBedrockAdapter;
use crate::infrastructure::providers::antigravity::AntigravityAdapter;
use crate::infrastructure::providers::chinese_apis::{
    GenericBearerProbeAdapter, MoonshotKimiAdapter, SiliconFlowAdapter,
};
use crate::infrastructure::providers::claude_code::ClaudeCodeAdapter;
use crate::infrastructure::providers::deepseek::DeepSeekAdapter;
use crate::infrastructure::providers::global_apis::{
    AnthropicApiAdapter, CursorIdeAdapter, GithubCopilotAdapter, OpenAiAdapter,
};
use crate::infrastructure::providers::openrouter::OpenRouterAdapter;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub fn build_provider_registry() -> HashMap<ProviderKind, Arc<dyn UsageProviderPort>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("DamnUsage/0.1.0")
        .build()
        .expect("http client must construct");

    let mut map: HashMap<ProviderKind, Arc<dyn UsageProviderPort>> = HashMap::new();

    map.insert(ProviderKind::Antigravity, Arc::new(AntigravityAdapter::new()));
    map.insert(ProviderKind::ClaudeCode, Arc::new(ClaudeCodeAdapter::new()));
    map.insert(
        ProviderKind::DeepSeek,
        Arc::new(DeepSeekAdapter::new(client.clone())),
    );
    map.insert(
        ProviderKind::OpenRouter,
        Arc::new(OpenRouterAdapter::new(client.clone())),
    );
    map.insert(
        ProviderKind::AmazonBedrock,
        Arc::new(AmazonBedrockAdapter::new(client.clone())),
    );
    map.insert(
        ProviderKind::OpenAi,
        Arc::new(OpenAiAdapter::new(client.clone())),
    );
    map.insert(
        ProviderKind::AnthropicApi,
        Arc::new(AnthropicApiAdapter::new(client.clone())),
    );
    map.insert(
        ProviderKind::SiliconFlow,
        Arc::new(SiliconFlowAdapter::new(client.clone())),
    );
    map.insert(
        ProviderKind::MoonshotKimi,
        Arc::new(MoonshotKimiAdapter::new(client.clone())),
    );
    map.insert(
        ProviderKind::CursorIde,
        Arc::new(CursorIdeAdapter::new(client.clone())),
    );
    map.insert(
        ProviderKind::GithubCopilot,
        Arc::new(GithubCopilotAdapter::new(client.clone())),
    );

    let generic_specs: &[(ProviderKind, &'static str, &'static str)] = &[
        (
            ProviderKind::GoogleGemini,
            "https://generativelanguage.googleapis.com/v1beta/models",
            "generativelanguage.googleapis.com",
        ),
        (
            ProviderKind::XaiGrok,
            "https://api.x.ai/v1/models",
            "api.x.ai/v1",
        ),
        (
            ProviderKind::ZhipuGlm,
            "https://open.bigmodel.cn/api/paas/v4/models",
            "open.bigmodel.cn",
        ),
        (
            ProviderKind::QwenDashScope,
            "https://dashscope.aliyuncs.com/compatible-mode/v1/models",
            "dashscope.aliyuncs.com",
        ),
        (
            ProviderKind::Groq,
            "https://api.groq.com/openai/v1/models",
            "api.groq.com/openai/v1",
        ),
        (
            ProviderKind::TogetherAi,
            "https://api.together.xyz/v1/models",
            "api.together.xyz/v1",
        ),
        (
            ProviderKind::MistralAi,
            "https://api.mistral.ai/v1/models",
            "api.mistral.ai/v1",
        ),
        (
            ProviderKind::FireworksAi,
            "https://api.fireworks.ai/inference/v1/models",
            "api.fireworks.ai",
        ),
        (
            ProviderKind::PerplexityAi,
            "https://api.perplexity.ai/models",
            "api.perplexity.ai",
        ),
    ];

    for &(kind, url, source_label) in generic_specs {
        map.insert(
            kind,
            Arc::new(GenericBearerProbeAdapter::new(
                client.clone(),
                kind,
                url,
                source_label,
            )),
        );
    }

    map
}
