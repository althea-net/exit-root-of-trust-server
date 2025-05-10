# Exit root of trust server

See the [Althea Whitepaper](https://blog.althea.net/content/files/2024/06/Althea-Whitepaper-v2.0.pdf) for more information about the Althea Protocol.

The software in this repo has two primary functions

- Facilicate interacting with the Althea client and exit database [smart contract](https://github.com/althea-net/rita/blob/master/solidity/contracts/AltheaDB.sol). Providing an easy UI for clients and exits to be registered in the database
- Provide a long lived signing authority (analogus to a certificate authority in SSL) to allow a client to bootstrap connectivity so that the client can read the smart contract themselves.

## Exit Database Smart Contract

This smart contract acts as a way for Exits and Clients in the Althea protocol to exchange keys securely, without fear of a man in the middle attack. In order to do this a trusted root authroity is used (the Smart Contract) the signing server simply provides a signed copy of the smart contract data to clients in an offline format, allowing them to bootstrap to reading the database themselves. 

The database contains a list of all registered clients, and all registered exits. An exit is a location where traffic in an Althea network is exited out to the wider internet and a point of trust since clients may send unencrypted traffic to and from the larger internet.

For more context read the whitepaper linked above.

## Future

In theory the signing function of this application may become obsolte once clients are able to process a light client proof from the Althea L1 blockchain themselves, at which point a proof of the smart contract data could be constructed from Althea L1 consensus and presented to the client, rather than requiring another trusted participant in the system simply to faciliate bootstrapping.

## Software structure 

This repo merely hosts the bin part of the Rust software and the frontend repository for performing these functions. The majority of the actual logic and testing is in the [Rita](https://github.com/althea-net/rita) repo. Where Rita is the routing and billing daemon that implements most of the Althea protocol's functions.
