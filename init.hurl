GET localhost:7700/keys
Authorization: Bearer {{meili_master_key}}

HTTP/1.1 200
[Captures]
admin_key: jsonpath "$..results[?(@.name == 'Default Admin API Key')].key"

POST localhost:7700/indexes/
Authorization: Bearer {{meili_master_key}}
{
  "primaryKey": "id",
  "uid": "vacancies"
}

HTTP/1.1 202

PUT localhost:7700/indexes/vacancies/settings/ranking-rules
Authorization: Bearer {{meili_master_key}}
[
  "words",
  "typo",
  "proximity",
  "attribute",
  "sort",
  "exactness",
  "created_at_ts:desc",
  "rank:desc"
]

HTTP/1.1 202

PUT localhost:7700/indexes/vacancies/settings/sortable-attributes
Authorization: Bearer {{meili_master_key}}
[
  "created_at_ts"
]

HTTP/1.1 202

PUT http://localhost:7700/indexes/vacancies/settings/filterable-attributes
Authorization: Bearer {{meili_master_key}}
[
  "tags",
  "language",
  "url"
]

HTTP/1.1 202
