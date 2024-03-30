[![Build](https://img.shields.io/github/actions/workflow/status/serenity-rs/poise/ci.yml?branch=current)](https://serenity-rs.github.io/poise/)
[![crates.io](https://img.shields.io/crates/v/poise.svg)](https://crates.io/crates/poise)
[![Docs](https://img.shields.io/badge/docs-online-informational)](https://docs.rs/poise/)
[![Docs (git)](https://img.shields.io/badge/docs%20%28git%29-online-informational)](https://serenity-rs.github.io/poise/)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust: 1.74+](https://img.shields.io/badge/rust-1.74+-93450a)](https://blog.rust-lang.org/2023/11/16/Rust-1.74.0.html)
--- above is poise related---

# CV-with-Poise
This project is built with the event handler example. Built on windows... deal with it 😂

# How to use
## 1. BUILD
```cargo build --example event_handler```

## 2. RUN
for CMD use
```set DISCORD_TOKEN=tokenstring && target\debug\examples\event_handler.exe```

for Powershell use
```($env:DISCORD_TOKEN='tokenstring') -and (target\debug\examples\event_handler.exe)```
