# BTC-Clients
a simplified BTC client with full node functionality.

## BLOCK struct

- Header
  - parent: a hash to parent block 
  - nonce: a random integer that will be used in POW mining
  - diffuculty: the threshold in POW check
  - timestamp: to decide the delay of a block
  - merkle root: the Merkle root of data

- Content
  - transactions carried in the block

## BLOCKCHAIN struct

- functions related to the longest chain rule
  - new(): create a new blockchain that only contains the genesis block (hard coded)
  - insert(): insert a block into the blockchain
  - tip(): return the last block hash in the longest chain 
  - all_blocks_in_longest_chain(): return all blocks' hashes (genesis -> tip)

## MINER
- the miner calls `blockchain.tip()` and set it as the parent of the block being mined.
- 
