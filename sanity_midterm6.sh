#!/bin/bash

if [[ $# -ne 2 ]]; then
	#LAMBDA=10
	#THETA=5
    echo "usage: ./sanity.sh <LAMBDA> <THETA>"
	exit 1
else
	LAMBDA=$1
	THETA=$2
fi
echo "LAMBDA=${LAMBDA}, THETA=${THETA}"

cargo build
BIN=target/debug/bitcoin
mkdir -p expt

nohup ./${BIN} --p2p 127.0.0.1:6000 --api 127.0.0.1:7000 > expt/1.out &
p1=$!
nohup ./${BIN} --p2p 127.0.0.1:6001 --api 127.0.0.1:7001 -c 127.0.0.1:6000 > expt/2.out &
p2=$!
nohup ./${BIN} --p2p 127.0.0.1:6002 --api 127.0.0.1:7002 -c 127.0.0.1:6001 > expt/3.out &
p3=$!

echo "p1=$p1, p2=$p2, p3=$p3"

#sleep 1 # to allow processes to spawn

echo "===== Starting Generators ====="
for i in {0..2}; do
	#echo "curl $i"
	retcode=1
	while [ $retcode -gt 0 ]; do
		curl -s http://127.0.0.1:700${i}/tx-generator/start\?theta\=${THETA} >/dev/null
		retcode=$?
		#echo "retcode=$retcode"
	done
done

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

#exit 0

TIME=300
echo "===== Sleeping for $TIME sec ====="
sleep $TIME

echo "===== Saving outputs to file ====="
for i in {0..2}; do
	curl -s http://127.0.0.1:700${i}/blockchain/longest-chain > expt/${i}.chain
	curl -s http://127.0.0.1:700${i}/blockchain/longest-chain-tx > expt/${i}.trx
	curl -s http://127.0.0.1:700${i}/blockchain/longest-chain-tx-count > expt/${i}.trx_count
	for j in {0,10,20}; do
		curl -s http://127.0.0.1:700${i}/blockchain/state?block=${j} > expt/${i}.state_${j}
	done
done

#echo "===== Sanity Checks ====="
#python check_midterm6.py 3

echo "===== Killing Processes ====="
kill -9 $p1 $p2 $p3

exit 0

