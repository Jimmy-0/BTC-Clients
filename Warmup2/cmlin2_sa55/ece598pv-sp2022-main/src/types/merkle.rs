use super::hash::{Hashable, H256};

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
	hash: Vec<H256>,
	leaf_size: usize,
}

fn combine_hash( hash1: H256, hash2: H256) -> H256 {
	//println!("called combine_hash");
	let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
	ctx.update(hash1.as_ref());
	ctx.update(hash2.as_ref());
	let digest = ctx.finish();
	let hash: H256 = digest.into();
	//println!("{}", hash);
	return hash;
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        //unimplemented!()
		
		//let d: Vec<T> = data.to_vec();
		//println!("in fn new");

		let leaf_size: usize = data.len();
		let num_levels: u32 = (leaf_size as f64).log(2.0).ceil() as u32;
		println!("leaf_size: {}, num_levels: {}", leaf_size, num_levels);
		let mut hash: Vec<H256> = Vec::new();
		let mut i: usize = 0;
		for datum in data.iter() {
			hash.push(datum.hash());
			println!("hash[{}]: {}", i, hash[i]);
			i = i+1;
		}

		let mut level_size: usize = leaf_size;
		i = 0;
		let mut total_nodes: usize = 0;
		for level in 0..num_levels {
			println!("level: {}, level_size: {}, total_nodes: {}", level, level_size, total_nodes);
			while i+1 < total_nodes + level_size {
				println!("i:{}", i);
				hash.push(combine_hash(hash[i], hash[i+1]));
				println!("h1:{}\nh2:{}\ncombined:{}", hash[i], hash[i+1], hash[hash.len()-1] );
				i += 2;
			}
			if level_size%2==1 {
				println!("i:{}", i);
				hash.push(combine_hash(hash[i], hash[i]));
				println!("h1:{}\nh2:{}\ncombined:{}", hash[i], hash[i], hash[hash.len()-1] );
				i += 1;
			}
			total_nodes += level_size;
			level_size = (level_size+1)>>1; // upper level will have half as many nodes
		}

		let tree = MerkleTree {
		 	hash: hash,
		 	leaf_size: data.len(),
		};
		//println!("exiting fn new");
		return tree;
	}

    pub fn root(&self) -> H256 {
        //unimplemented!()

		//let hash: H256 = [0u8; 32].into();
		//return hash;
		// return self.hash[0];
		let root_idx: usize = self.hash.len() - 1;
		return self.hash[root_idx];
	}

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut proof: Vec<H256> = Vec::new();
        let mut idx: usize = index;
		let mut i = index;
        let mut level_size = self.leaf_size;
        let mut total_nodes = 0;
		let mut level: u32 = 0;
		while level_size !=1{
			if (index>>level)%2 ==0 {
				if i==total_nodes + level_size - 1 { // right node does not exist when hash[i] is the last node in that level
					proof.push(self.hash[i]);
				} else { // right node exists
                	proof.push(self.hash[i+1]);
            	}
			} else { // left node always exists
                proof.push(self.hash[i-1]);
            }
            total_nodes += level_size;
            idx = idx>>1;
			i = total_nodes + idx;
            level_size = (level_size+1)>>1;
			level += 1;
        }
        return proof;
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    //unimplemented!()

	//println!("in fn verify");
	let mut hash: H256 = *datum;
	let mut level: u32 = 0;
	for sibling in proof.iter() {
		if (index>>level)%2==0 {
			hash = combine_hash(hash, *sibling);
		} else {
			hash = combine_hash(*sibling, hash);
		}
		println!("level: {}\nsibling: {}\nhash: {}", level, *sibling, hash);
		level += 1;
	}

	return hash==*root;
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST


#[cfg(test)]
mod tests {
    use crate::types::hash::H256;
    use super::*;
    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }
    #[test]
    fn sp2022autograder011() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }
    #[test]
    fn sp2022autograder012() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }
    #[test]
    fn sp2022autograder013() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
