# Database

A simple distributed database built with Rust without using any external libraries. It aims to be extremely simple and readable.
It's not about getting the maximum performance but instead focusing on simplicity. It can be used as a template so you can
customise it and add features as you like by forking it.

## Design

The design is based on RocksDB/LevelDB. It uses multi-paxos for consensus. Data is partitioned amongst multiple
groups of nodes. To get the data a gossiping protocol is used that's using an in-memory table as a cache on each node.

- [x] Storage - SSTables implementation
- [ ] Leveled compactions
- [ ] Server - Listens for and responds to requests
- [ ] Multi-Paxos - Consensus protocol
- [ ] Gossiping protocol
- [ ] Bloom filters
