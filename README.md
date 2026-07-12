# grainpit

markov tarpit but grain

might also be able to mess with the bots that spam requests for exposed .envs because if you go to a page that isnt directly linked to in one of the tarpit pages it returns a fake config file

keep in mind that youll have to figure out getting the user here on your own, because i use haproxy i have a rule to automatically force users who make too
many requests to new urls in a short period of time or who request obviously botted urls here for 24h

## setup

grainpit is designed to be

- fast
- lightweight
- effective

the server i am running this on has 2gb ram and 2 cores and most of those resources are used by varnish and haproxy, so this should be able to run on almost anything, if you experience slowdowns you should implement rate limits on your reverse proxies end

if you need to change the ip/port that grainpit binds to you can use the TARPIT_ADDR variable, the default is `127.0.0.1:5000`

### systemd/bare metal

install this using `cargo install --path .`

example systemd service:

```ini
[Unit]
Description=Tarpit
After=network.target

[Service]
ExecStart=/root/.cargo/bin/tarpit
Restart=always

[Install]
WantedBy=default.target
RequiredBy=network.target
```

### docker/podman

theres a docker compose for this that you can use, run `docker compose up -d --build` and tarpit will start running at 127.0.0.1:5000

### haproxy example

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
