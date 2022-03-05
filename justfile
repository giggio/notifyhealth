alias b:= build
alias t:= test

amd64_target := "x86_64-unknown-linux-musl"
arm32v7_target := "armv7-unknown-linux-musleabihf"
win64_target := "x86_64-pc-windows-gnu"
binary := "notifyhealth"
base_image := "giggio/" + binary
version := `toml get Cargo.toml package.version | jq -r`

default:
    just --list

build:
    cargo build

test:
    cargo test

clean:
    cargo clean

run:
    cargo run

run_without_network:
    unshare -r -n -- cargo run

build_release:
    cargo build --release

build_linux_amd64:
    cross build --release --target {{ amd64_target }} --features openssl

docker_build_linux_amd64: (_docker_build_linux amd64_target "linux_amd64" "linux/amd64")
release_linux_amd64: build_linux_amd64 docker_build_linux_amd64 (_release "linux_amd64")

build_linux_armv7:
    cross build --release --target {{ arm32v7_target }} --features openssl

docker_build_linux_armv7: (_docker_build_linux arm32v7_target "linux_arm32v7" "linux/arm/v7")

_docker_build_linux target tag platform:
    mkdir -p target/output
    cp target/{{ target }}/release/{{ binary }} target/output/
    docker buildx build -t {{ base_image }}:{{ version }}-{{ tag }} -t {{ base_image }}:{{ tag }} --platform {{ platform }} .

release_linux_armv7: build_linux_armv7 docker_build_linux_armv7 (_release "linux_arm32v7")

build_linux: build_linux_amd64 build_linux_armv7

docker_build_linux: build_linux docker_build_linux_amd64 docker_build_linux_armv7

build_windows_amd64:
    cross build --release --target {{ win64_target }}

docker_build_windows_amd64:
    mkdir -p target/output
    cp target/{{ win64_target }}/release/{{ binary }}.exe target/output/
    docker buildx build -t {{ base_image }}:{{ version }}-windows_amd64 -t {{ base_image }}:windows_amd64 --platform windows/amd64 -f Dockerfile.windows .

release_windows_amd64: build_windows_amd64 docker_build_windows_amd64 (_release "windows_amd64")

_release tag:
    docker push {{ base_image }}:{{ version }}-{{ tag }}
    docker push {{ base_image }}:{{ tag }}

release_with_docker_only:
    DOCKER_CLI_EXPERIMENTAL=enabled docker manifest create {{ base_image }}:latest \
        --amend {{ base_image }}:linux_amd64 \
        --amend {{ base_image }}:windows_amd64 \
        --amend {{ base_image }}:linux_arm32v7
    DOCKER_CLI_EXPERIMENTAL=enabled docker manifest push {{ base_image }}:latest
    DOCKER_CLI_EXPERIMENTAL=enabled docker manifest create {{ base_image }}:{{ version }} \
        --amend {{ base_image }}:{{ version }}-linux_amd64 \
        --amend {{ base_image }}:{{ version }}-windows_amd64 \
        --amend {{ base_image }}:{{ version }}-linux_arm32v7
    DOCKER_CLI_EXPERIMENTAL=enabled docker manifest push {{ base_image }}:{{ version }}

release_linux: release_linux_amd64 release_linux_armv7

release_windows: release_windows_amd64

release_all: release_linux release_windows release_with_docker_only
