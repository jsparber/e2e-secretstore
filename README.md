# Crypto-module
This is the reference implementation created for the master thesis "Blockchain-based end-to-end encryption for Matrix instant messaging" written by Julian Sparber.
The thesis proposes an end-to-end encryption system for the realtime communication network Matrix, which uses the [Secret Store](https://openethereum.github.io/wiki/Secret-Store) feature of the [OpenEhtereum](https://openethereum.github.io/) Ethereum client.
## Setup
To use this library an Ethereum network needs to be created. The scripts `network/start.sh` and `network/start_ssh.sh` can be used to automatically create a local network or a network on a set of remote computers.

You will need to build OpenEthereum yourself because the binary releases do not enable the SecretStore feature.

Currently, all files have hardcoded values, therefore please check before running any script that they contain correct usernames, paths, server addresses, etc.
### Secret Store
#### Local Network
Run the command `./start.sh -s [NUMBER_OF_NODES]` to create a network with a specific number of OpenEhtereum instances on the local machine. After deploying a smart contract the command `./start.sh -c [CONTRACT_ADDRESS]` can be used to set the permissioning contract for the Secret Store.
#### Remote Network
Run the command `./start_ssh.sh -s [NUMBER_OF_NODES]` to create a network with a specific number of OpenEhtereum instances on remote machines found in the file `ercolani.txt` and runs them all on the remote computers, except one node that is run on a local machine. The instances can be terminated by pressing `ctrl+c` once, and also the fallback script `stop.sh` can be used to make sure that all nodes are completely terminated. After deploying a smart contract to the network, the network can be stopped and then started again with the command `./start_ssh.sh -c [CONTRACT_ADDRESS]` to set the permissioning contract for the Secret Store.
The crypto-module automatically deploys the smart contract `SSPermissions.sol` if the file `
contract-address.txt ` doesn't exist at the root of the project and it stores the address of the deployed contract in this file. 

It may be needed to set the correct port forwarding at the router to make the local Ethereum node discoverable by the remote nodes.

### User's node
Additionally to the network setup, a node with a user account has to be created (the config file `users.toml` can be used). This can be done by running `parity --config users.toml account new`

Before any test can be run, they need to be updated with the correct account address and password and, eventually, also the location for the Secret Store node and the user's node needs to be changed.

To test the setup the command `cargo test setup1 -- --test-threads=1` can be used,
which does only check if the library can correctly access the Secret Store and the user's node and deploys automatically the smart contract if the file `contract-address.txt ` is empty.

## Tests
To run all tests just execute `cargo test -- --test-threads=1`. The source for tests can be found in the `src/lib.rs` file.
 
