TARGETS=hunter2
WEB_DIR=./web/
DEPLOY_HOST=cool-arnberger.webschuur.com
DOCKER_NETWORK=docker-bridge
DIR := ${CURDIR}
CURRENT_VERSION=$(shell git tag --list | sort --version-sort --reverse | head -n1 | cut -c2-100)
NEXT_VERSION=$(shell expr $(CURRENT_VERSION) + 1)

all:
	cargo build --release

deploy:
	for file in $(TARGETS); do scp ./target/release/$$file deploy@$(DEPLOY_HOST):/tmp/$$file && ssh deploy@$(DEPLOY_HOST) mv /tmp/$$file /usr/local/bin/$$file; done
	ssh deploy@$(DEPLOY_HOST) sudo systemctl restart hunter2.service
	rsync --recursive web/* deploy@$(DEPLOY_HOST):/u/apps/flockingbird_jobsearch/current/public/
	rsync job_tags.txt deploy@$(DEPLOY_HOST):/u/apps/hunter2/shared/job_tags.txt

meili:
	docker run --name hunter2_meili_dev --network $(DOCKER_NETWORK) --publish 7700:7700 --volume $(DIR)/dev.ms:/data.ms --rm --detach --env=MEILI_MASTER_KEY="" -d getmeili/meilisearch:v0.27.2
	docker run --name hunter2_meili_test --network $(DOCKER_NETWORK) --publish 7701:7700 --volume $(DIR)/test.ms:/data.ms --rm --detach --env=MEILI_MASTER_KEY="m3i7i3" -d getmeili/meilisearch:v0.27.2

release-tag:
	git tag --sign --annotate "v$(NEXT_VERSION)"
