# bee

A distributed bit-cask like LS Hash Table KV Store

```text
------------ recover from persisted state
    |
    ------- listner
    |
    ------- server for KV
    |
    ------- timeout for rpc's --> Leader Elections
                              |
                              --> AppendLogs
```
L
a[2] b[3] c[2]

b.votedFor = null
c.votedFor = node[a]

if (c.votedFor != null)
    respose false
if (c.logs > b.logs)
    node b is behind node c
    hence b can't get a vote from c, and
    it steps down from Candiate -> Follower (following node a)
else
    node `c` increses its term to 3
    votes for node `b`
