# grainpit

[![Crates.io Version](https://img.shields.io/crates/v/grainpit)](https://crates.io/crates/grainpit)
[![Crates.io Downloads](https://img.shields.io/crates/d/grainpit)](https://crates.io/crates/grainpit)

markov tarpit but grain

> [!IMPORTANT]
> grainpit is under heavy development currently! please keep in mind that right now if you want stability you should use the `webserver` version!

![image of the tarpit](example.png)

## backends

grainpit itself is just a markov library and some extra utilities, there are 3 different backends you can use to actually add functionality

### `webserver`

[![Crates.io Version](https://img.shields.io/crates/v/grainpit_webserver)](https://crates.io/crates/grainpit_webserver)
[![Crates.io Downloads](https://img.shields.io/crates/d/grainpit_webserver)](https://crates.io/crates/grainpit_webserver)

this only exposes a webserver which you have to setup rules on your reverse proxy to actually get to, currently this is the most stable

you can enable content compression for this proxy with the `compress` feature incase network load is a concern for you

[grainpit_webserver installation and configuration instructions](./docs/webserver.md)

### `proxy` (in development)

this is under heavy development and is more complex, but proxies the traffic through grainpit which allows injecting hidden links to the tarpit and more configuration on grainpits side

this is planned to be the preferred way to use grainpit once its completed

currently this is empty in the repo because the general project structure for it is still being figured out

### cgi script

> [!NOTE]
> this currently does not support the full feature set and may be out of date as it does not directly depend on the main grainpit library. you are better off using one of the other backends unless you absolutely need this version
> compilation also takes like 8G of ram because the generate.c implementation is horrible

written in c unlike the rest of the project (also means compatibility if you are on a machine without support for rust), thank you to @doclic for writing this implementation

[grainpit.cgi compilation instructions](./grainpit.cgi/README)

## features

- fast and lightweight, on my low resource proxy vps (2 cores, e5-2680) it manages to hover around ~800µs per request with 13mb ram usage while handling ~30 requests per second continuously to the tarpit without consuming enough cpu time where regular requests slow down substantially
- batshit insane default training data that (somehow) manages to put out html/css
- able to generate fake config files as well for the malicious credential scanning bots
- effective, as of writing this GPTBot has been sending requests to the tarpit since july 13 as well as amazons scraper and claudes scraper more recently

## reasons you might want to use a different tarpit

- not as many features as other alternatives have
- requires reverse proxy config to send bots here (a proxy is currently in progress so you dont need this)

## development

> [!WARNING]
> this section is a work in progress

### getting started

make sure you have rust installed with rustup already and this repo cloned

once you have those installed you can develop grainpit by running `cargo run --bin <binary>` (e.g. grainpit_webserver for the webserver version)

> [!TIP]
> for development the default `dev` profile has been configured to be almost as optimized as the release profile (and more optimized than the default rust release profile)

### benchmarks

if you are contributing changes that may affect performance its appreciated (but not required) if you can include a benchmark comparison of before and after by doing this:

```bash
git checkout main
cargo bench -p benchmarks -- --save-baseline main
git checkout <your branch>
cargo bench -p benchmarks -- --baseline main
```
