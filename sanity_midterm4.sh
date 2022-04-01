#!/bin/bash

if [[ $# -lt 1 ]]; then
	LAMBDA=0
else
	LAMBDA=$1
fi
echo "LAMBDA=${LAMBDA}"

#NETID="cmlin2-sa55"
#unzip -qq ${NETID}.zip -d $NETID

cargo build
BIN=target/debug/bitcoin
mkdir -p expt

./${BIN} --p2p 127.0.0.1:6000 --api 127.0.0.1:7000 > expt/1.out &
p1=$!
./${BIN} --p2p 127.0.0.1:6001 --api 127.0.0.1:7001 -c 127.0.0.1:6000 > expt/2.out &
p2=$!
./${BIN} --p2p 127.0.0.1:6002 --api 127.0.0.1:7002 -c 127.0.0.1:6001 > expt/3.out &
p3=$!

echo "p1=$p1, p2=$p2, p3=$p3"

#sleep 1 # to allow processes to spawn

echo "===== Starting Miners ====="
for i in {0..2}; do
	#echo "curl $i"
	retcode=1
	while [ $retcode -gt 0 ]; do
		curl -s http://127.0.0.1:700${i}/miner/start\?lambda\=${LAMBDA} >/dev/null
		retcode=$?
		#echo "retcode=$retcode"
	done
done

TIME=300
echo "===== Sleeping for $TIME sec ====="
sleep $TIME

chain1=$(curl -s http://127.0.0.1:7000/blockchain/longest-chain)
chain2=$(curl -s http://127.0.0.1:7001/blockchain/longest-chain)
chain3=$(curl -s http://127.0.0.1:7002/blockchain/longest-chain)

#echo -e "===== CHAINS =====\n${chain1}\n======\n${chain2}\n=======\n${chain3}\n========"
echo $chain1>expt/1.chain
echo $chain2>expt/2.chain
echo $chain3>expt/3.chain

echo "===== Sanity Checks ====="
python check.py 3

echo "===== Killing Processes ====="
kill -9 $p1 $p2 $p3

exit 0

