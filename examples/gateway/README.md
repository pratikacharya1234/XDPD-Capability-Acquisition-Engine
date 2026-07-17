# XDPD Gateway — LLM Inference Proxy Demo

A demo service that sits between your application and any LLM API (OpenAI, Anthropic, etc.). It learns repeated reasoning patterns and serves cached responses instantly — zero API cost, zero latency, on a repeat match.

This is a demo/reference implementation, not a hardened production proxy — no auth, no rate limiting, no retry/backoff logic on the upstream call.

## Quick Start

```bash
cargo run --release
```

Server starts on `http://127.0.0.1:8080`.

## How It Works

```
Client → XDPD Gateway → LLM API
              |
         subroutine table
         (the cache — no Redis, no vector DB, no extra infra)
```

1. Request arrives at the gateway
2. Gateway hashes the prompt and checks the subroutine table
3. If pattern is learned: return cached response instantly (0 API cost)
4. If new: forward to LLM, learn the pattern, cache the response, return it

## API Endpoints

### Chat Completions (OpenAI-compatible)

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-5.6-sol","messages":[{"role":"user","content":"What is 2+2?"}]}'
```

First call: cache miss, generates response (or forwards to upstream LLM).
Second call with same prompt: cache hit, instant response, zero cost.

### Stats

```bash
curl http://localhost:8080/stats
```

```json
{
  "uptime_secs": 3600,
  "total_requests": 5000,
  "cache_hits": 1750,
  "hit_rate_pct": 35.0,
  "tokens_saved": 210000,
  "skills_learned": 12,
  "engine": "xdpd-gateway",
  "version": "0.2.1"
}
```

(Numbers above are illustrative field shapes, not a measured run — `version` reflects whatever `xdpd` version the gateway was built against.)

### Health Check

```bash
curl http://localhost:8080/health
```

### Learned Skills

```bash
curl http://localhost:8080/skills
```

## Running Against a Real LLM

```bash
# Point at your real LLM endpoint
OPENAI_API_KEY=sk-... UPSTREAM_URL=https://api.openai.com/v1/chat/completions cargo run --release

# Custom port
PORT=3000 cargo run --release
```

Without `OPENAI_API_KEY`, the gateway runs in simulation mode — useful for testing and demos.

## Why This Matters (Illustrative, Not Measured)

Every LLM API call costs money. When users ask the same types of questions —
"Summarize this document," "Extract entities from this text," "Format this
as JSON" — the LLM goes through the same reasoning path. XDPD learns these
patterns and shortcuts them on a repeat.

The 35% hit rate / $135/month figure in the stats example above is a
hypothetical scenario, not a measurement from a real deployment — nobody
has run this gateway against production LLM traffic and reported real
savings yet. Treat it as a shape for your own numbers once you do, not a
benchmark.

## Architecture

The cache is NOT Redis. It's NOT a vector database. It's the XDPD subroutine table — a growing instruction set that learns patterns and compiles them into reusable subroutines. When the system learns, it adds a word to its language. That word is computational, not textual.
