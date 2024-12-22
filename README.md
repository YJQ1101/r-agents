# r-agents: Build Your Own LLM Agents
r-agents provide a convenient way to build your exclusive agents.

## Prepare

The library reads [API key](https://platform.openai.com/account/api-keys) from the environment variable `OPENAI_API_BASE` and `OPENAI_API_KEY`.

Meanwhile, there is also the proxy address of the local machine `HTTP_PORT`
```bash
export OPENAI_API_BASE=https://api.openai.com/v1
export OPENAI_API_KEY='sk-...'
export HTTP_PORT=8848
```

If you use ollama, you can set
```bash
export OPENAI_API_BASE=http://localhost:11434/v1
```

## Usage
`-a agents.yaml` specifies the agent configuration file.
`-t tools.yaml` specifies the tool configuration file.
`--agent demo` specifies that the related agent should be used.
```
cargo run -- -a agents.yaml -t tools.yaml --agent demo
```


## Local Server Proxy
r-agent implements agent ability through local proxy, user request LLM after being modified by the agent you are using
```bash
Chat Completions API: http://127.0.0.1:8848/v1/chat/completions
```

## Example
```json
curl -v http://localhost:8848/r-agents/v1/chat/completions \
-H "Content-Type: application/json" \
  -d '{
    "model": "llama3.2",
    "messages": [
      {
        "role": "user",
        "content": "What is the weather like?"
      }
    ]
  }'
```

## Writing Your Own Agents

Building agents is remarkably straightforward. 


Create a new yaml in the [./src/agents/](./src/agents/) directory (.e.g. `demo`).

## Writing Your Own Tools

Building tools is remarkably straightforward. 

Create a new json file and an executable file in the [./src/tools/](./src/tools/) directory (.e.g. `get_current_time`).
