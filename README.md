# Message Store Server

The message store server that implements the the msg-store library in a fast and light weight http server, Actix.
The msg-store allows for messages to be forwarded on a priority basis where bandwidth and disk capacity may be limited.

## Getting started
```bash
cargo install msg-store-server
```
or locally
```bash
cargo install --path .
```

for the leveldb backend use the level feature
```
cargo install --path . --no-default-features --features level
```

run with
```bash
msg-store-server
```

Adding a message
```bash
# request:
curl --location --request POST 'localhost:8080/api/msg' \
--data-raw '{
    "priority": 1,
    "msg": "sit exercitation eu aliquip ipsum"
}'

# 200 response:
{ "uuid": "1638901859659994278-1" }
```

Getting a message
```bash
# request
curl --location --request GET 'localhost:8080/api/msg'

# response:
{ 
    "data": { 
        "uuid": "1638901859659994278-1", 
        "msg": "sit exercitation eu aliquip ipsum" 
    } 
}
```

* Messages are forwarded on a highest priority then oldest status basis.
* Messages are pruned on a lowest priority then oldest status basis.
* Messages are only pruned once the store has reached the max byte size limit.
* The store as a whole contains a max byte size limit option, as does each individual priority group.  
For example, a developer can limit the size of the store to 1,000 bytes, while restricting priority group 1 to only 500 bytes, and leave higher priorities free with no restriction (except that of the store.)
* The store keeps track of basic statistics such as counting the messages that have been inserted, deleted, and pruned.
* Messages that have been deleted have been so on instructions of the developer using the del method. 
* Messages that have been pruned have been so automatically on insert of a new message or store/group defaults update once the max byte size limit has been reached.
* The store can contain 4,294,967,295 priority groups.

## Example
Four message are needed to be forwarded to a distant server.
The first message is placed in priority 1, the second in priority 2, the third message in priority 1, and the fourth in priority 2 again as shown in the rust example and table below.
```bash
# first message
curl --location --request POST 'localhost:8080/api/msg' \
--data-raw '{
    "priority": 1,
    "msg": "my first message"
}'
# response
{ "uuid": "1638909040889405720-1" }

# second message
curl --location --request POST 'localhost:8080/api/msg' \
--data-raw '{
    "priority": 2,
    "msg": "my second message"
}'
# response
{ "uuid": "1638909059786945856-1" }

# thrid message
curl --location --request POST 'localhost:8080/api/msg' \
--data-raw '{
    "priority": 1,
    "msg": "my thrid message"
}'
# response
{ "uuid": "1638909073359330845-1" }

# forth message
curl --location --request POST 'localhost:8080/api/msg' \
--data-raw '{
    "priority": 2,
    "msg": "my fourth message"
}'
# response
{ "uuid": "1638909087105753215-1" }

```

| priority 1 | priority 2 |
|:----------:|:----------:|
| msg 1      | msg 2      |
| msg 3      | msg 4      |

When get is called, msg 2 will be the first retrieved, because it is in the highest priority group and is also the oldest message in that group. The second msg retrieved would be msg 4.

While this may be the default, it is not strictly enforced. A developer could pass a priority to the get method to get the next message from that group, or one could also pass the uuid to the method to get the exact message desired.

Request the next message in queue
```bash
# request
curl --location --request GET 'localhost:8080/api/msg'
```
Response:
```json
{ 
    "data": { 
        "uuid": "1638909059786945856-1", 
        "msg": "my second message" 
    } 
}
```

Request a message from a specific priority group
```bash
curl --location --request GET 'localhost:8080/api/msg?priority=1'
```
```json
{ 
    "data": { 
        "uuid": "1638909040889405720-1", 
        "msg": "my first message" 
    } 
}
```

Request a specific message
```bash
curl --location --request GET 'localhost:8080/api/msg?uuid=1638909073359330845-1'
```

Response:
```json
{ 
    "data": { 
        "uuid": "1638909073359330845-1", 
        "msg": "my thrid message" 
    } 
}
```

Request the last inserted message from the highest priority group (LIFO)
```bash
curl --location --request GET 'localhost:8080/api/msg?reverse=true'
```
Response:
```json
{ 
    "data": { 
        "uuid": "1638909087105753215-1", 
        "msg": "my fourth message" 
    } 
}
```


On the other hand if there is a max byte size limit set, the first message to be pruned would msg 1, because it is in the lowest priority group and also the oldest message in that group. The second message pruned would be msg 3.

# API Documentation
The api documenation is in the OpenAPI 3.0 format and can be found in [/postman/schemas](/postman/schemas)