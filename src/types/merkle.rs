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
		if index>=self.leaf_size {
			return proof;
		}
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
	if index>=leaf_size {
		return false;
	}
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
	macro_rules! gen_merkle_tree_data_eight_case {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
				(hex!("0101010101010101010101010101010101010101010101010101010101010224")).into(),
				(hex!("0404040404040400440040404004040400404040400404040040404040040400")).into(),
				(hex!("0010101010101010101010101010101010101010101010101010101010101005")).into(),
				(hex!("0010101010101010101010101010101010101010101010101010101010101006")).into(),
				(hex!("0010101010101010101010101010101010101010101010101010101010101007")).into(),
				(hex!("0010101010101010101010101010101010101010101010101010101010101008")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root_self_eight_case() {
        let input_data: Vec<H256> = gen_merkle_tree_data_eight_case!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("a674de8a0d06ce67ff436e5a285e94fe7a09a3a6af90ebc8fcaac5466bc64224")).into()
        );
		/*
		1st layer
        "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        "0101010101010101010101010101010101010101010101010101010101010202"
		"af8720b28a0c14b8c2fb4a24be3da181acce38ccfdf842cc01808d74e890dfe2" is the hash of
		"0101010101010101010101010101010101010101010101010101010101010224"
		"4d3c5f7b99cac8d985a579e1a88e3d36721f590d6f7df224733231c16f7089b4" is the hash of
		"0404040404040400440040404004040400404040400404040040404040040400"
		"f3da65066b524082bebb401c37af34294a2732efd462376aa8e5e2f0286fe2f7" is the hash of
		"0010101010101010101010101010101010101010101010101010101010101005"
		"31fa3fd6947543060c7b287b4372ae3624c23b6474549334a0cd8e8194dc0ef3" is the hash of
		"0010101010101010101010101010101010101010101010101010101010101006"
		"8857bbc13186b859573d22705bcec5055d390568a8999e0fb43f2df9531039ee" is the hash of
		"0010101010101010101010101010101010101010101010101010101010101007"
		"c1e58753644079108567f9f17766ec24e1391193bd76f18e6e640dfe78da3b97" is the hash of
		"0010101010101010101010101010101010101010101010101010101010101008"
		2nd layer 
		"6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        the concatenation of these two hashes "b69..." and "965..."
		"dea84136958c9b102eb95000433bf0a5f6f0ffe14c368468419344d7473af271" is the hash of
        the concatenation of these two hashes "af8..." and "4d3..."
		"80cf5bcdfde47f51e809779386f1a203145ad8ddf011600379df0cdbcb717029" is the hash of
        the concatenation of these two hashes "f3d..." and "31f..."
		"3b4aec9ae166cdf3d4e156e389b28deed974341728974aa5ac60efb8f096055c" is the hash of
        the concatenation of these two hashes "885..." and "cle..."
		3rd layer
		"cc78a3cd1b3aeaa24e6e0c892f5c73a85e17a48dfb5c404db341be962a3d5b6d" is the hash of
        the concatenation of these two hashes "6b7..." and "dea..."
		"b4e0a1baeb632622999db4810b437c2322d87e86b5e1cbc405aa3f98328b6bd9" is the hash of
        the concatenation of these two hashes "80c..." and "3b4..." 
		4th layer 
		"a674de8a0d06ce67ff436e5a285e94fe7a09a3a6af90ebc8fcaac5466bc64224" is the hash of
        the concatenation of these two hashes "cc7..." and "b4e..." and this is the root
		*/
        
        // notice that the order of these two matters
    }
	#[test]
	fn merkle_proof_eight() {
        let input_data: Vec<H256> = gen_merkle_tree_data_eight_case!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![
					   (hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f")).into(),
					   (hex!("dea84136958c9b102eb95000433bf0a5f6f0ffe14c368468419344d7473af271")).into(),
					   (hex!("b4e0a1baeb632622999db4810b437c2322d87e86b5e1cbc405aa3f98328b6bd9")).into(),
				   ]
        );
		// the proof of "b69..." should be "965..." , "dea..." and "b4e..."
        
    }

	macro_rules! gen_merkle_tree_data_five_case {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
				(hex!("0101010101010101010101010101010101010101010101010101010101010224")).into(),
				(hex!("0202020020202002020200202020020202020020202020020202020200202022")).into(),
				(hex!("0901809018239821838129830921830921893801928391289038109283018290")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root_self_five_case() {
        let input_data: Vec<H256> = gen_merkle_tree_data_five_case!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("a268d5059d8f618c1ffff54d8a7f50c4728acd4c659d3634bf95ce1780561e4d")).into()
        );
		/*
		1st layer
        "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        "0101010101010101010101010101010101010101010101010101010101010202"
		"af8720b28a0c14b8c2fb4a24be3da181acce38ccfdf842cc01808d74e890dfe2" is the hash of
		"0101010101010101010101010101010101010101010101010101010101010224"
		"f70089eaf209fe16bc9b0e5b78d6858d71c52881e065133f029415c59ff6570f" is the hash of
		"0202020020202002020200202020020202020020202020020202020200202022"
		"dad251b5040ed49de9d950a2143c448123feee32a5dcc7f974069a5b2aed2c32" is the hash of
		"0901809018239821838129830921830921893801928391289038109283018290"
		2nd layer
		"6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        the concatenation of these two hashes "b69..." and "965..."
		"ef1e0cb8412d1c37bbc19fb12fa8a13cc267a7b6e53e61689f16cff83159e485" is the hash of
        the concatenation of these two hashes "af8..." and "f70..."
		"a2c52bc29476845c2c2d18aa0c8ffef9a6c2b82707e67e3f41eae8ad5ff4d762" is the hash of
        the concatenation of these two hashes "dad..." and "dad..."
		"2fa8bf87cdb118c7899044d5dc8575341f59bccecfc263051b3f22c30eab9d2b" is the hash of
        the concatenation of these two hashes "6b7..." and "ef1..."
		"555a7eedf9e821509525d8bc3733a5fa1e810080601db691efd949749639a6b0" is the hash of
        the concatenation of these two hashes "a2c..." and "a2c..."
		3rd layer
		"a268d5059d8f618c1ffff54d8a7f50c4728acd4c659d3634bf95ce1780561e4d" is the hash of
        the concatenation of these two hashes "2fa..." and "555..." and this is the root
		*/
		
		
        
        // notice that the order of these two matters
    }
	#[test]
	fn merkle_proof_five() {
        let input_data: Vec<H256> = gen_merkle_tree_data_five_case!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(4);
        assert_eq!(proof,
                   vec![
					   (hex!("dad251b5040ed49de9d950a2143c448123feee32a5dcc7f974069a5b2aed2c32")).into(),
					   (hex!("a2c52bc29476845c2c2d18aa0c8ffef9a6c2b82707e67e3f41eae8ad5ff4d762")).into(),
					   (hex!("2fa8bf87cdb118c7899044d5dc8575341f59bccecfc263051b3f22c30eab9d2b")).into(),
				   ]
        );
		// the proof of the fifth node,"dad..." should be "dad..." , "a2c..." and "2fa..."
        
    }
	#[test]
    fn merkle_verifying_five() {
        let input_data: Vec<H256> = gen_merkle_tree_data_five_case!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }

	macro_rules! gen_merkle_tree_data_three_case {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
				(hex!("0101010101010101010101010101010101010101010101010101010101010224")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root_self_three_case() {
        let input_data: Vec<H256> = gen_merkle_tree_data_three_case!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("10bb66da48b01dad1a87c69b097fc92c75d10135031c71855c1524875fba7e3b")).into()
        );
		/*
        "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        "0101010101010101010101010101010101010101010101010101010101010202"
		"6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        the concatenation of these two hashes "b69..." and "965..."
		extra test
		"af8720b28a0c14b8c2fb4a24be3da181acce38ccfdf842cc01808d74e890dfe2" is the hash of 
		"0101010101010101010101010101010101010101010101010101010101010224"
		"b801329936d912d102b181ba23269c280f361d696a8473d439178002bd81f7d7" is the hash of 
		the concatenation of "af8" and "af8"
		"10bb66da48b01dad1a87c69b097fc92c75d10135031c71855c1524875fba7e3b" = "6b7" and "b80"
		the root should be "10bb66da48b01dad1a87c69b097fc92c75d10135031c71855c1524875fba7e3b"
		*/
        
        // notice that the order of these two matters
    }
	#[test]
	fn merkle_proof_three() {
        let input_data: Vec<H256> = gen_merkle_tree_data_three_case!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![
					   (hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f")).into(),
					   (hex!("b801329936d912d102b181ba23269c280f361d696a8473d439178002bd81f7d7")).into(),
				   ]
        );
		// the proof of "b69..." should be "965..." and "b80..." 
        
    }

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
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
    fn merkle_proof() {
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
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
