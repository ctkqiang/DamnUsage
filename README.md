# DamnUsage

macOS Notch LLM API & Coding Agent Usage Monitor built with Tauri v2, Rust CQRS + Hexagonal Architecture, Framer Motion spring physics, and English / Chinese localization.

## Supported Providers (20)

- **Coding Agents**: Antigravity (AGY), Claude Code, Cursor, GitHub Copilot
- **Cloud Platforms**: Amazon Bedrock (AWS SigV4 Cost Explorer & CloudWatch), Google Gemini / Vertex AI
- **Aggregators**: OpenRouter, SiliconFlow (硅基流动), Together AI, Fireworks AI
- **Global APIs**: OpenAI, Anthropic API, xAI Grok, Groq, Mistral AI, Perplexity AI
- **Chinese LLM APIs**: DeepSeek, Moonshot Kimi (月之暗面), Zhipu GLM (智谱), Qwen DashScope (通义千问)

## Architecture

- **Backend (`src-tauri/crates/damnusage-core`)**: Hexagonal (Ports & Adapters) + CQRS (CommandBus / QueryBus) + Event-Driven Materialized Projections (`UsageProjector`)
- **Desktop Shell (`src-tauri`)**: Tauri v2 macOS borderless transparent Notch overlay (`NSStatusWindowLevel`, hardware notch `safeAreaInsets` detection, Tray menu, reactive domain event streaming)
- **Frontend (`src`)**: React + TypeScript + Tailwind CSS + Framer Motion spring animations + `i18next` (`en` / `zh-CN` / `zh-TW`) + CQRS IPC bridge

## Build on macOS

```bash
npm install
npm run tauri dev
```

For production `.dmg` / `.app` bundle:

```bash
npm run tauri build
```
