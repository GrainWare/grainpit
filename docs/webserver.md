# `grainpit_webserver`

## installation

### cargo

install this using `cargo install grainpit_webserver`, if you want to build from git use `cargo install --git https://github.com/GrainWare/grainpit/ grainpit_webserver`

<details>
<summary>example systemd service</summary>

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

</details>

### docker/podman

theres a docker compose for this that you can use, run `docker compose up -d --build` and grainpit will start running at 127.0.0.1:5000

## configuration

if you need to change the ip/port that grainpit binds to you can use the GRAINPIT_ADDR variable, the default is `127.0.0.1:5000`

you can add extra domains/subdomains that you own to drop in occasionally as links if you have the bandwidth, this may boost how good your site looks to scrapers and may also make them scrape faster

in order to add these there are a few variables you can configure

- `GRAINPIT_EXTRAURLS`: comma separated list like `https://example.com/,https://otherexample.com/`
- `GRAINPIT_EXTRAURLS_CHANCE`: chance to make a link start with this in percentage (default is 5%)

## reverse proxy setup

if you setup grainpit for any other reverse proxies (especially nginx and caddy) contribute documentation it would be really really appreciated thanks

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
