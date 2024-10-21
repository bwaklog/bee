# bee

A distributed bit-cask like LS Hash Table KV Store

Node starup tasks
```text
────┐
    │
  recover from persisted state
    │
    │
  starting node manager daemon
    │
    ├──── listener
    │
    ├──── server for KV
    │
    └──── timeout for rpc's
            │
            ├──── Leader Election
            │
            └──── Append Log RPC's + Heart Beats
```
