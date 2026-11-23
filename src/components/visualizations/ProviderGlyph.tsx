import React from "react";
import type { ProviderKind } from "../../cqrs/types";

interface ProviderGlyphProps {
  kind: ProviderKind;
  shortName: string;
  accentHex: string;
  secondaryHex: string;
  size?: "sm" | "md" | "lg";
}

const GLYPH_MAP: Record<ProviderKind, string> = {
  antigravity: "AG",
  claude_code: "CC",
  deep_seek: "DS",
  open_router: "OR",
  amazon_bedrock: "BR",
  open_ai: "OA",
  anthropic_api: "AN",
  google_gemini: "GM",
  xai_grok: "xA",
  silicon_flow: "SF",
  moonshot_kimi: "KM",
  zhipu_glm: "GL",
  qwen_dash_scope: "QW",
  groq: "GQ",
  together_ai: "TG",
  mistral_ai: "MS",
  fireworks_ai: "FW",
  perplexity_ai: "PX",
  cursor_ide: "CR",
  github_copilot: "CP",
};

export const ProviderGlyph: React.FC<ProviderGlyphProps> = ({
  kind,
  shortName,
  accentHex,
  secondaryHex,
  size = "md",
}) => {
  const glyph = GLYPH_MAP[kind] ?? shortName.slice(0, 2).toUpperCase();
  const dims =
    size === "sm"
      ? "h-6 w-6 text-[10px] rounded-lg"
      : size === "lg"
        ? "h-10 w-10 text-sm rounded-xl"
        : "h-8 w-8 text-xs rounded-xl";

  return (
    <div
      className={`relative flex shrink-0 items-center justify-center font-mono font-bold text-white shadow-inner ${dims}`}
      style={{
        background: `linear-gradient(135deg, ${accentHex} 0%, ${secondaryHex} 100%)`,
        boxShadow: `0 6px 18px -6px ${accentHex}99`,
      }}
    >
      <span className="tracking-tight drop-shadow-sm">{glyph}</span>
    </div>
  );
};
