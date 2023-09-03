# Penguin

Penguin provides a simple REST api for configuring domain blocklists for certain client that
are then synced to configuration for a Squid proxy.


# APIs:

GET /v1/client - gets a list of all clients
POST /v1/client - creates a new client
GET /v1/client/{id} - gets a client by its id.
DELETE /v1/client/{id} - removes a client.
PUT /v1/client/{id} - updates a client

GET /v1/domainlist - gets a list of all blocklists
POST /v1/domainlist - creates a new blocklist
GET /v1/domainlist/id - gets a single blocklist
DELETE /v1/domainlist/id - deletes a single blocklist

GET /v1/client/{id}/rule

# Example flow:

Create a client.

`POST /v1/client`

Request:
```json
{
  "ip": "192.168.1.33",
  "name": "Caitlin's Laptop",
}
```

Response:
```json
{
  "id": 1,
  "ip": "192.168.1.33",
  "name": "Caitlin's Laptop"
}
```

Now, create a blocklist of banned domains.

`POST /v1/domainlist`

Request:
```json
{
  "name": "bad_domains",
  "domains": [
    ".horribleweb.com",
    ".terriblestuff.com"
  ]
}
```

Response:
```json
{
  "id": 1,
  "name": "bad_domains",
  "domains": [
    ".horribleweb.com",
    ".terriblestuff.com",
  ]
}
```

Create a rule that applies this blocklist to our client.

`PUT /v1/client/1`

Request:
```json
{
  "ip": "192.168.1.33",
  "name": "Caitlin's Laptop",
  "rules": [
    {
      "kind": "deny_http_access",
      "domainlists": [ 1 ]
    }
  ]
}
```

Create another blocklist for sites that are sometimes ok, but we'd like to ban normally.

`POST /v1/domainlist`

Request:
```json
{
  "name": "game_domains",
  "domains": [
    ".poki.com",
    ".roblox.com",
    ".minecraft.net"
  ]
}
```

Response:
```json
{
  "id": 2
  ...
}
```

Apply this blocklist by default.

`PUT /v1/client/1`

Request:
```json
{
  "ip": "192.168.1.33",
  "name": "Caitlin's Laptop",
  "rules": [
    {
      "kind": "deny_http_access",
      "domainlists": [ 1, 2 ]
    }
  ]
}
```

Add a temporary lease that allows access to these sites until some specified time.

`PUT /v1/client/1`

Request:
```json
{
  "ip": "192.168.1.33",
  "name": "Caitlin's Laptop",
  "rules": [
    {
      "kind": "deny_http_access",
      "domainlists": [ 1, 2 ]
    }
  ],
  "leases": [
    {
      "end_date": "2023-08-30T20:00:00",
      "rule": {
        "kind": "allow_http_access",
        "domainlists": [ 2 ]
      }
    }
  ]
}
```

## Implementation

The implementation generates / maintains a squid configuration directory that's expected
to be consumed by the `include` directive in `squid.conf`. For the example above, it would generate a directory (say `/etc/penguin/squid.d`) containing (assuming the lease wasn't present):

`client_001.conf`:
```
acl client_001 src 192.168.1.33/255.255.255.255
acl domains_001 dstdomain "./domains_001.txt"
acl domains_002 dstdomain "./domains_002.txt"
http_access deny client_001 domains_001 domains_002
```

When the lease is in place, it'd temporarily modify this to:

`client_001.conf`:
```
acl client_001 src 192.168.1.33/255.255.255.255
acl domains_001 dstdomain "./domains_001.txt"
http_access deny client_001 domains_001
```

The implementation also stores its own configuration data in json format in a separate directory, e.g.:

`/etc/penguin/conf/clients.json`
`/etc/penguin/conf/domains.json`

