# 🦙 easyllama

**easyllama**는 `llama.cpp`의 강력한 성능을 복잡한 파라미터 설정 없이 터미널에서 직관적으로 사용할 수 있게 해주는 Rust 기반의 TUI(Terminal User Interface) 컨트롤러입니다.

Ollama의 정책 변화에 구애받지 않고, 로컬 환경에서 가장 빠르고 자유롭게 LLM을 활용하세요.

---

## ✨ 핵심 기능

- **🚀 원클릭 환경 구축**: 실행 시 시스템의 GPU(CUDA, Metal 등)를 자동 감지하여 `llama.cpp`를 최적화된 상태로 자동 빌드 및 설치합니다.
- **🎮 직관적인 TUI**: 수십 개의 CLI 인자를 외울 필요 없습니다. 터미널 화면 내 대시보드에서 온도(Temp), 컨텍스트 크기 등을 실시간으로 조절하세요.
- **⚡ Rust 기반의 고성능**: 최소한의 리소스만 사용하여 모델 추론 속도에 영향을 주지 않으면서도 매끄러운 UI 경험을 제공합니다.
- **🧠 지능형 파싱**: DeepSeek과 같은 모델의 `<think>` 과정을 별도 창으로 분리하여 가독성을 높였습니다.
- **📂 모델 관리자**: Hugging Face 리포지토리에서 GGUF 모델을 직접 다운로드하고 관리할 수 있습니다.

---

## 🛠 설치 방법

### 사전 요구 사항

- **Rust Toolchain**: `rustup`이 설치되어 있어야 합니다.
- **C++ Compiler**: `gcc` 또는 `clang` (llama.cpp 빌드용).
- **CUDA Toolkit** (선택 사항): NVIDIA GPU 가속을 사용하려는 경우.

### 설치

```bash
# 저장소 클론
git clone https://github.com/your-repo/easyllama.git
cd easyllama

# 빌드 및 설치
cargo install --path .
```

---

## 🚀 시작하기

### 1. 초기화 및 빌드

처음 실행 시 `init` 명령어를 통해 환경에 맞는 `llama.cpp`를 자동으로 빌드합니다.

```bash
# 하드웨어 감지 및 llama.cpp 자동 빌드
easyllama init
```

### 2. 모델 실행

모델 경로를 지정하거나 대화 모드로 진입합니다.

```bash
# 대화형 TUI 실행
easyllama chat --model ./path/to/model.gguf
```

### 3. 주요 단축키

- `Tab`: 대화창과 파라미터 설정 패널 간 이동
- `Ctrl + S`: 현재 설정 저장
- `Ctrl + C`: 대화 중단 및 종료
- `↑ / ↓`: 파라미터 값 조절

---

## 🖥 화면 구성 (UI Layout)

- **Main Chat**: 마크다운 스타일이 적용된 실시간 스트리밍 대화창.
- **Side Panel**: `Temperature`, `Top-P`, `Repeat Penalty` 등 주요 파라미터 슬라이더.
- **Bottom Bar**: 실시간 $Token/s$ 속도 및 GPU 메모리 점유율 표시.

---

## 🛠 기술 스택

- **Core**: [Rust](https://www.rust-lang.org/)
- **UI**: [ratatui](https://github.com/ratatui-org/ratatui)
- **Engine**: [llama.cpp](https://github.com/ggerganov/llama.cpp)
- **Async**: [Tokio](https://tokio.rs/)

---

## 🤝 기여하기

`easyllama`는 오픈소스 프로젝트입니다. 이슈 보고, 기능 제안, PR(Pull Request)은 언제나 환영합니다!

1. 저장소를 Fork 합니다.
2. 새로운 브랜치를 생성합니다 (`git checkout -b feature/amazing-feature`).
3. 변경 사항을 Commit 합니다 (`git commit -m 'Add some amazing feature'`).
4. 브랜치에 Push 합니다 (`git push origin feature/amazing-feature`).
5. Pull Request를 생성합니다.

---

## 📄 라이선스

이 프로젝트는 MIT 라이선스에 따라 배포됩니다. 자세한 내용은 `LICENSE` 파일을 참조하세요.

---

**easyllama**와 함께 로컬 LLM의 진정한 자유를 경험하세요! 🦙✨
