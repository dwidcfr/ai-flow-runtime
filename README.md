# ai-flow-runtime

Personal open-source stack for building and running AI conversation flows: Rust runtime engines, HTTP API, and a unified web console.

## Components

| Path | Description |
|------|-------------|
| `ai-flow-runtime` | Core runtime: sessions, routing, response, search |
| `ai-api` | HTTP API + Studio authoring endpoints |
| `ai-platform-web` | React console (Projects, Studio, Playground, Evals) |
| `ai-prompts` | Prompt registry and template rendering |
| `ai-flow-engine` | Flow graph loader and validator |
| `ai-evals` | Scenario-based evaluation harness |

## Quick start

```bash
cp .env.example .env   # set GEMINI_API_KEY if using live Gemini

# API :8080 + web :3000
./ai-platform-web/platform.sh
```

## Demo data

Sample project `insurance_demo` uses fictional client data and the neutral `demo` prompt set (`Alex` / `Acme Insurance`). No production credentials are included.

## Notes

Requires a Gemini API key only when `API_USE_GEMINI=true`. Mock LLMs work without external services for local development and tests.
