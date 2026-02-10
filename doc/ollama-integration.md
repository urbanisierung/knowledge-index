# Ollama Integration Guide

Use kdex with [Ollama](https://ollama.ai/) for a completely local AI workflow. Your code and knowledge never leave your machine.

## Why Local LLMs?

- **Privacy**: Your code and notes stay on your hardware
- **Offline**: Works without internet connection
- **Cost**: No API fees, unlimited queries
- **Speed**: No network latency for local models

## Quick Start

### 1. Install Ollama

```bash
# macOS / Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Or download from https://ollama.ai/
```

### 2. Pull a Model

```bash
# Recommended for code
ollama pull codellama

# Or for general knowledge
ollama pull llama3
ollama pull mistral
```

### 3. Index Your Knowledge

```bash
# Index your codebase
kdex index ~/projects/my-app

# Index your notes
kdex index ~/Documents/obsidian-vault
```

### 4. Search and Ask

```bash
# Search, then ask Ollama about results
kdex search "authentication" --json | \
  jq -r '.results[:3] | .[].snippet' | \
  ollama run codellama "Explain how authentication works in this code:"
```

## Usage Patterns

### Code Explanation

```bash
# Find and explain a function
kdex search "handleLogin" --json | \
  jq -r '.results[0].snippet' | \
  ollama run codellama "Explain this function step by step:"
```

### Documentation Q&A

```bash
# Search your notes and get answers
kdex search "kubernetes deployment" --json | \
  jq -r '.results[].snippet' | \
  ollama run llama3 "Based on my notes, how do I deploy to Kubernetes?"
```

### Code Review

```bash
# Get review suggestions for specific patterns
kdex search "TODO\|FIXME\|HACK" --json | \
  jq -r '.results[].snippet' | \
  ollama run codellama "Review these code comments and suggest improvements:"
```

### Debugging Help

```bash
# Search error messages in your docs
kdex search "connection refused" --json | \
  jq -r '.results[].snippet' | \
  ollama run llama3 "Based on my documentation, how do I fix this error?"
```

## Shell Functions

Add these to your `.bashrc` or `.zshrc`:

```bash
# Ask about your codebase
ask-code() {
  kdex search "$1" --json | \
    jq -r '.results[:5] | .[].snippet' | \
    ollama run codellama "$2"
}

# Ask about your notes
ask-notes() {
  kdex search "$1" --json | \
    jq -r '.results[:5] | .[].snippet' | \
    ollama run llama3 "$2"
}

# Usage:
# ask-code "database connection" "How does the app connect to the database?"
# ask-notes "meeting notes" "Summarize my recent meetings"
```

## Advanced: Building Context Windows

For complex questions, build a comprehensive context:

```bash
#!/bin/bash
# context-query.sh - Build context and query Ollama

QUERY="$1"
QUESTION="$2"
MODEL="${3:-llama3}"

# Gather context from kdex
CONTEXT=$(kdex search "$QUERY" --json --limit 10 | \
  jq -r '.results[] | "File: \(.file)\n\(.snippet)\n---"')

# Build prompt
PROMPT="Based on the following context from my local files:

$CONTEXT

Question: $QUESTION"

# Query Ollama
echo "$PROMPT" | ollama run "$MODEL"
```

Usage:
```bash
chmod +x context-query.sh
./context-query.sh "API endpoints" "List all the API endpoints in my project"
```

## Performance Tips

### Model Selection

| Model | Best For | RAM Required |
|-------|----------|--------------|
| `codellama:7b` | Code understanding | 8GB |
| `codellama:13b` | Complex code analysis | 16GB |
| `llama3:8b` | General knowledge Q&A | 8GB |
| `mistral:7b` | Fast, general purpose | 8GB |
| `deepseek-coder:6.7b` | Code generation | 8GB |

### Optimize Search Results

```bash
# Limit results for faster context
kdex search "query" --limit 5 --json

# Filter by file type
kdex search "query" --type rust --json

# Filter by repository
kdex search "query" --repo my-app --json
```

### Reduce Token Usage

```bash
# Extract only snippets (less tokens)
kdex search "query" --json | jq -r '.results[].snippet'

# Get just file paths for reference
kdex search "query" --json | jq -r '.results[].absolute_path'
```

## Comparison: Cloud vs Local

| Aspect | Cloud AI (GPT-4, Claude) | Local (Ollama + kdex) |
|--------|--------------------------|----------------------|
| Privacy | Data sent to servers | 100% local |
| Cost | Per-token pricing | Free after setup |
| Speed | Network dependent | Local inference |
| Offline | ❌ Requires internet | ✅ Works offline |
| Context | Limited by API | Full file access |

## Troubleshooting

### Ollama not responding

```bash
# Check if Ollama is running
ollama list

# Start Ollama service
ollama serve
```

### Out of memory

```bash
# Use a smaller model
ollama run codellama:7b

# Or set memory limit
OLLAMA_MAX_LOADED_MODELS=1 ollama run llama3
```

### Slow responses

- Use smaller models (7B vs 13B)
- Limit search results with `--limit`
- Consider GPU acceleration (Ollama supports CUDA/Metal)

## See Also

- [MCP Integration](mcp-integration.md) — For GitHub Copilot and Claude Desktop
- [Documentation](documentation.md) — Full kdex usage guide
- [Ollama Documentation](https://github.com/ollama/ollama) — Ollama setup and models
