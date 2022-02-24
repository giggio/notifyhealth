alias b:= build
alias t:= test

amd64_target := "x86_64-unknown-linux-musl"
arm32v7_target := "armv7-unknown-linux-musleabihf"
binary := "notifyhealth"
base_image := "giggio/" + binary

default:
    just --list

echo:
    echo oi
    echo {{ amd64_target }}
    echo {{ arm32v7_target }}
    echo {{ binary }}
    echo {{ base_image }}

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

build_amd64_static:
	cross build --release --target {{ amd64_target }}

docker_build_amd64_static:
	mkdir -p target/output
	cp target/{{ amd64_target }}/release/{{ binary }} target/output/
	VERSION=$(./target/output/{{ binary }} --version | cut -f2 -d ' '); \
	docker buildx build -t {{ base_image }}:$VERSION-amd64 -t {{ base_image }}:amd64 --platform linux/amd64 --build-arg PLATFORM=x86_64 --push .

release_amd64_static: build_amd64_static docker_build_amd64_static

build_armv7_static:
	cross build --release --target {{ arm32v7_target }}

docker_build_armv7_static:
	mkdir -p target/output
	cp target/{{ arm32v7_target }}/release/{{ binary }} target/output/
	VERSION=$(./target/output/{{ binary }} --version | cut -f2 -d ' '); \
	docker buildx build -t {{ base_image }}:$VERSION-arm32v7 -t {{ base_image }}:arm32v7 --platform linux/arm/v7 --build-arg PLATFORM=armhf --push .

release_armv7_static: build_armv7_static docker_build_armv7_static

release_with_docker_only: docker_build_amd64_static docker_build_armv7_static
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest create {{ base_image }}:latest \
		--amend {{ base_image }}:amd64 \
		--amend {{ base_image }}:arm32v7
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest push {{ base_image }}:latest
	VERSION=$(./target/{{ amd64_target }}/release/{{ binary }} --version | cut -f2 -d ' '); \
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest create {{ base_image }}:$VERSION \
		--amend {{ base_image }}:$VERSION-amd64 \
		--amend {{ base_image }}:$VERSION-arm32v7; \
	DOCKER_CLI_EXPERIMENTAL=enabled docker manifest push {{ base_image }}:$VERSION

release: release_amd64_static release_armv7_static release_with_docker_only
