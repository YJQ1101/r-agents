## Usage

The library reads [API key](https://platform.openai.com/account/api-keys) from the environment variable `OPENAI_API_BASE` and `OPENAI_API_KEY`.

Meanwhile, there is also the proxy address of the local machine `HTTP_PORT`
```bash
export OPENAI_API_BASE='gpt4o'
export OPENAI_API_KEY='sk-...'
export HTTP_PORT=8848
```

## Writing Your Own Agents

Building agents is remarkably straightforward. 


Create a new yaml in the [./src/agents/](./src/agents/) directory (.e.g. `demo`).

## Writing Your Own Tools

Building tools is remarkably straightforward. 

Create a new yaml file and an executable file in the [./src/tools/](./src/tools/) directory (.e.g. `get_current_time`).
