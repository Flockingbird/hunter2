DOCKER_NETWORK=docker-bridge
DIR := ${CURDIR}
CURRENT_VERSION=$(shell git tag --list | sort --version-sort --reverse | head -n1 | cut -c2-100)
NEXT_VERSION=$(shell expr $(CURRENT_VERSION) + 1)

all:
	$(MAKE) -C ./bot

test:
	export MEILI_ADMIN_KEY=$(shell zenv --file .env.test -- ./admin_key.sh)
	zenv --file .env.test -- $(MAKE) -C ./bot test

meili:
	docker run --name hunter2_meili_dev --network $(DOCKER_NETWORK) --publish 7700:7700 --volume $(DIR)/dev.ms:/data.ms --rm --detach --env=MEILI_MASTER_KEY="slagboomwicketsovjets7_" -d getmeili/meilisearch:v0.29.0
	docker run --name hunter2_meili_test --network $(DOCKER_NETWORK) --publish 7701:7700 --volume $(DIR)/test.ms:/data.ms --rm --detach --env=MEILI_MASTER_KEY="m3i7i3" -d getmeili/meilisearch:v0.29.0

release-tag:
	git tag --sign --annotate "v$(NEXT_VERSION)"
