# BTC-Clients
a simplified BTC client with full node functionality.
## BLOCKCHAIN struct

- functions related to the longest chain rule
  - new(): create a new blockchain that only contains the genesis block (hard coded)
  - insert(): insert a block into the blockchain
  - tip(): return the last block hash in the longest chain 
  - all_blocks_in_longest_chain(): return all blocks' hashes (genesis -> tip)
  - `new()`: create a new blockchain that only contains the genesis block (hard coded)
  - `insert()`: insert a block into the blockchain
  - `tip()`: return the last block hash in the longest chain 
  - `all_blocks_in_longest_chain()`: return all blocks' hashes (genesis -> tip)

## MINER
- the miner calls `blockchain.tip()` and set it as the parent of the block being mined.
- 
- After a block is generated, it will be insert into blockchain
- thread safe wrapper of blockchain

## MAIN MINING LOOP
- loop that try random nonces to solve the POW.
- `blockchain.tip()` to get the parent
- timestamp
- difficulty: static/constant difficulty
- nonce: increment nonce by one in every iteration

> `block.hash() <= difficulty` => the block is generated and then it can be inserted into blockchain

## NETWORK
- to communiate with other nodes/clients 
- forms the peer-to-peer network
- gossip protocol 

### MESSAGE TYPES
- `NewBlockHashes`: if the hashes are not in blockchain. Send `GetBlocks` to ask for hashes
- `GetBlocks`: if the hashes are in blockchain. Send `Blocks` to get blocks and send the hashes.
- `Blocks`: insert the blocks into blockchain if not ready in it.

## TRANSACTION 

### TRANSACTION NETWORK MESSAGES
- `NewTransactionHashes`
- `GetTransactions`
- `Transactions`

### TRANSACTION FORMAT: ACCOUNT BASED 
- account based model:
  - recipient address
  - a value
  - account nonce
- add `Signature` to transaction, append the public key and the signature to transaction by creating a struct `SignedTransaction` that contains the transaction, the public key, and the signature

### TRANSACTION MEMPOOL
- To store all the recieved valid transactions that are not included in the blockchain
- used by the miner to include transactions in the blocks being mined.
- the miner will add transactions in the mempool to the block till it reaches the block size limit.
- need the thread safe wrapper on the mempool
