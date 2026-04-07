# Multi-engine model backends

This project calls an OpenAI-style API:

- `POST {base_url}/chat/completions`
- `POST {base_url}/embeddings`

In Docker Model Runner (DMR), that base URL is typically:

- `http://localhost:12434/engines/{engine}/v1`

Where `{engine}` identifies the backend implementation (for example `llama.cpp`).

## Current configuration (single engine)

By default, this repo uses a **single engine** for both chat and embeddings:

- `models.engine`: one engine name

This is the simplest setup and is usually best when everything runs under the same backend (like DMR + `llama.cpp`).

## When multiple engines are beneficial

You might want to use **different engines** when different tasks have different performance/quality requirements.

Common reasons:

- **Performance specialization**
  - Chat can be latency-sensitive.
  - Embeddings can be throughput/batching-sensitive.
  - Reranking can be CPU/GPU-heavy and benefits from high-throughput backends.

- **Different model formats**
  - Some backends are optimized for GGUF (often `llama.cpp`).
  - Other backends might prefer safetensors / FP16 (often `vLLM`-style setups).

- **Hardware differences**
  - You may want chat on GPU and embeddings on CPU (or vice versa) depending on load and cost.

## How we would represent multi-engine in config

If/when you want multi-engine support, a clean shape is:

```json
{
  "models": {
    "api_url": "http://localhost:12434/engines/{engine}/v1",
    "chat_engine": "llama.cpp",
    "embeddings_engine": "llama.cpp",
    "llm_model": "ai/ministral3",
    "embedding_model": "ai/mxbai-embed-large"
  }
}
```

Then:

- Chat uses `{engine} = chat_engine`
- Embeddings uses `{engine} = embeddings_engine`

## Notes for Docker Model Runner

- `docker model status` shows which backends are available.
- If a backend isn’t available (e.g. `vllm` “not implemented”), you can still download models, but you won’t be able to serve them via that backend on this machine.

