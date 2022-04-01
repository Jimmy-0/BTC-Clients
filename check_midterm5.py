import sys
import json

def read_json(filename):
	f = open(filename)
	chain = json.load(f)
	return chain

if len(sys.argv)<2:
	print("Running Instructions:\npython check.py <num_chains>")
	sys.exit(0)
else:
	numChains = int(sys.argv[1])
print("number of chains = {}".format(str(numChains)))

chains = []
min_length = -1
min_trx_throughput = -1
min_trx_per_block = -1
min_frac_unique_trx = 2
for i in range(numChains):
	chain = read_json("expt/"+str(i)+".trx")
	trx_count = read_json("expt/"+str(i)+".trx_count")
	trx_count = int(trx_count)
	chains.append(chain)
	min_length = len(chain) if min_length==-1 else min(min_length, len(chain))
	min_trx_throughput = trx_count if min_trx_throughput==-1 else min(min_trx_throughput, trx_count)
	trx_per_block = 0 if len(chain)==1 else trx_count / (len(chain) - 1)
	min_trx_per_block = trx_per_block if min_trx_per_block==-1 else min(min_trx_per_block, trx_per_block)
	# count trx and unique trx
	count = 0
	s = set()
	for c in chain:
		count += len(c)
		for t in c:
			s.add(t)
	assert trx_count == count, "trx_count not in accordance with chain length"
	frac_unique_trx = len(s) / trx_count
	assert frac_unique_trx >= 0.9, "frac_unique_trx < 0.9"
	min_frac_unique_trx = min(min_frac_unique_trx, frac_unique_trx)

common_prefix = True
for chain_id in range(1, len(chains)):
	common_prefix = common_prefix and chains[0][1][0]==chains[chain_id][1][0]
	if not common_prefix:
		break

time_in_min=5

print("min trx throughput = {} \n\
min trx per block = {} \n\
fraction of unique trx = {} \n\
common prefix = {} \n\
test = {}".format(min_trx_throughput, min_trx_per_block, min_frac_unique_trx, common_prefix, "PASS" if min_trx_throughput>=100*time_in_min and min_trx_per_block>=10 and min_trx_per_block<=100*time_in_min and min_frac_unique_trx>=0.9 and common_prefix else "FAIL"
))


