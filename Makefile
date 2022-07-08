TARGETS=hunter2
WEB_DIR=./web/
DEPLOY_HOST=cool-arnberger.webschuur.com
DOCKER_NETWORK=docker-bridge
DIR := ${CURDIR}

all:
	cargo build --release

deploy:
	for file in $(TARGETS); do scp ./target/release/$$file deploy@$(DEPLOY_HOST):/tmp/$$file && ssh deploy@$(DEPLOY_HOST) mv /tmp/$$file /usr/local/bin/$$file; done
	ssh deploy@$(DEPLOY_HOST) sudo systemctl restart hunter2.service
	rsync --recursive web/* deploy@$(DEPLOY_HOST):/u/apps/flockingbird_jobsearch/current/public/
	rsync job_tags.txt deploy@$(DEPLOY_HOST):/u/apps/hunter2/shared/job_tags.txt

meili:
	docker run --name hunter2_meili --network $(DOCKER_NETWORK) --publish 7700:7700 --volume $(DIR)/data.ms:/data.ms --rm --detach -d getmeili/meilisearch:latest
