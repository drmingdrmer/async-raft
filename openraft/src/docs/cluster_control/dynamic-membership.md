# Dynamic Membership

In contrast to the original raft, openraft considers all memberships as **joint** memberships.
A uniform configuration is simply a specific case of joint: the combination of only one configuration.

Openraft provides the following mechanisms for managing member node lifecycles:


## Membership API

### [`Raft::add_learner(node_id, node, blocking)`][`Raft::add_learner()`]

This method adds a learner to the cluster,
and immediately starts synchronizing logs from the leader.

- A **Learner** does not vote for leadership.


### [`Raft::change_membership(members, retain)`][`Raft::change_membership()`]

This method initiates a membership change and returns when the proposed
membership is effective and committed.

If there are nodes in the given membership that are not `Learners`, this method will fail.
Therefore, the application should always call [`Raft::add_learner()`] first.

Once the new membership is committed, a `Voter` not in the new configuration is removed if `retain=false`,
otherwise, it is converted to a `Learner` if `retain=true`.


#### Example of using `retain`

Given the original membership as `{"members":{1,2,3}, "learners":{}}`,
call `change_membership` with `members={3,4,5}`, then:

- If `retain=true`,  the new membership is `{"members":{3,4,5}, "learners":{1,2}}`.
- If `retain=false`, the new membership is `{"members":{3,4,5}, "learners":{}}`.


## Add a new node as a `Voter`

To add a new node as a `Voter`:
- First, add it as a `Learner`(non-voter) with [`Raft::add_learner()`].
  In this step, the leader sets up replication to the new node, but it cannot vote yet.
- Then, convert it into a `Voter` with [`Raft::change_membership()`].

```ignore
let client = ExampleClient::new(1, get_addr(1)?);

client.add_learner((2, get_addr(2)?)).await?;
client.add_learner((3, get_addr(3)?)).await?;
client.change_membership(&btreeset! {1,2,3}, true).await?;
```

A complete snippet of adding voters can be found in [Mem KV cluster example](https://github.com/datafuselabs/openraft/blob/d041202a9f30b704116c324a6adc4f2ec28029fa/examples/raft-kv-memstore/tests/cluster/test_cluster.rs#L75-L103).


## Remove a voter node

-   Call `Raft::change_membership()` on the leader to initiate a two-phase
    membership configuration change, e.g., the leader will propose two configuration logs:
    joint configuration log: `[{1, 2, 3}, {3, 4, 5}]` and then the uniform configuration log:
    `{3, 4, 5}`.

-   As soon as the leader commits the second configuration log, the node to remove can
    be safely terminated.

**Note that** An application does not have to wait for the configuration log to be
replicated to the node to remove. Because a distributed consensus protocol
tolerates a minority member crash.


To read more about Openraft's [Extended Membership Algorithm][`extended_membership`].

[`Raft::add_learner()`]: `crate::Raft::add_learner`
[`Raft::change_membership()`]: `crate::Raft::change_membership`
[`extended_membership`]: `crate::docs::data::extended_membership`