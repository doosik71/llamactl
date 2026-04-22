# ezllama

`ezllama`는 `llama.cpp` 기반 도구를 터미널에서 쉽게 실행할 수 있게 해주는 Rust 애플리케이션입니다.

프로그램을 실행하면 다음 순서로 동작합니다.

1. CUDA Toolkit을 확인합니다.
2. `llama-cli`, `llama-server`, `llama-completion`을 확인합니다.
3. `llama.cpp`가 없으면 `nvidia-smi`로 CUDA architecture를 감지해 해당 값으로 빌드합니다.
4. 자동 감지에 실패하면 빌드 타겟 architecture를 목록에서 선택하게 합니다.
5. 실행 모드를 고릅니다. `client` 또는 `server`
6. 모델을 고릅니다. 직접 지정할 수도 있습니다.
7. 선택한 모드에 따라 `llama-cli`, `llama-server`, 또는 `llama-completion`을 실행합니다.

자동화 실행 모드에서는 상태 출력과 선택 UI를 건너뜁니다.

## 설치

### 요구 사항

- Rust toolchain
- `llama.cpp` 관련 실행 파일
- CUDA Toolkit은 선택 사항이지만, 현재 프로그램은 실행 전 확인을 수행합니다.
- NVIDIA GPU에서 CUDA 빌드를 자동 감지하려면 `nvidia-smi`가 필요합니다.

### 빌드

```bash
cargo build --release
```

설치:

```bash
cargo install --path .
```

### 실행

```bash
ezllama [options]
```

주의:

- `cargo run ezllama`처럼 `--` 없이 인자를 넘기면 Cargo가 아닌 프로그램 인자로 해석되지 않을 수 있습니다.
- 실제 인자는 `ezllama ...` 형태로 전달해야 합니다.

## 사용법

### 대화형 실행

아무 옵션도 주지 않으면 실행 모드와 모델을 순서대로 선택합니다.

```bash
ezllama
```

### 실행 모드 지정

`--mode client` 또는 `--mode server`를 사용하면 실행 모드 선택을 건너뜁니다.

```bash
ezllama --mode client
ezllama --mode server
```

### 모델 지정

`--model <model name>`를 사용하면 모델 선택을 건너뜁니다.

```bash
ezllama --model Qwen/Qwen3-4B-GGUF
```

### 대화형 client 실행

`--mode client`를 사용하면 선택한 모델로 `llama-cli`를 실행합니다.

```bash
ezllama --mode client --model Qwen/Qwen3-4B-GGUF
```

### server 실행

`--mode server`를 사용하면 선택한 모델로 `llama-server`를 실행합니다.

```bash
ezllama --mode server --model Qwen/Qwen3-4B-GGUF
```

## 명령행 옵션

- `--mode client|server`
  - 실행 모드를 지정합니다.
- `--model <name>`
  - 사용할 모델을 지정합니다.
- `--help`, `-h`
  - 도움말을 출력합니다.

제약:

- `--prompt`와 `--file`은 동시에 사용할 수 없습니다.
- `--prompt`와 `--file`은 `--mode client`에서만 사용할 수 있습니다.

## 동작 요약

- CUDA Toolkit이 없으면 설치를 시도할 수 있습니다.
- `llama-cli`, `llama-server`, `llama-completion`이 모두 있어야 정상 실행됩니다.
- `llama.cpp` 자동 설치 시 CUDA architecture를 먼저 자동 감지하고, 실패하면 목록에서 선택합니다.
- 모델 목록은 Hugging Face GGUF 검색 결과를 기반으로 표시됩니다.
- 모델 선택 화면에서는 방향키, `PageUp/PageDown`, `Enter`, `Esc`를 사용할 수 있습니다.

## 개발

테스트 실행:

```bash
cargo test
```

포맷 확인:

```bash
cargo fmt --check
```

## 라이선스

MIT
