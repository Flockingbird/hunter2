#!/bin/bash
set -e

KEY_UID="37231c0f-bd61-44b4-bfc8-0c4e4eb8815e"

curl "${MEILI_URI}/keys" --silent -H "Authorization: Bearer ${MEILI_MASTER_KEY}" -H 'Content-Type: application/json' --data "{\"uid\": \"${KEY_UID}\", \"name\": \"Admin Key\", \"description\": \"Custom Admin key\", \"actions\": [\"*\"], \"indexes\": [\"*\"], \"expiresAt\": null}" > /dev/null

curl "${MEILI_URI}/keys/${KEY_UID}" --silent -H "Authorization: Bearer ${MEILI_MASTER_KEY}" -H 'Content-Type: application/json' | jq -r '.key'
