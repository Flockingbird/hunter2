# hunter2
Hunter2 is a job hunt bot that indexes jobs and candidates from the fediverse

## Architecture

Hunter2 is written in Rust. Using message passing between threads.

The [editable sequence diagram](http://www.plantuml.com/plantuml/png/dP2nJiGm38RtF4N7ThXxWAggRc1XO0716bc9uI8rRaYSYhuzeOKJraK8cAp-zVV9-K-98NBsamfbEkC243SU73MGjgd47vhPFJi3x6PAciyHmPRDQYx072mmVaTaejHhneoD3ycKlzkKDs5Ay28CCYeAjHr-67TBtzpj2tt_10R8h6s0n7hPFxgZghwcuDMyRomoHky8jIEBRnxRnC0vTLLHxOxRVtBt6ARrPYP-B89OQB8NXv95_lJy2oAQ4mp7LKSqIzjTBGs7hwNXqpmyRC9nh0fbLB0pa9rGB_AQFx6o-2COCsVYNphYHulQpiGwN_TuGhBPJ-y0) shows the threads and their main function (details and
internal plumbing omitted).

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
```
This modifies the ranking and adds a sortable attribute.
