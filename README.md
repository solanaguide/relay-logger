# Logging Fork Of Jito Relayer
Relay Logger acts as a transaction processing unit (TPU) proxy for Solana validators. This fork has been modified to log all tx received by the relayer to a rolling set of CSV files, allowing futher analysis.

Assuming your relayer is already processing all transactions, and your TPU ports are closed, you can use this relayer as a drop in replacement for Jito's in order to log locally, without needing any additional patches or modifications to your validator source.

# Additional Installation Steps

1. Build as you would normally, as shown below
2. Ensure the log4rs.yml file is in the same directory as the binary
3. Ensure you set WorkingDir= to the folder containing this yml in your systemd service file and use `daemon-reload` to reload the service before restarting

# What does it do?

For every packet received, we analyze the transaction to extract:
- The transaction signature
- The requested compute limit (or the default 200_000 if not set)
- The requested compute price, if any
- num_signatures_required
- The IP address

These are then used to calculate a fee ratio, which is roughly `total_fee/total_cu_requested`

This is handled by the following code in `core/src/txlog.rs`

```rust
    let priority_fee = (compute_unit_limit as u64 * compute_unit_price) / 1_000_000u64;
    let transaction_fee = priority_fee + (num_signatures * LAMPORTS_PER_SIGNATURE);
    let compute_fee_ratio = transaction_fee as f64 / (1f64 + compute_unit_limit as f64);

```

By default, logs will be stored under ./txlogs/txlog.csv in the following format
    
```csv
timestamp,signature,compute_unit_limit,transaction_fee,compute_fee_ratio,ip_address
```

Log behaviour can be altered by editing log4rs.yml and restarting your service. By default it stores up to 1GB per log file and retains up to 10 logs. On my setup I am currently storing 50MB per file and up to 200 files, this allows me to post my logs to ElasticSearch each minute using Cron, so my db is as up to date as possible whilst not touching the live txlog.csv file that is still being written.


# What to do with the data?

It's up to you - more experimentation on this would be great. Personally, I feel we should be optimizing not just for high rewards but also the highest number of non-vote transactions per block, to ensure we are landing as many transactions as possible for users. My current setup (tho I change every couple of days) is:

Every minute, run a cron job to:
1. Parse each log file in ./txlogs
2. Post the data to an ElasticSearch Cloud instance

Then, on ElasticSearch I've built dashboards that show me reports such as
- Potential Fees Per Minute vs Avg Fee ratio - good to see how spiky demand can be
- Top IPs By Request - and their median fee ratio, to compare good vs spammy low fee only IPs
- Amount of duplication per IP / C-Class IP
- IPs per signature (not doing anything with this yet)
- 'Big Spender' IPs I want to ensure I allow in before any filters

At the moment I'm  building two lists:
- The worst behaviour - mostly high reqests, tiny fee_ratios, but also 1 or 2 IPs that are just excessively high TX counts
- The best behaved IPs - people I want to ensure don't get lost by any other filter issues such as rate limiting etc
- (at one point) Bad C-Class - however this included a lot of gossip IPs, and after implementing I saw a marked decrease in the number of non vote transactions I was processing

These are then paired with the tpu-traffic-classifier rulesets, so that I allow staked connections and 'best behaved' IPs, then block bad behaviour before allowing in both gossip and non-gossip IPs with a tight per-IP RPS limit.

Separately, I am also using a websocket connection to listen to blocks as they are created + update my ElasticSearch index to mark signatures as 'landed' when they are packed into a block. The data is a little unreliable so I'm not yet doing anything with this yet.

I'm currently creating ipsets manually from this data, however once I am happy with the filters I'm using I plan to automate the generation of IP sets:
- Build a query in ElasticSearch/Kibana
- Query it from my server, and apply any additional filtering (eg Top IPs => Top IPs where fee_ratio < ....)
- Create/update ipset

I'm still not sure the best way to maintain lists of blocked IPs over time, and when a ban could expire. I'm considering simply disabling all ipset rules for a random minute or two each hour. Would love to discuss ideas

# What does it look like?

I've setup a Kibana dashboard that shows live connections to my node. Note this is *post* firewall so many of the IPs I've deemed bad will not be appearing, but hopefully its helpful to see the kinds of queries I'm looking at

https://my-deployment-57a9de.kb.europe-west3.gcp.cloud.es.io:9243/s/public-space/app/dashboards#/view/ca870b11-c3e2-4179-a779-269b57ddd49e?_g=(filters%3A!()%2CrefreshInterval%3A(pause%3A!t%2Cvalue%3A0)%2Ctime%3A(from%3Anow-1h%2Cto%3Anow))

I can't make it public, so you can log in using:

Elasticsearch User: demo
Password: solana

Note too that it is a pain to update due to weird Kibana permissions, so over time it may not have everything i'm considering when analyzing traffic, but hopefully there's enough there to stimulate some ideas + discussion

Also in most reports I use Median not Mean for averages; this is very slow so the dashboard uses Mean instead.

# Building
```shell
# pull submodules to get protobuffers required to connect to Block Engine and validator
$ git submodule update -i -r
# build from source
$ cargo b --release
```

# Releases

## Making a release

We opt to use cargo workspaces for making releases.
First, install cargo workspaces by running: `cargo install cargo-workspaces`.
Next, check out the master branch of the jito-relayer repo and 
ensure you're on the latest commit.
In the master branch, run the following command and follow the instructions:
```shell
$ ./release
```
This will bump all the versions of the packages in your repo, 
push to master and tag a new commit.

## Running a release
There are two options for running the relayer from releases:
- Download the most recent release on the [releases](https://github.com/jito-foundation/jito-relayer/releases) page.
- (Not recommended for production): One can download and run Docker containers from the Docker [registry](https://hub.docker.com/r/jitolabs/jito-transaction-relayer).

# Running a Relayer
See https://jito-foundation.gitbook.io/mev/jito-relayer/running-a-relayer for setup and usage instructions.
