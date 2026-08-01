# grainpit

![Crates.io Version](https://img.shields.io/crates/v/grainpit)

markov tarpit but grain

> [!IMPORTANT]
> grainpit is under heavy development currently! please keep in mind that right now if you want stability you should use the `webserver` version!

![image of the tarpit](example.png)

## backends

grainpit itself is just a markov library and some extra utilities, there are 3 different backends you can use to actually add functionality

### `webserver`

this only exposes a webserver which you have to setup rules on your reverse proxy to actually get to, currently this is the most stable

you can enable content compression for this proxy with the `compress` feature incase network load is a concern for you

### `proxy` (in development)

this is under heavy development and is more complex, but proxies the traffic through grainpit which allows injecting hidden links to the tarpit and more configuration on grainpits side

this is planned to be the preferred way to use grainpit once its completed

currently this is empty in the repo because the general project structure for it is still being figured out

### cgi script (unofficial)

see <https://codeberg.org/doclic/grainpit.cgi/>

## features

- fast and lightweight, on my low resource proxy vps (2 cores, e5-2680) it manages to hover around ~800µs per request with 13mb ram usage while handling ~30 requests per second continuously to the tarpit without consuming enough cpu time where regular requests slow down substantially
- batshit insane default training data that (somehow) manages to put out html/css
- able to generate fake config files as well for the malicious credential scanning bots
- effective, as of writing this GPTBot has been sending requests to the tarpit since july 13 as well as amazons scraper and claudes scraper more recently

## reasons you might want to use a different tarpit

- not as many features as other alternatives have
- requires reverse proxy config to send bots here (a proxy is currently in progress so you dont need this)

## configuration

if you need to change the ip/port that grainpit binds to you can use the GRAINPIT_ADDR variable, the default is `127.0.0.1:5000`

you can add extra domains/subdomains that you own to drop in occasionally as links if you have the bandwidth, this may boost how good your site looks to scrapers and may also make them scrape faster

in order to add these there are a few variables you can configure

- `GRAINPIT_EXTRAURLS`: comma separated list like `https://example.com/,https://otherexample.com/`
- `GRAINPIT_EXTRAURLS_CHANCE`: chance to make a link start with this in percentage (default is 5%)

## installation

### systemd/bare metal

install this using `cargo install grainpit_webserver`

example systemd service:

```ini
[Unit]
Description=Tarpit
After=network.target

[Service]
ExecStart=/root/.cargo/bin/grainpit_webserver
Restart=always

[Install]
WantedBy=default.target
RequiredBy=network.target
```

### docker/podman

theres a docker compose for this that you can use, run `docker compose up -d --build` and grainpit will start running at 127.0.0.1:5000

## reverse proxy setup

### haproxy

im using haproxy as my reverse proxy, you can probably do similar with other reverse proxies, here is an example based on my haproxy config:

```haproxy
# ...
backend per_ip_and_url_rates
    stick-table type binary len 8 size 1m expire 24h store http_req_rate(24h)

backend per_ip_rates
    stick-table type ip size 1m expire 24h store gpc0,gpc0_rate(5s),http_err_rate(5m),gpt0

frontend http
    # ...

    http-request track-sc0 src table per_ip_rates
    http-request track-sc1 url32+src table per_ip_and_url_rates unless { path_end .css .js .png .jpeg .gif .avif .webp .svg .ico }
    acl exceeds_limit sc_gpc0_rate(0) gt 50
    acl exceeds_limit sc_http_err_rate(0) gt 10
    acl exceeds_limit path -i -m beg /wp-admin
    acl exceeds_limit path -i -m beg /.env
    acl exceeds_limit path -i -m beg /.git
    acl exceeds_limit path -i -m beg /wp-login.php
    http-request sc-inc-gpc0(0) if { sc_http_req_rate(1) eq 1 } !exceeds_limit
    http-request sc-set-gpt0(0) 1 if exceeds_limit
    use_backend evil_bot_punishment_zone if { sc_get_gpt0(0) eq 1 }

    default_backend web_servers

backend web_servers
    # ...

backend evil_bot_punishment_zone
    balance first
    timeout queue 5s
    server tarpit localhost:5000 check
```

i also recommend combining this with <https://github.com/ai-robots-txt/ai.robots.txt/> to provide better matching

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
