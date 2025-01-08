# r-agents: Build Your Own LLM Agents Without Writing Code
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

Through the `autogen.sh` script first build `agents.yaml`, `tools.yaml`, `rags.yaml`, this script will read the current set of tools, rags and agent, to prevent the use of the process can not find our configured files.
```bash
./autogen.sh
```

To implement tool and document selection, we need to set up the vector database to match the tool and document that best matches the current context.
```bash
cargo run --bin chromadb -- -t tools.yaml  -r rags.yaml
```
**Note that** you have to install the vector database locally or in docker first!

## Usage
`-a agents.yaml` specifies the agent configuration file.
`-t tools.yaml` specifies the tool configuration file.
`--agent demo` specifies that the related agent should be used.
`-d chromadb` specifies that the vector database should be used.

```
cargo run --bin ragents -- -a agents.yaml --agent coder -t tools.yaml -d chromadb
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

Building agent is remarkably straightforward. 


Create a new yaml in the [./src/agents/](./src/agents/) directory (.e.g. `demo`).

## Writing Your Own Tools

Building tool is remarkably straightforward. 

Create a new json file and an executable file in the [./src/tools/](./src/tools/) directory (.e.g. `get_current_time`).

## Writing Your Own RAG
Building rag is remarkably straightforward. 
Create a new yaml in the [./src/rags/](./src/rags/) directory (.e.g. `demo`).
