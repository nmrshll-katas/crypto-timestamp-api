# Cryptographically verifiable timestamps

Goals:

- Build a centralized timestamping service that allows users to post data and get cryptographic signatures from the service's private key. The service has a known public key.
- The service should be running over HTTP. You can use REST, JSON-RPC, etc.
- Any user can post data to the service to sign by bearing a "cost" directly proportional to the size of the data. You may consider PoW algorithms, one suggestion is Cuckoo (Rust implementation https://github.com/CodeChain-io/rust-cuckoo/).
- Any user can query the service's public key and verify the signature.
- It should not be possible to get more than 1 signature on the same data.
- The HTTP service should be non-blocking, i.e. it should be able to serve multiple requests in parallel.
- Unit tests should be added.

## Decisions

- Routes/methods:

  - get service's public key
    - no rate-limiting/PoW
  - sign data
    - rate-limited with PoW
    - data as base64-encoded bytes (since HTTP is text-only)
  - no need for a route to verify the signature:
    - this can be done on the client side
    - the user would need: the data, the server's signature, and to know which signature scheme was used
    - could build a client to automate that

- Errors:
  - Invalid PoW
  - Data already signed
  - ...

The server needs persistence to keep track of the data already signed.
We use Postgres, and store the hashes of the data already signed.

Signatures are done with ed25519. Hashes are done with Blake3.
Proof-of-work cost of request is done with Cuckoo.
