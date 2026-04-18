# PRD: easyllama (Rust 기반 llama.cpp 터미널 UI 컨트롤러)

사용자가 복잡한 파라미터 학습 없이 `llama-cli`의 강력한 기능을 터미널에서 직관적으로 제어할 수 있도록 돕는 Rust 기반의 TUI(Terminal User Interface) 도구, **easyllama**의 제품 요구사항 문서입니다.

---

## 1. 제품 개요 (Product Overview)

* **목적**: `llama.cpp`의 코어 성능을 유지하되 사용자 경험(UX)을 극대화한 가벼운 Rust 기반 인터페이스 제공.
* **타겟 사용자**: 로컬 LLM을 사용하지만 CLI 파라미터 설정에 피로감을 느끼는 사용자, 서버 환경에서 GUI 없이 고성능 추론을 원하는 개발자.
* **핵심 가치**: "Zero-Learning, Full-Performance."

---

## 2. 주요 기능 (Key Features)

### 2.1 환경 자동 구성 (Auto-Provisioning)

* **종속성 체크**: 시스템 내 CUDA Toolkit, C++ 컴파일러, CMake 존재 여부 확인.
* **Smart Build**: 사용자의 GPU 환경(NVIDIA, Apple Metal, Intel/AMD 등)을 자동 감지하여 최적의 빌드 플래그로 `llama.cpp`를 자동 설치 및 컴파일.
* **모델 관리**: Hugging Face 리포지토리 ID만으로 모델 다운로드부터 양자화 선택까지 가이드.

### 2.2 고성능 TUI 레이아웃 (`ratatui` 활용)

* **Chat Dashboard**: 실시간 텍스트 스트리밍과 마크다운 스타일링이 적용된 대화창.
* **Live Parameter Tuner**: 대화 중에도 온도($$Temperature$$), Context Size, Penalty 등을 단축키와 슬라이더로 즉시 변경.
* **Resource Monitor**: 하단 상태 바에 실시간 CPU/GPU 사용량 및 $Token/s$ 속도 상시 표시.

### 2.3 지능형 추론 제어

* **Speculative Decoding UI**: 메인 모델과 드래프트 모델 간의 관계를 시각화하고 설정 관리.
* **Reasoning Parser**: DeepSeek 등의 추론 모델 사용 시 `<think>` 과정을 별도의 색상으로 분리하여 가독성 확보.
* **Structured Output**: JSON 스키마 적용 시 문법 오류를 방지하는 실시간 가이드 제공.

---

## 3. 기술 스택 (Technical Stack)

* **Language**: Rust (Memory Safety & Zero-cost Abstractions)
* **TUI Library**: `ratatui` - 인터랙티브하고 반응성이 뛰어난 UI 구성.
* **Runtime**: `tokio` - 비동기 프로세스 제어 및 `llama-cli`와의 양방향 스트리밍.
* **Build Tool**: `cc` & `cmake` crate - Rust 내부에서 `llama.cpp` 빌드 자동화 제어.

---

## 4. 사용자 여정 (User Journey)

1. **설치**: `cargo install easyllama` 실행.
2. **초기화**: `easyllama init` 명령어로 하드웨어 최적화 빌드 수행 (CUDA 자동 설정 포함).
3. **모델 로드**: TUI 내에서 모델 리스트를 탐색하거나 Hugging Face 주소를 붙여넣어 로드.
4. **추론**: 직관적인 패널을 통해 파라미터를 조절하며 모델과 대화.
5. **관리**: 세션 로그를 저장하거나 커스텀 파라미터 프로필을 생성하여 저장.

---

## 5. 상세 기능 요구사항 (Functional Requirements)

| 우선순위 | 기능명 | 상세 내용 |
| :--- | :--- | :--- |
| **P0** | **Process Wrapping** | `llama-cli`를 서브프로세스로 실행하고 가상 터미널(PTY)을 통해 제어 |
| **P0** | **Interactive UI** | 단축키(Hotkeys) 기반의 화면 전환 및 파라미터 실시간 조정 패널 |
| **P1** | **Auto-Toolchain** | CUDA Toolkit 미설치 시 가이드 및 `llama.cpp` 자동 빌드 자동화 |
| **P1** | **Model Cache** | GGUF 파일 관리 및 자동 양자화 도구(`llama-quantize`) 연동 |
| **P2** | **Multi-modal Support** | 터미널 내 이미지 경로 입력 및 비전 모델 결과 처리 |
| **P2** | **Session History** | SQLite를 활용한 과거 대화 내역 검색 및 컨텍스트 복원 |

---

## 6. 비기능 요구사항 (Non-Functional Requirements)

* **Zero Dependency**: 빌드 후 바이너리 하나로 동작하는 정적 링크 지향.
* **Latency**: UI 오버헤드를 전체 연산의 **1% 미만**으로 유지.
* **Cross-Platform**: Linux(Native/WSL2), macOS(Metal) 환경에서 동일한 UX 보장.

---

## 7. 향후 로드맵 (Roadmap)

* **v0.1.0**: 기본 TUI 대화 기능 및 `llama.cpp` 소스 빌드 자동화.
* **v0.5.0**: 시각적 파라미터 프로필 관리 및 Hugging Face Hub 연동 강화.
* **v1.0.0**: 정식 릴리즈 및 Speculative Decoding 시각화 완성.
