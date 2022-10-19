#!/bin/bash
set -e

KEY_UID="e58f9bac-77e7-473c-a981-1a317edfac6e"

curl "${MEILI_URI}/keys" --silent -H "Authorization: Bearer ${MEILI_MASTER_KEY}" -H 'Content-Type: application/json' --data "{\"uid\": \"${KEY_UID}\", \"name\": \"Client Key\", \"description\": \"Client key with RO access to search and stats\", \"actions\": [\"search\", \"stats.get\"], \"indexes\": [\"vacancies\"], \"expiresAt\": null}" > /dev/null

curl "${MEILI_URI}/keys/${KEY_UID}" --silent -H "Authorization: Bearer ${MEILI_MASTER_KEY}" -H 'Content-Type: application/json' | jq -r '.key'
