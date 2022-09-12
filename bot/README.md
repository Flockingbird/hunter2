# hunter2
Hunter2 is a job hunt bot that indexes jobs and candidates from the fediverse

## Architecture

Hunter2 is written in Rust. Using message passing between threads.

The [editable sequence diagram](https://www.plantuml.com/plantuml/uml/VOvFIiKm4CRtSueiNnVF0uHItWLS2Mx3a8msqAGnCodqzfPKqSJYQfZlpyVlmO9PIbWW7TTRdp2A2kYZGguN6YUkuj-yHV5hP2Dp9dH7yb9lYYKv5FfLwHI0ZqA5L4Xi3xVUbecOyRrPw2I0odsVpX6jdazVwrUq6Er-IyXYjlCPdg1Z-gV8Wb9u0EWfVZgITvF9RhVXJsYyVuV6h-NnfGvEq-LW9sarOtGBcQsZUL1q9IoV) shows the threads and their main function (details and internal plumbing omitted).

![Plantuml Sequence Diagram](/doc/sequence_diagram.png)

## Storage for search in Meilisearch.

Meilisearch is an indexed search engine, similar to Elasticsearch. But much
simpler to host and set-up.

TODO: write about how to set up and deploy meilisearch.

Once deployed, you must tune the database a little. So that we can order by
date and that date is taken into consideration when searching.

If your meilisearch runs on search.example.com (TODO: explain authentication for production version)
```
curl \
  -X POST 'http://search.example.com/indexes/vacancies/settings/ranking-rules' \
  -H 'Content-Type: application/json' \
  --data-binary '[
    "words",
    "typo",
    "proximity",
    "attribute",
    "sort",
    "exactness",
    "created_at_ts:desc",
    "rank:desc"
  ]

curl \
  -X POST 'http://search.example.com/indexes/vacancies/settings/sortable-attributes' \
  -H 'Content-Type: application/json' \
  --data-binary '["created_at_ts"]'

curl \
  -X POST 'http://search.example.com/indexes/vacancies/settings/filterable-attributes' \
  -H 'Content-Type: application/json' \
  --data-binary '["tags", "language", "url"]'
```
This modifies the ranking, adds a sortable attribute and adds the required filters for faceted search etc?
