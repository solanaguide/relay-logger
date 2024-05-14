# Logging Fork Of Jito Relayer
Jito Relayer acts as a transaction processing unit (TPU) proxy for Solana validators. This fork has been modified to log all tx received by the relayer to a rolling set of CSV files, allowing futher analysis.

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
// let compute_unit_price = compute_unit_price / 1_000_000u64;
    let priority_fee = (compute_unit_limit as u64 * compute_unit_price) / 1_000_000u64;
    let transaction_fee = priority_fee + (num_signatures * LAMPORTS_PER_SIGNATURE);
    let compute_fee_ratio = transaction_fee as f64 / (1f64 + compute_unit_limit as f64);

```

By default, logs will be stored under ./txlogs/txlog.csv in the following format
    
```csv
timestamp,signature,compute_unit_limit,transaction_fee,compute_fee_ratio,ip_address
```

Logs should be rotated when >1gb. This may happen fast if you allow all traffic; ensure you are monitoring your HDD space and prune is working correctly.



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
