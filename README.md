# hunter2
Hunter2 is a job hunt bot that indexes jobs and candidates from the fediverse

## Architecture

Hunter2 is written in Rust. Using message passing between threads.

The [editable sequence diagram](http://www.plantuml.com/plantuml/png/dP2nJiGm38RtF4N7ThXxWAggRc1XO0716bc9uI8rRaYSYhuzeOKJraK8cAp-zVV9-K-98NBsamfbEkC243SU73MGjgd47vhPFJi3x6PAciyHmPRDQYx072mmVaTaejHhneoD3ycKlzkKDs5Ay28CCYeAjHr-67TBtzpj2tt_10R8h6s0n7hPFxgZghwcuDMyRomoHky8jIEBRnxRnC0vTLLHxOxRVtBt6ARrPYP-B89OQB8NXv95_lJy2oAQ4mp7LKSqIzjTBGs7hwNXqpmyRC9nh0fbLB0pa9rGB_AQFx6o-2COCsVYNphYHulQpiGwN_TuGhBPJ-y0) shows the threads and their main function (details and
internal plumbing omitted).

![Plantuml Sequence Diagram](/doc/sequence_diagram.png)


