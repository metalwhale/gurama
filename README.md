# gurama
A cute llama that helps you correct English grammar.
<p align="center">
  <img src="https://github.com/metalwhale/gurama/blob/main/icon.jpg" width="128" height="128" />
</p>

## Development
1. Download [`openbuddy-openllama-7b-v5-q4_0.bin`](https://huggingface.co/metalwhale/openbuddy-openllama-7b-v5-q4_0/blob/main/openbuddy-openllama-7b-v5-q4_0.bin) file and put it into [`./model`](./model/) directory
2. Get inside the container:
    ```bash
    cd ./infra-dev
    docker-compose up -d
    docker-compose exec -it app bash
    ```
3. Start the server:
    ```bash
    cd gurama/
    cargo run --release
    ```
4. Testing:
    ```bash
    curl -H "Content-Type: application/json" -X POST http://localhost:7860/correct -d '{"sentence":"Is you okay?"}'
    ```
    ```bash
    {"corrected_sentence":"'Are you okay?'."}
    ```

## Release build
Build the image (Replace **`DOCKER_HUB_USERNAME`** with your Docker Hub username)
<pre>
cd infra-release/
docker buildx create --name multi-platform-builder --platform linux/arm64,linux/amd64 --bootstrap --use
docker buildx build --platform linux/amd64,linux/arm64 -f Dockerfile -t <b>DOCKER_HUB_USERNAME</b>/gurama --push ../
docker run --rm -p 7860:7860 -e GURAMA_APP_PORT=7860 <b>DOCKER_HUB_USERNAME</b>/gurama
</pre>

## Live demo
Send request to [HuggingFace space](https://huggingface.co/spaces/metalwhale/gurama)
```bash
curl -H "Content-Type: application/json" -X POST https://metalwhale-gurama.hf.space/correct -d '{"sentence":"Is you okay?"}'
```

## Kudos
- [OpenBuddy - Open Multilingual Chatbot](https://huggingface.co/OpenBuddy/openbuddy-openllama-7b-v5-fp16)
- [llama.cpp](https://github.com/ggerganov/llama.cpp)
- [Large Language Models for Everyone, in Rust](https://github.com/rustformers/llm)
- The icon was generated on https://dreamlike.art using `Dreamlike Anime 1.0` model
