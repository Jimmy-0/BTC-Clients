import sys
import json

def read_chain(filename):
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
minLength = -1
maxLength = -1
for i in range(1, numChains+1):
	chain = read_chain("expt/"+str(i)+".chain")
	minLength = len(chain) if minLength==-1 else min(minLength, len(chain))
	maxLength = max(maxLength, len(chain))
	chains.append(chain)

count=0
while count < minLength:
	match=True
	j=1
	while match and j<len(chains):
		match = match and chains[0][count]==chains[j][count]
		j+=1
	if not match:
		break
	count+=1

print("longest chain length = {} \n\
length difference = {} \n\
common prefix = {} \n\
test = {}".format(minLength, maxLength-minLength, count, "PASS" if minLength>=50 and maxLength-minLength<=3 and minLength-count<=3 else "FAIL"
))


