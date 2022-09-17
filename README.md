# bitcoin-indexer

## Building

```
docker build --no-cache --tag quentinadam/bitcoin-indexer .
```

## Running

### Create a docker network
```
docker network create --driver bridge network
```

### Run bitcoin client container
```
docker run \
--rm \
--detach \
--volume /data/bitcoind:/home/bitcoin/.bitcoin \
--net=network \
--publish 127.0.0.1:8332:8332 \
--name=bitcoind \
ruimarinho/bitcoin-core \
-rpcallowip=0.0.0.0/0 \
-rpcbind=0.0.0.0 \
-rpcuser=user \
-rpcpassword=password
```

### Run indexer container
```
docker run \
--init \
--rm \
--detach \
--volume /data/bitcoind:/data/bitcoind \
--volume /data/indexer:/data/indexer \
--net=network \
--publish 0.0.0.0:80:80 \
--env HOST=0.0.0.0 \
--env PORT=80 \
--env THREADS=6 \
--env BATCH_SIZE=25000 \
--env STORE_FILE_PATH=/data/indexer/store.dat \
--env BLOCK_FILES_PATH=/data/bitcoind/blocks \
--env CONFIRMATIONS=6 \
--env UPDATE_INTERVAL=1000 \
--env RPC_SERVER_HOST=bitcoind \
--env RPC_SERVER_PORT=8332 \
--env RPC_SERVER_USER=user \
--env RPC_SERVER_PASSWORD=password \
--name bitcoin-indexer \
quentinadam/bitcoin-indexer
```
