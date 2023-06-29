This repository contains code used to estimate how much space storing all ethereum events takes.

The `collect` crate samples the events of random blocks. For each block it collects the number of logs, number of topics, total dynamic log data. This data is stored in binary form in the file `out`. Data collection can be interrupted and resumed at any time. Running it for longer produces more accurate data in the following step because we have more random samples.

The `analyze` crate reads the file produced in the previous step. It extrapolates the data from one random sample to the next. This yields an estimate for the total amount of data needed to store all events.

`collect` requires the environment variable `NODE_URL` to be set. It fetches events from this node. Otherwise there are no external dependencies.

```
env NODE_URL=... cargo run --bin collect --release
cargo run --bin analyze
```
