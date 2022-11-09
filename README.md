# cron-parser

This is program to parse and validate the cron expressions

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone git@github.com:iho/cron-parser.git
cargo test
cargo run "* * * * * /usr/bin/find"
```