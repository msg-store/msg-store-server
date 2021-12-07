# Message Store Server

## API

### Add Message
GET /api/msg

Add a mesage to the message store

#### Paramaters
* priority (required)   
description: The priority of the message.   
type: integer   

* msg (required)   
description: The message to save in the store.   
type: string   

```
curl --location --request POST 'localhost:8080/api/msg' \
--data-raw '{
    "priority": 1,
    "msg": "my message"
}'
```
