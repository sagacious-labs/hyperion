.PHONY: run
run: compile
	RUST_LOG=debug sudo -E ./target/debug/hyperion

.PHONY: compile
compile:
	cargo build $(CFLAGS)

.PHONY: docker
docker:
	@test -n "$(VERSION)" || (echo "VERSION is a required variable for target \"docker\"" ; exit 1)
	DOCKER_BUILDKIT=1 docker build . -t utkarsh23/hyperion:v$(VERSION)

