# Building

### Prerequisites

[Bun](https://bun.sh/) | [Cargo](https://www.rust-lang.org/tools/install) | [Clang](https://releases.llvm.org/download.html) | [Cmake](https://cmake.org/download/)

**Windows**:

Tools: [vcpkg](https://vcpkg.io/en/)

`vcpkg` packages

```console
C:\vcpkg\vcpkg.exe install opencl
```

`Winget` packages

```console
winget install -e --id JernejSimoncic.Wget
winget install -e --id 7zip.7zip
```

**Linux**:

Based on [tauri/prerequisites/#setting-up-linux](https://tauri.app/v1/guides/getting-started/prerequisites/#setting-up-linux)

```console
sudo apt-get update
sudo apt-get install -y ffmpeg libopenblas-dev # runtime
sudo apt-get install -y pkg-config build-essential libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev clang cmake libssl-dev # tauri
sudo apt-get install -y libavutil-dev libavformat-dev libavfilter-dev libavdevice-dev # ffmpeg
```

## Build

Install dependencies from `desktop` folder

```console
bun install
```

Execute pre build scripts and follow the instructions it provide

```console
bun scripts/pre_build.js
```

## Build with `Nvidia` support

See [whisper.cpp#nvidia-support](https://github.com/ggerganov/whisper.cpp?tab=readme-ov-file#nvidia-gpu-support)

1. Enable `cuda` feature in `Cargo.toml`

2. Install [`cuda`](https://developer.nvidia.com/cuda-downloads)

3. Add to `tauri.windows.conf.json`

<details>

<summary>tauri.windows.conf.json</summary>

```json
{
	"bundle": {
		"resources": {
			"ffmpeg\\bin\\x64\\*.dll": "./",
			"openblas\\bin\\*.dll": "./",
			"clblast\\bin\\*.dll": "./",
			"C:\\vcpkg\\packages\\opencl_x64-windows\\bin\\*.dll": "./",
			"C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA\\v12.5\\bin\\cudart64_*": "./",
			"C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA\\v12.5\\bin\\cublas64_*": "./",
			"C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA\\v12.5\\bin\\cublasLt64_*": "./"
		}
	}
}
```

</details>

4. Build in `Powershell`

```console
$env:CUDA_PATH = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.5"
$env:CudaToolkitDir = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.5"
bun run scripts/pre_build.js --build
```

## Gotchas

On Ubuntu you may need to copy some libraries for `ffmpeg_next` library

```console
sudo cp -rf /usr/include/x86_64-linux-gnu/libsw* /usr/include/
sudo cp -rf /usr/include/x86_64-linux-gnu/libav* /usr/include/
```

If the CPU failed to execute an instruction, then build with the following environment variable

```console
WHISPER_NO_AVX = "ON"
WHISPER_NO_AVX2 = "ON"
WHISPER_NO_FMA = "ON"
WHISPER_NO_F16C = "ON"
```

## Test

```
export RUST_LOG=trace
cargo test -- --nocapture
```

# Lint

```console
cargo fmt
cargo clippy
```

# Create new release

1. Increment verison in `tauri.conf.json` and commit
2. Create new git tag and push

```console
git tag -a v<version> -m "v<version>" && git push --tags
```

It will create releases for `Windows`, `Linux`, and `macOS`

Along with `latest.json` file (used for auto updater).

When `Release` action finish, it will run `Deploy landing` action

and update downloads links in landing page.

# Landing

## Compress images

```console
bunx tinypng-go static/*.png
```
