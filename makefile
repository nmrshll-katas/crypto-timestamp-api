.PHONY: all $(MAKECMDGOALS)
.DEFAULT_GOAL=dev
dev: deps pg adminer migrate_dev
	${pg_dsn} cargo +$v run
dockerized: deps build pg adminer migrate_docker

build:
	docker build -f deploy/api.Dockerfile -t ${cwd} . 
test: down deps pg migrate_dev
	${pg_dsn} cargo +$v test -- --nocapture

# MANUAL TEST REQUESTS
/:
	curl ${addr}/
pubkey:
	curl ${addr}/pubkey
sign_data:
	curl ${post} ${addr}/sign_data -i -d '{"data_base64":"hello world","pow_proof_base64":"YoUrPrOoF"}'
addr=http://0.0.0.0:8080
post= -X POST -H "Content-Type: application/json"

# PROCESSES
api:
	$(eval srvc=pg) ${docker_run} -d -p 0.0.0.0:8080:8080 hello-world
pg:
	$(eval srvc=pg) ${docker_run} -p 127.0.0.1:5432:5432 -e POSTGRES_PASSWORD=docker -e POSTGRES_USER=docker -e POSTGRES_DB=docker -d postgres:alpine
adminer: 
	$(eval srvc=adminer) ${docker_run} -d -p 127.0.0.1:7897:8080 adminer:4.2.5
migrate_dev: 
	@$(eval SHELL:=/bin/bash) while ! test "`echo -ne "\x00\x00\x00\x17\x00\x03\x00\x00user\x00username\x00\x00" | nc -w 3 127.0.0.1 5432 2>/dev/null | head -c1`" = R; do echo "waiting on postgres..."; sleep 0.3; done;
	${pg_dsn} diesel migration run
migrate_docker:
	docker run --rm \
    -w /workdir -v $(shell pwd)/migrations:/workdir/migrations \
    -e DATABASE_URL="postgres://docker:docker@host.docker.internal/docker" \
    -it clux/diesel-cli diesel migration run
down:
	-docker rm -f -v `docker ps -a -q --filter "name=${cwd}"`
logs: 
	$(eval srvc=api) docker logs -f ${container_name}
cwd = $(notdir $(shell pwd))
container_name = ${cwd}-${svc}
docker_run=@docker container inspect ${container_name} > /dev/null 2>&1 || docker run --rm --name ${container_name} -v $(shell pwd)/.config:/config:ro
pg_dsn=DATABASE_URL=postgres://docker:docker@127.0.0.1/docker

# DEPS
deps: installs
	@rustc --version | grep -E 'nightly.*2020-09-25' $s || rustup override set $v
	@diesel --version $s || cargo install diesel_cli --version 1.4.1 --no-default-features --features postgres
installs: 		# install manually: docker, build-essential, pkg-config
	@rustup --version $s || curl https://sh.rustup.rs -sSf | sh -s -- -y; export PATH="/root/.cargo/bin:$${PATH}"
s = &>/dev/null
v = nightly-2020-09-26