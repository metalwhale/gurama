# gurama
A cute llama that helps you correct English grammar.
<p align="center">
  <img src="https://github.com/metalwhale/gurama/blob/main/icon.jpg" width="128" height="128" />
</p>

## Development
1. Download [`openbuddy-openllama-7b-v5-q4_0.bin`](https://huggingface.co/metalwhale/openbuddy-openllama-7b-v5-q4_0/blob/main/openbuddy-openllama-7b-v5-q4_0.bin) file and put it into [`./model`](./model/) directory
2. Start and get inside the container:
    ```bash
    cd ./infra-dev
    docker-compose up -d
    docker-compose exec -it app bash
    ```

## Kudos
- [OpenBuddy - Open Multilingual Chatbot](https://huggingface.co/OpenBuddy/openbuddy-openllama-7b-v5-fp16)
- [llama.cpp](https://github.com/ggerganov/llama.cpp)
- [Large Language Models for Everyone, in Rust](https://github.com/rustformers/llm)
- The icon was generated on https://dreamlike.art using `Dreamlike Anime 1.0` model
