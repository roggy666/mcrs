# Inventory transactions: behavioural specification

Version: see `assets/minecraft/version.json`.

This document states the request: an ordered list of operations on slots that applies in full or
not at all. The request does not depend on an edition. This document also states how a click of
Java Edition and a stack request of Bedrock Edition become operations.

`item-stack.md` states the stack. This document
refers to its rules by id.

## 0. Conventions

### 0.1 Terms

- **Server**, **client**, **vanilla server**, **clientbound**, **serverbound**: as `login.md`,
  section 0.1, defines them.
- **Stack**, **empty stack**, **count**, **maximum stack size**, **strict validation**: as
  `item-stack.md`, section 0.1, defines them.
- **Holder**: an object that has a fixed number of slots, for example a player or a chest.
- **Slot**: one place of a holder. A slot holds one stack or the empty stack.
- **Slot reference**: a holder and the index of one of its slots.
- **Cursor**: the holder that holds the stack that a player carries with the pointer. Each player
  has one cursor. The cursor has one slot, with the index 0.
- **Operation**: one of `split`, `merge`, `swap` and `drop` (section 3).
- **Request**: a request id and an ordered list of operations (ITX-MOD-001).
- **Request id**: a number that the sender of a request selects. The result repeats it.
- **Working state**: the stacks of the slots while the server examines a request (ITX-ATM-001).
- **Slot limit**: the largest count that a slot can hold (ITX-LIM-001).
- **Capacity**: the largest count of one stack that a slot can hold (ITX-LIM-002).
- **Take filter**: a condition of a slot that permits the removal of items (ITX-LIM-004).
- **Place filter**: a condition of a slot that permits a given stack to enter (ITX-LIM-005).
- **Dropped stack**: a stack that a `drop` operation removes from the slots (ITX-OP-020).
- **Rejection**: the result of a request that the server examines and that does not apply. It has
  a reason and the index of an operation.
- **Plugin**: code that a user of the library adds to the server.
- **Actor**: the entity that made a request. For a request of an edge, the actor is the player of
  the connection. For a request of a command, of a plugin or of the vanilla gameplay, the actor is
  the entity that made the request.
- **Request source**: the identity that an edge or a plugin holds as a sender of requests.
- **Denial point**: the point where a plugin can stop a request before the server examines it
  (ITX-ATM-008).
- **Denial**: the result of a request that a plugin stops at the denial point (ITX-ATM-008).
- **Edge**: the part of the server that changes the packets of one edition into requests.
- **Clicked slot**: the slot that a `container_click` packet names.
- **Menu**: the ordered set of slots that a player has open. A later specification defines it.
- **Drag**: the sequence of `container_click` packets of the click kind quick craft.
- **Result slot**: a slot of a menu whose stack the menu makes from other slots, for example the
  slot of the craft result. A take from a result slot can give the slot a new stack.
- **Pass**: one execution of the two steps of a quick move (ITX-JAV-009).

### 0.2 Provenance

- `black_box_confirmed`: shown by an artefact that the game writes.
- `source_suggested_black_box_confirmed`: taken from the reference behaviour, and consistent with
  an artefact that the game writes.
- `source_suggested`: taken from the reference behaviour, with no result. The confidence is at
  most medium.
- `documentation`: taken from public documentation of a protocol.
- `design`: a choice of this project. The rule states the reason.
- `unresolved`: not decided. Section 11 collects these items.

### 0.3 Artefacts cited

- `assets/mcrs/reports/packets.json`: the packet id report of the game. It gives the packet names
  `container_click`, `container_set_slot`, `container_set_content` and `set_cursor_item`.
- The public protocol documentation of Bedrock Edition, repository
  `Mojang/bedrock-protocol-docs`, for protocol version 2225. Section 7 uses its names.
- The recordings of experiment E17, in `tools/mcrs_minecraft_client_recorder/recordings/inventory/`.
  A vanilla client plays each step with the input agent of `tools/vanilla-client`, against the
  vanilla server. A recording gives an event for each serverbound packet, with the hashed stacks
  of each click. It also marks each serverbound packet that the proxy removed. It gives the
  clientbound packets of the step that the recorder decodes, with each slot packet of the
  server. A rule cites a recording by its file name without the extension `.txt`.

### 0.4 ID prefixes

| Prefix | Subject |
|--------|---------|
| `ITX-MOD` | the request and its inputs |
| `ITX-LIM` | the slot limit, the capacity and the filters |
| `ITX-OP` | the operations |
| `ITX-REJ` | the rejection reasons and their order |
| `ITX-ATM` | atomicity and the result |
| `ITX-JAV` | the clicks of Java Edition |
| `ITX-BED` | the stack requests of Bedrock Edition |

### 0.5 Notation

`X` and `Y` are slot references. An integer range `a..b` includes `a` and `b`. Each value below
reads the working state.

| Notation | Meaning |
|----------|---------|
| `count(X)` | the count of the stack in `X`, or 0 when `X` holds the empty stack |
| `max(X)` | the maximum stack size of the stack in `X` (STK-CMP-001) |
| `limit(X)` | the slot limit of `X` |
| `cap(X, Y)` | the capacity of `X` for the stack in `Y`: the smaller of `limit(X)` and `max(Y)` |
| `same(X, Y)` | `X` and `Y` hold stacks that are not empty and that have the same item and components (STK-EQ-001) |
| `take(X)` | the take filter of `X` permits the removal |
| `place(X, Y)` | the place filter of `X` permits the stack in `Y` |
| `C` | the slot of the cursor of the player |
| `S` | the clicked slot |

## 1. The request

**ITX-MOD-001** (`design`, semantic). A request has a request id and an ordered list of
operations. The list can be empty. Each operation names its slots by slot reference. One request
can name slots of more than one holder. Reason: a click and a stack request of each edition are
such a list.

**ITX-MOD-002** (`design`, semantic). The cursor is a holder. An operation names the cursor slot
as it names each other slot. No operation is special for the cursor. A move from a slot to the
cursor is a take. A move from the cursor to a slot is a place. Reason: the two editions then use
the same operations.

**ITX-MOD-003** (`design`, semantic). A request has these inputs, and no other input:

- the stack of each slot that the request names;
- the slot limit of each such slot (ITX-LIM-001);
- the take filter and the place filter of each such slot (ITX-LIM-004, ITX-LIM-005).

The result of a request is a function of the operations and of these inputs. It does not depend
on time, on a random number or on the edition of the sender. A denial (ITX-ATM-008) is not a
result of this rule. A plugin decides it from inputs that this document does not state.

**ITX-MOD-004** (`design`, semantic). A request has at most 128 operations. Reason: the longest
list that a click of section 6 gives has 99 operations (ITX-JAV-011, ITX-JAV-014). A fixed
maximum gives a fixed cost for each request. See open question Q2.

**ITX-MOD-005** (`design`, semantic). The server applies the requests of one player in the order
in which they arrive. This document does not state the order of the requests of two players in
one tick. A parity test must not compare that order.

**ITX-MOD-006** (`design`, semantic). An operation does not state a stack that the sender
expects in a slot. An edge that receives such data examines it before it makes the request
(ITX-JAV-004, ITX-BED-003).

## 2. Limits and filters

**ITX-LIM-001** (`source_suggested_black_box_confirmed`, semantic). Each slot has a slot limit in the range `1..99`.
The specification of the holder gives the slot limit. The slot limit is 99 when that
specification gives none. The slot limit of the cursor slot is 99.

**ITX-LIM-002** (`source_suggested_black_box_confirmed`, semantic). The capacity of a slot for a stack is the smaller
of the slot limit and the maximum stack size of that stack. A `split` and a `merge` must not make
a count larger than the capacity.

**ITX-LIM-003** (`design`, unspecified). A stack can have a count larger than its maximum stack
size (STK-CNT-002). A `swap` moves such a stack as it is (ITX-OP-012). The result of a `split` or
a `merge` that has such a stack in its source slot is not specified beyond ITX-LIM-002. Reason:
strict validation rejects such a stack, and the vanilla client cannot make one.

**ITX-LIM-004** (`source_suggested_black_box_confirmed`, semantic). Each slot has a take filter. The take filter
permits or refuses the removal of items from the slot by the player. The specification of the
holder gives the take filter. The take filter permits when that specification gives none. The
take filter of the cursor slot always permits.

**ITX-LIM-005** (`source_suggested_black_box_confirmed`, semantic). Each slot has a place filter. For a given stack,
the place filter permits or refuses that stack in the slot. The count of the stack has no part in
the place filter. The specification of the holder gives the place filter. The place filter
permits when that specification gives none. The place filter of the cursor slot always permits.

**ITX-LIM-006** (`design`, semantic). A filter reads only the slot, the player and the stack. A
filter must give the same answer for each operation of one request. Reason: the result of a
request must not depend on the order in which the server reads the filters.

## 3. The operations

Each operation has conditions and a result. When a condition is false, the request has a rejection
with the reason that ITX-REJ-002 gives. The result changes the working state.

### 3.1 Split

**ITX-OP-001** (`design`, semantic). `split(from, to, n)` moves exactly `n` items from the slot
`from` to the slot `to`. It moves `n` items or it is a rejection. It never moves a smaller number.

**ITX-OP-002** (`design`, semantic). A `split` applies when each of these conditions is true:

1. `from` and `to` exist, and they are different slots.
2. `n` is 1 or more.
3. `from` holds a stack that is not empty, and `n` is not larger than `count(from)`.
4. `take(from)` is true.
5. `to` holds the empty stack, or `same(from, to)` is true.
6. `place(to, from)` is true.
7. `count(to) + n` is not larger than `cap(to, from)`.

**ITX-OP-003** (`design`, semantic). The result of a `split` is:

- `count(from)` decreases by `n`. When it becomes 0, `from` holds the empty stack (STK-EMP-001).
- When `to` held the empty stack, `to` holds a new stack with the count `n`. Its item and its
  components are those of the stack of `from`.
- When `to` held a stack, `count(to)` increases by `n`.

**ITX-OP-004** (`design`, semantic). A `split` does not use the condition of STK-EQ-003. Two
stacks with the same item and components can join when the capacity permits. Reason: a click of
the reference joins such stacks (ITX-JAV-008). The difference shows only on a stack that strict
validation rejects. STK-EQ-004 states which caller of the reference does the check.

### 3.2 Merge

**ITX-OP-005** (`design`, semantic). `merge(from, to)` moves as many items as `to` can accept from
`from` to `to`. The number is `m`, the smaller of `count(from)` and `cap(to, from) - count(to)`.
The items that do not move stay in `from`.

**ITX-OP-006** (`design`, semantic). A `merge` applies when each of these conditions is true:

1. `from` and `to` exist, and they are different slots.
2. `from` holds a stack that is not empty.
3. `take(from)` is true.
4. `to` holds the empty stack, or `same(from, to)` is true.
5. `place(to, from)` is true.
6. `m` is 1 or more.

**ITX-OP-007** (`design`, semantic). The result of a `merge` is the result of `split(from, to, m)`
(ITX-OP-003). A `merge` that can move no item is a rejection, not an operation without effect.
Reason: the sender then learns that its view of the slots is wrong.

### 3.3 Swap

**ITX-OP-010** (`design`, semantic). `swap(left, right)` exchanges the stacks of two slots. Each
stack keeps its item, its components and its count. One of the two slots can hold the empty
stack. The operation then moves one stack to an empty slot.

**ITX-OP-011** (`design`, semantic). A `swap` applies when each of these conditions is true:

1. `left` and `right` exist, and they are different slots.
2. At least one of the two slots holds a stack that is not empty.
3. The take filter permits for each of the two slots that holds a stack.
4. `place(right, left)` is true when `left` holds a stack.
5. `place(left, right)` is true when `right` holds a stack.
6. `count(left)` is not larger than `limit(right)`.
7. `count(right)` is not larger than `limit(left)`.

**ITX-OP-012** (`design`, semantic). A `swap` compares the count with the slot limit, not with the
capacity. It does not join two stacks, also when `same(left, right)` is true. Reason: a `swap`
makes no new count, so the maximum stack size has no part.

### 3.4 Drop

**ITX-OP-020** (`design`, semantic). `drop(from, n)` removes exactly `n` items from the slot
`from`. The removed items are one dropped stack, with the item and the components of the stack of
`from` and the count `n`. The request gives its dropped stacks to the caller in the order of the
operations. Two `drop` operations give two dropped stacks.

**ITX-OP-021** (`design`, semantic). A `drop` applies when each of these conditions is true:

1. `from` exists.
2. `n` is 1 or more.
3. `from` holds a stack that is not empty, and `n` is not larger than `count(from)`.
4. `take(from)` is true.

**ITX-OP-022** (`design`, semantic). The result of a `drop` is that `count(from)` decreases by
`n`. When it becomes 0, `from` holds the empty stack. This document does not state where a dropped
stack goes in the world.

## 4. Rejection

**ITX-REJ-001** (`design`, semantic). A request that the server examines has one of two results:
applied, or a rejection. A rejection has one reason from this closed list, and the index of one
operation. The index of the first operation is 0.

| Reason | Condition | Operations |
|--------|-----------|------------|
| `too_many_ops` | the request has more than 128 operations (ITX-MOD-004) | the request |
| `no_such_slot` | a slot reference names a holder that does not exist, or an index that the holder does not have | all |
| `same_slot` | the two slot references of one operation are equal | `split`, `merge`, `swap` |
| `bad_count` | `n` is 0 | `split`, `drop` |
| `empty_source` | `from` holds the empty stack; for a `swap`, the two slots hold the empty stack | all |
| `not_enough` | `n` is larger than `count(from)` | `split`, `drop` |
| `take_denied` | the take filter refuses for a slot that loses items | all |
| `not_equal` | `to` holds a stack, and `same(from, to)` is false | `split`, `merge` |
| `place_denied` | the place filter of a slot refuses the stack that enters | `split`, `merge`, `swap` |
| `no_room` | the count is larger than the capacity (`split`), `m` is 0 (`merge`), or a count is larger than a slot limit (`swap`) | `split`, `merge`, `swap` |

No other reason exists. An edge can refuse to make a request from a packet. That refusal is not a
rejection of this document (ITX-MOD-006). A denial is also not a rejection (ITX-ATM-008).

**ITX-REJ-002** (`design`, semantic). When more than one reason applies to one operation, the
rejection has the reason that comes first in this order:

1. `no_such_slot`
2. `same_slot`
3. `bad_count`
4. `empty_source`
5. `not_enough`
6. `take_denied`
7. `not_equal`
8. `place_denied`
9. `no_room`

This is the order of the table of ITX-REJ-001 without `too_many_ops`. Reason: the checks that
need no stack come first, and a filter comes before the check that needs it.

**ITX-REJ-003** (`design`, semantic). The server examines `too_many_ops` before each other
reason. The index of that rejection is 128, the index of the first operation past the maximum.

**ITX-REJ-004** (`design`, semantic). The server examines the operations in the order of the
list. The rejection names the first operation that has a false condition. The server does not
examine the operations after it.

**ITX-REJ-005** (`design`, semantic). For a reason that applies to two slots of one operation,
the first slot reference of the operation decides before the second. This has no effect on the
reason or on the index. It has an effect only on a log.

## 5. Atomicity and the result

**ITX-ATM-001** (`design`, semantic). The server examines each operation against the working
state. The working state starts as the stacks of the slots when the server starts the request.
The result of each operation that applies changes the working state for the operations after it.
Thus a later operation can use a stack that an earlier operation moved.

**ITX-ATM-002** (`design`, semantic). A request applies in full or not at all. When each
operation applies, the stacks of the slots become the working state, and the caller gets the
dropped stacks. When one operation is a rejection, no slot changes and no dropped stack exists.

**ITX-ATM-003** (`design`, semantic). No other change to the slots that the request names occurs
between the first operation and the last operation of one request.

**ITX-ATM-004** (`design`, semantic). A request with an empty list is applied when no plugin
denies it (ITX-ATM-008). It changes no slot.

**ITX-ATM-005** (`design`, semantic). An applied request keeps the items. For each item and
component patch, this sum is the same before and after the request: the counts of the slots that
the request names, plus the counts of the dropped stacks. No operation of this document makes or
destroys an item.

**ITX-ATM-006** (`design`, semantic). The result of a request has the request id, and one of
these three:

- applied: each slot whose stack changed, with the stack before and the stack after, and the
  dropped stacks in order;
- rejection: the reason and the index of the operation;
- denial: the two values of ITX-ATM-009. A denial has no reason of ITX-REJ-001 and no index of an
  operation.

A denial is a result of its own, not a reason of ITX-REJ-001. Reason: each reason of ITX-REJ-001
is a function of the inputs of ITX-MOD-003 and names one operation. A denial depends on other
inputs, names no operation, and does not show a defect of the edge (ITX-JAV-003).

**ITX-ATM-007** (`design`, semantic). After a rejection or a denial, the client must get the
stack that the server holds for each slot that the client changed before the answer. The edge of
each edition sends it (ITX-JAV-005, ITX-JAV-022, ITX-BED-004, ITX-BED-006). Reason: the client
showed its own result before the answer came.

**ITX-ATM-008** (`design`, semantic). The server gives each request to the denial point one
time. The moves of a close are the one exception (ITX-ATM-012). Each request has an actor: the
request of an edge, of a command, of a plugin and of the vanilla gameplay. Thus a plugin that
denies by actor has an effect on each request of that actor. At the denial point, each plugin can
read the request and its actor. A plugin can then deny the request.

A request that a plugin denies has the result denial, and the server does not examine its
operations. A request that no plugin denies continues with ITX-REJ-003. When no plugin denies, no
request has a denial, and the server behaves as the reference (deviation D5). Reason: a plugin
must be able to stop an action that the filters cannot see, for example the clicks and the hand
actions of a frozen player. A filter reads only the slot, the player and the stack (ITX-LIM-006).

**ITX-ATM-009** (`design`, semantic). A denial has two values: the request source that the
plugin holds, and a code in the range `0..65535` that this plugin selects. The result gives the
two values to each reader of the result. When two or more plugins deny one request, the result
keeps the first denial. A later denial of the same request changes nothing. A plugin cannot
cancel a denial.

**ITX-ATM-010** (`design`, semantic). A denied request does not apply (ITX-ATM-002).
No slot changes, and no dropped stack exists. The server does these steps for each packet, in
this order:

1. The edge does the checks of the packet, for example ITX-JAV-001 and INV-CRE-003, and makes the
   request. A packet that fails a check makes no request, so it does not reach the denial point.
2. The request reaches the denial point (ITX-ATM-008).
3. When no plugin denies the request, the server examines `too_many_ops` (ITX-REJ-003) and then
   the operations (ITX-REJ-004).

The requests of one actor reach the denial point in the order of ITX-MOD-005. A request reaches
it after each earlier request of that actor has a result. A denial does not stop the next request
of the actor. That request reads the slots that the denial did not change.

**ITX-ATM-011** (`design`, semantic). At the denial point, a plugin must not change a slot that
the request names. The result of a request after such a change is not specified. A plugin that
must change a slot when it denies makes the change after the request has its result. Reason: the
edge computed the operations from these slots, and ITX-JAV-003 permits no change between the read
and the request.

**ITX-ATM-012** (`design`, semantic). The moves of a close never reach the denial point, and a
plugin cannot deny them. These moves are steps 1 and 2 of INV-CLS-001. This rule applies to each
close:

- a close after a `container_close` packet (INV-CLS-001);
- a close when the player leaves the server (INV-CLS-003);
- a close at a dimension change (INV-CLS-003).

Thus the cursor stack and the stacks of the craft grid always go where INV-CLS-001 states.
Reason: decision of the owner. The player file does not hold the cursor and the
craft grid (INV-SAV-001), so a denial at a leave would destroy them.

The moves of a close also read no take filter (INV-CLS-005). They enter only a slot whose place
filter permits the stack, as each insert of the player inventory does (INV-INS-003). Thus they
never have a rejection of ITX-REJ-001, for example `take_denied` or `place_denied`.

## 6. Java Edition: the click

A `container_click` packet has a container id, a state id, a slot index, a button and a click
kind. It also has a list of changed slots and a cursor stack, as hashed stacks (STK-HSH-001). The edge
reads the slots and makes one request for each packet. The operations of that request apply by
construction, because the edge computes them from the stacks that it reads.

### 6.1 The packet and the answer

**ITX-JAV-001** (`source_suggested_black_box_confirmed`, semantic). The server does these checks on the packet, in
this order. The first check that fails ends the work on the packet.

| # | Check | When it fails |
|---|-------|---------------|
| 1 | The container id is the id of the menu that the player has open. | The server does nothing and sends nothing. |
| 2 | The player is not a spectator and is not dead. | The server changes no slot. It sends each slot of the menu and the cursor stack. |
| 3 | The menu is valid for the player. A later specification states this check. | The server does nothing and sends nothing. |
| 4 | The slot index is smaller than the number of slots of the menu. | The server does nothing and sends nothing. |

Check 4 passes for the slot indices -1 and -999. ITX-JAV-027 states each other negative slot
index. In check 2, the server also sends the data values of the menu. When a menu other than the
inventory menu becomes not valid for the player, the server closes it as INV-CLS-002 states. A
click for its container id then fails check 1.

**ITX-JAV-027** (`design`, unspecified). The vanilla client sends only -1 and -999 as a negative
slot index. The server treats each other negative slot index as -1. A parity test must not send
such a slot index. Reason: one rule covers the input that the vanilla client does not send.

**ITX-JAV-002** (`source_suggested_black_box_confirmed`, semantic). After the checks, the edge makes the request of
the click kind (ITX-JAV-006) and the server applies it. A state id that is not the state id of
the server does not stop the click.

**ITX-JAV-003** (`design`, semantic). The edge reads the slots and the server applies the request
with no other change to those slots between the two. A request of a click that the edge makes
correctly has no rejection. A rejection of such a request shows a defect of the edge. The denial
point is between the read and the examination (ITX-ATM-010). A denial is not a defect of the
edge.

**ITX-JAV-004** (`source_suggested_black_box_confirmed`, client agreement). After the request, the server compares
the packet with the slots. A stack of the server matches a hashed stack of the packet as
STK-HSH-003 states.

| Condition | What the server sends |
|-----------|-----------------------|
| The state id of the packet is not the state id of the server. | Each slot of the menu and the cursor stack, in one `container_set_content` packet. |
| The state id is equal, and a slot does not match what the client holds for it. | That slot, in a `container_set_slot` packet. |
| The state id is equal, and the cursor stack does not match. | The cursor stack, in a `set_cursor_item` packet. |
| The state id is equal, and each slot and the cursor stack match. | Nothing. |

For a slot that the packet lists as changed, the client holds the hashed stack of the packet. For
each other slot, the client holds the stack that the server sent last.

**ITX-JAV-005** (`design`, semantic). When a request of a click has a rejection, the server
changes no slot and then does ITX-JAV-004. The client thus gets each stack that differs from its
own result. The server does not disconnect the player. Reason: ITX-ATM-007, and the reference
also only corrects the client.

### 6.2 The click kinds

**ITX-JAV-006** (`source_suggested_black_box_confirmed`, semantic). Each click kind gives the operations of this
table. A click whose condition is false gives a request with an empty list.

| Click kind | Wire value | Buttons | Operations | Rule |
|------------|------------|---------|------------|------|
| pick up | 0 | 0 primary, 1 secondary | `swap`, `split` or `drop` between `S` and `C`; at most 1 | ITX-JAV-007, ITX-JAV-008 |
| quick move | 1 | 0, 1 | `split` from `S` to slots of the menu, or the `drop` of ITX-JAV-007 | ITX-JAV-009 |
| swap with a hotbar key | 2 | `0..8` hotbar slot, 40 off hand | `swap` or `split` between `S` and a slot of the player; at most 2 | ITX-JAV-010 |
| clone | 3 | not used | none; the click makes items | ITX-JAV-012 |
| throw | 4 | 0 one item, other values the full stack | `drop` from `S` | ITX-JAV-013 |
| quick craft | 5 | stage and drag kind (ITX-JAV-015) | `split` from `C` to each slot of the drag, on the last packet | ITX-JAV-014 to ITX-JAV-018 |
| pick up all | 6 | 0 forward, other values backward | `split` from slots of the menu to `C` | ITX-JAV-011 |

A click kind with a button that the table does not list gives an empty list.

**ITX-JAV-007** (`source_suggested_black_box_confirmed`, semantic). Pick up and quick move with the slot index -999
drop from the cursor. When `C` holds the empty stack, the list is empty. With button 0 the list
is `drop(C, count(C))`. With button 1 the list is `drop(C, 1)`. Pick up and quick move with each
other negative slot index give an empty list.

**ITX-JAV-008** (`source_suggested_black_box_confirmed`, semantic). Pick up on a slot of the menu gives the operation
of the first row of this table whose condition is true. `k` is `count(C)` with button 0, and 1
with button 1.

| # | Condition | Operation |
|---|-----------|-----------|
| 1 | An item of `C` or of `S` takes the click for itself. A later specification states these items. | none in this document |
| 2 | `S` and `C` hold the empty stack. | none |
| 3 | `S` holds the empty stack, and `place(S, C)` is false. | none |
| 4 | `S` holds the empty stack. | `split(C, S, n)`, where `n` is the smaller of `k` and `cap(S, C)` |
| 5 | `take(S)` is false. | none |
| 6 | `C` holds the empty stack, button 0. | `swap(S, C)` |
| 7 | `C` holds the empty stack, button 1. | `split(S, C, n)`, where `n` is `count(S)` divided by 2, rounded up |
| 8 | `place(S, C)` is true, and `same(S, C)` is true. | `split(C, S, n)`, where `n` is the smaller of `k` and `cap(S, C) - count(S)` |
| 9 | `place(S, C)` is true, and `count(C)` is not larger than `cap(S, C)`. | `swap(S, C)` |
| 10 | `place(S, C)` is true. | none |
| 11 | `same(S, C)` is true. | `split(S, C, n)`, where `n` is the smaller of `count(S)` and `max(C) - count(C)` |
| 12 | each other case | none |

When `n` is 0 or less, the click gives no operation. In row 11, the click gives no operation
when `n` is smaller than `count(S)` and `place(S, S)` is false.

**ITX-JAV-009** (`source_suggested_black_box_confirmed`, semantic). Quick move on a slot of the menu
gives no
operation when `take(S)` is false or `S` holds the empty stack. In each other case, the
specification of the menu gives an ordered list of destination slots for `S`. The edge then makes
the operations in two steps:

1. When the stack of `S` can stack (STK-EQ-003), for each destination `D` in order where
   `same(S, D)` and `place(D, S)` are true: `split(S, D, n)`. `n` is the smaller of `count(S)` and
   `cap(D, D) - count(D)`. The step stops when `S` holds the empty stack.
2. When `S` still holds a stack, for the first destination `D` in order that holds the empty
   stack and for which `place(D, S)` is true: `split(S, D, n)`. `n` is the smaller of `count(S)`
   and `cap(D, S)`.

Deviation D4 applies to the place filter in step 1. A `split` with `n` of 0 or less is not in the list. The
destination order of each menu is not in this document. ITX-JAV-026 states the repeat of a quick
move.

**ITX-JAV-026** (`source_suggested_black_box_confirmed`, semantic). The repeat of a quick move applies to each menu.
This includes the inventory menu, the slot of its craft result and each other result slot. After
a pass, the click does one more pass when each of these conditions is true:

- the last pass moved at least one item;
- `S` holds a stack with the item of the stack before the click.

The second condition compares only the item, not the components. Each pass gets the destination
list again from the specification of the menu, with the stacks at the start of that pass. A pass
after the first does not examine `take(S)` again. The repeat stops after the first pass for which
a condition is false.

On a slot that is not a result slot, a pass changes the stack of `S` only by the items that it
moves. Thus each destination gets at most one `split` in all passes of one click. A pass after
the first moves items only when step 2 of the pass before it moved fewer items than `count(S)`,
because `cap(D, S)` was smaller. On a result slot, the specification of that slot states how the
slot gets a new stack after a take. Thus each pass on a result slot can move items again. See open
question Q2.

**ITX-JAV-010** (`source_suggested_black_box_confirmed`, semantic). Swap with a hotbar key uses the slot `H` of the
player that the button names. When `S` is the slot `H`, the click gives no operation. Thus no
slot changes, and the edge makes no `swap` that ITX-OP-011 rejects (ITX-JAV-003). In each other
case, the click gives the operations of the first row whose condition is true.

| # | Condition | Operations |
|---|-----------|------------|
| 1 | The slot index is negative. | none (deviation D1) |
| 2 | `S` and `H` hold the empty stack. | none |
| 3 | `H` holds the empty stack. | `swap(S, H)` when `take(S)` is true; none in the other case |
| 4 | `S` holds the empty stack, and `place(S, H)` is false. | none |
| 5 | `S` holds the empty stack, and `count(H)` is not larger than `cap(S, H)`. | `swap(H, S)` |
| 6 | `S` holds the empty stack. | `split(H, S, cap(S, H))` |
| 7 | `take(S)` is false, or `place(S, H)` is false. | none |
| 8 | `count(H)` is not larger than `cap(S, H)`. | `swap(S, H)` |
| 9 | each other case | The stack of `S` goes to the inventory of the player, then `split(H, S, cap(S, H))`. |

In row 9, the stack of `S` enters the inventory of the player by the insert rule of the player
inventory. The insert skips each slot whose place filter refuses the stack (INV-INS-003). The part
that does not fit becomes a dropped stack. `inventory.md` states that rule (INV-INS-001).
Experiment E10 shows the dropped stack as an item entity.

**ITX-JAV-011** (`source_suggested_black_box_confirmed`, semantic). Pick up all gives operations when each of these
conditions is true: the slot index is 0 or more, `C` holds a stack, and `S` holds the empty stack
or `take(S)` is false. The edge visits the slots of the menu two times. With button 0 the order
is from the first slot to the last slot. With each other button the order is from the last slot
to the first slot. A visit stops when `count(C)` is equal to `max(C)`. For each slot `X` where
each condition of the list below is true, the edge adds `split(X, C, n)`. `n` is the smaller of
`count(X)` and `max(C) - count(C)`.

- `same(X, C)` is true, and `count(X)` is not larger than `max(C)`.
- `take(X)` is true.
- The menu permits pick up all from `X`. `inventory.md` (INV-MNU-005) and
  `client-container-screen.md` (CCS-MNU-004) state this condition for their menus. A later
  specification states it for each other menu.
- In the first visit only: `count(X)` is not equal to `max(X)`.
- `n` is equal to `count(X)`, or `place(X, X)` is true.

The list has at most 98 operations, because each `split` adds 1 or more to `count(C)`.

**ITX-JAV-012** (`source_suggested_black_box_confirmed`, semantic). Clone has an effect only when each of these
conditions is true: the player has infinite materials (creative mode), `C` holds the empty stack,
the slot index is 0 or more, and `S` holds a stack. The cursor then holds a copy of the stack of
`S` with the count `max(S)`. `S` does not change. This click makes items, so no operation of this
document states it (ITX-ATM-005). `inventory.md` states its operation (INV-CRE-005).

**ITX-JAV-013** (`source_suggested_black_box_confirmed`, semantic). Throw gives an operation when each of these
conditions is true: `C` holds the empty stack, the slot index is 0 or more, `S` holds a stack,
`take(S)` is true, and the player can drop items. `inventory.md` states the last condition
(INV-MNU-006). With button 0 the list is `drop(S, 1)`. With each other button the list is
`drop(S, count(S))`. A later specification states the repeat of a throw for a slot that fills
again. That repeat occurs only with button 1.

### 6.3 The drag

**ITX-JAV-014** (`source_suggested_black_box_confirmed`, semantic). A drag is a sequence of quick craft packets: one
start packet, one packet for each slot, and one end packet. The server holds a drag state for each
menu: idle or open, the drag kind, and a set of slots. Only the end packet gives operations. The
list has at most 99 operations.

**ITX-JAV-015** (`source_suggested_black_box_confirmed`, wire). The button of a quick craft packet holds two numbers.
The stage is `button & 3`. The drag kind is `(button >> 2) & 3`.

| Stage | Meaning | | Drag kind | Meaning |
|-------|---------|-|-----------|---------|
| 0 | start | | 0 | an equal part for each slot |
| 1 | add the clicked slot | | 1 | one item for each slot |
| 2 | end | | 2 | a full stack for each slot; creative mode only |

Stage 3 and drag kind 3 are not valid.

**ITX-JAV-016** (`source_suggested_black_box_confirmed`, semantic). The server does this for each quick craft packet,
by the first row whose condition is true. "Reset" sets the drag state to idle with an empty set.

| # | Condition | Result |
|---|-----------|--------|
| 1 | Stage 0 or stage 3 when the drag is open; stage 1, 2 or 3 when the drag is idle. | reset |
| 2 | `C` holds the empty stack. | reset |
| 3 | Stage 0, and the drag kind is 0 or 1, or is 2 for a player with infinite materials. | The drag is open with that drag kind and an empty set. |
| 4 | Stage 0. | reset |
| 5 | Stage 1. | The server adds `S` to the set when ITX-JAV-017 permits. |
| 6 | Stage 2. | The edge makes the request of ITX-JAV-018. Then reset. |

A click of each other click kind while the drag is open resets the drag. That click gives an
empty list.

**ITX-JAV-017** (`source_suggested_black_box_confirmed`, semantic). A slot `S` enters the set when each of these
conditions is true:

- `S` holds the empty stack, or `same(S, C)` is true and `count(S)` is not larger than `max(C)`;
- `place(S, C)` is true;
- the drag kind is 2, or `count(C)` is larger than the number of slots in the set;
- the menu permits a drag to `S`. `inventory.md` (INV-MNU-005) and `client-container-screen.md`
  (CCS-MNU-004) state this condition for their menus. A later specification states it for each
  other menu.

A slot that is in the set does not enter it again. A stage 1 packet with a negative slot index
adds no slot (deviation D1).

**ITX-JAV-018** (`source_suggested_black_box_confirmed`, semantic). The end packet gives these operations. `z` is the
number of slots in the set.

- When `z` is 0, the list is empty.
- When `z` is 1, the result is the pick up click of ITX-JAV-008 on that slot. The button of that
  click is the drag kind. Drag kind 2 thus gives an empty list.
- When `z` is 2 or more and the drag kind is 0 or 1, each slot `X` of the set gets one
  `split(C, X, n)`.

The part `a` is `count(C)` divided by `z`, rounded down, for drag kind 0. It is 1 for drag kind 1.
For each slot, `n` is the smaller of `a` and `cap(X, C) - count(X)`. The edge reads `count(C)` one
time, before the first `split`. A slot gets a `split` only when the conditions of ITX-JAV-017 are
true again, with this change: `count(C)` is `z` or more. A `split` with `n` of 0 or less is not in
the list. The items that no slot gets stay in the cursor.

**ITX-JAV-019** (`design`, semantic). The `split` operations of ITX-JAV-018 are in the order of
the slot index in the menu, smallest first. Reason: the result does not depend on the order, and
a fixed order makes the request the same in each run.

**ITX-JAV-020** (`source_suggested_black_box_confirmed`, semantic). A drag of drag kind 2 with 2 or more slots makes
items. The server does this for each slot `X` of the set for which the conditions of ITX-JAV-017
are true again:

- `X` holds a stack with the item and the components of `C` and the count `cap(X, C)`;
- `g(X)` is `cap(X, C)` minus the count that `X` held before.

After the last slot, the count of the cursor is `count(C)` minus the sum of each `g(X)`. When
that number is 0 or less, the cursor holds the empty stack. The click thus makes only the items
that the cursor did not hold. No operation of this document states it (ITX-ATM-005).
`inventory.md` states its operations (INV-CRE-005).

### 6.4 The denial

**ITX-JAV-021** (`source_suggested`, client agreement). The vanilla client changes some slots
before the answer of the server comes. This table lists them for each packet that gives a request.

| Packet of the client | What the client changes before the answer |
|----------------------|-------------------------------------------|
| `container_click` | each slot that the packet lists as changed, and the cursor (ITX-JAV-004) |
| `player_action`, status 4 | `P[n]`, the selected slot: it removes the items of the drop (INV-SEL-004) |
| `player_action`, status 5 or 7 | as ITX-JAV-025 states |
| `set_creative_mode_slot` | the slot of the packet (INV-CRE-004) |

`P[n]` and `M[i]` are as `inventory.md`, section 0.5, states. The moves of a close are not in the
table, because a plugin cannot deny them (ITX-ATM-012).

**ITX-JAV-025** (`source_suggested_black_box_confirmed`, client agreement). Before the answer of
the server, the vanilla client does this for a `player_action` packet:

- status 5: it removes one item from `P[n]`, the selected slot (INV-SEL-004);
- status 7: it changes neither `P[n]` nor `P[40]`.

Recordings: `inventory_e17_drop` (status 5), `inventory_e17_swap` (status 7).

**ITX-JAV-022** (`design`, client agreement). When a plugin denies a request of the Java edge, the
server does what this table states. The comparison is that of INV-SYN-003.

| Input | After the denial |
|-------|------------------|
| a click (ITX-JAV-002) | The server does ITX-JAV-004, as after a rejection (ITX-JAV-005). The client gets each slot and the cursor that it changed. ITX-JAV-024 states the drag. |
| a drop, status 4 or 5 (INV-SEL-004) | The client view of `P[n]` becomes not known. The comparison sends `P[n]`. |
| the hand swap, status 7 (INV-SEL-005) | The server does INV-SEL-006, as after a rejection. The comparison sends `P[n]` and `P[40]`. The client changed neither slot (ITX-JAV-025), so the two packets carry the stacks that it holds. The use of an item does not stop. |
| a creative set (INV-CRE-004) | The client view of `M[i]` is the stack of the packet, as INV-CRE-006 states for a failed check. The comparison sends `M[i]` when the two stacks differ. |

A denied request gives no dropped stack, so no item entity appears. The edge sends no other
packet for a denial, and the player stays connected. No packet of Java Edition carries the two
values of a denial. A plugin that denies can send a message to the player itself. Reason:
ITX-ATM-007, and each cause of a change reaches the client by one path (INV-SYN-004).

**ITX-JAV-024** (`design`, semantic). Each quick craft packet gives one request (ITX-JAV-002).
At stage 0 and at stage 1, that request has an empty list. When a plugin denies the request of a
quick craft packet, the drag becomes idle with an empty set:

- stage 0: the drag does not open;
- stage 1: the server does not add the slot to the set;
- stage 2: the drag resets, as row 6 of ITX-JAV-016 states.

A denied click of each other click kind resets an open drag, as ITX-JAV-016 states for each such
click. After a denied stage 0 or stage 1, row 1 of ITX-JAV-016 resets the drag at each later
packet of that drag. The end packet then gives an empty list, and ITX-JAV-004 corrects the client.
Reason: where ITX-JAV-016 gives a result, a denial keeps it. Where it gives none, a reset keeps no
state of a denied request.

## 7. Bedrock Edition: the stack request

The public protocol documentation names the packet `ItemStackRequest`. One request of that packet
has a request id and a list of actions. A slot of an action has a container, a slot index and a
stack network id. The answer has the request id, a result, and for a success the new count and
stack network id of each changed slot.

**ITX-BED-001** (`documentation`, semantic). The actions map to the operations as this table
states. The names and the numbers come from the public protocol documentation.

| Action | Number | Fields | Operation |
|--------|--------|--------|-----------|
| `Take` | 0 | amount, source, destination | `split(source, destination, amount)` |
| `Place` | 1 | amount, source, destination | `split(source, destination, amount)` |
| `Swap` | 2 | source, destination | `swap(source, destination)` |
| `Drop` | 3 | amount, source, randomly | `drop(source, amount)` |
| `Destroy` | 4 | amount, source | none in this document; the action destroys items |
| `Consume` | 5 | amount, source | none in this document; a craft uses it |
| `Create` | 6 | result slot | none in this document; the action makes items |
| each other action, `9..20` | | | none in this document |

See open question Q5.

**ITX-BED-002** (`documentation`, semantic). One Bedrock request is one request of this document.
The actions give the operations in the same order. The request id of the packet is the request
id. Thus a Bedrock request applies in full or not at all (ITX-ATM-002).

**ITX-BED-003** (`design`, semantic). The stack network id is state of the edge for one
connection. The edge compares each stack network id of a request with its own state before it
makes the request. A stack network id that is wrong is a refusal of the edge, not a rejection of
ITX-REJ-001. Reason: the request must not depend on
an edition (ITX-MOD-003).

**ITX-BED-004** (`design`, semantic). After a rejection or a refusal, the edge sends an answer
with the request id and a result that is not a success. The slots do not change. A later
specification of the Bedrock edge states which result value each reason gives.

**ITX-BED-005** (`design`, semantic). A `Take` or a `Place` whose amount makes a count larger
than the capacity has the rejection `no_room` (ITX-OP-002). The server must not accept a count
that the stack does not permit. Reason: the limits of a stack are the same for each edition.

**ITX-BED-006** (`design`, semantic). When a plugin denies a Bedrock request, the edge sends the
answer of ITX-BED-004: the request id and a result that is not a success. No slot changes. The
later specification of the Bedrock edge states the result value of a denial, as it does for each
reason.

## 9. Intentional deviations

**D1.**

- **Rule**: ITX-JAV-010 and ITX-JAV-017.
- **Reference**: two clicks with a negative slot index give an internal error in the reference.
  The reference then disconnects the player with the reason `disconnect.packetError` (E6, E14).
  They are the swap with a hotbar key and the stage 1 packet of a drag. Each negative slot index
  has this result, because check 4 of ITX-JAV-001 passes for it.
- **Visible to the client**: no. The vanilla client sends these clicks only for a slot of the
  menu.
- **Test**: a differential test must not send these two click kinds with a negative slot index.

**D2.**

- **Rule**: ITX-LIM-003.
- **Reference**: the reference moves a stack with a count larger than its maximum stack size
  with limits that differ between the click kinds. A drag can decrease the count of a slot.
- **Visible to the client**: yes, but only for a stack that strict validation rejects.
- **Test**: a differential test must not make a stack with a count larger than its maximum stack
  size.

**D3.**

- **Rule**: ITX-JAV-019.
- **Reference**: the reference visits the slots of a drag in an order that it does not define.
- **Visible to the client**: no. The stacks after the click are the same in each order.
- **Test**: a differential test compares the stacks, not the order of the packets for the slots.

**D4.**

- **Rule**: ITX-JAV-009.
- **Reference**: step 1 of the reference does not read the place filter of a destination.
- **Visible to the client**: not known. It shows only if a menu lists a destination that holds a
  stack and refuses an equal stack. The specification of each menu must examine this.
- **Test**: a differential test must not use such a destination until that specification exists.

**D5.**

- **Rule**: ITX-ATM-008 to ITX-ATM-011, ITX-JAV-022, ITX-JAV-024 and ITX-BED-006.
- **Reference**: the reference has no denial point. A plugin cannot stop a click, a hand action or
  a creative set. Each such packet that passes the checks of the server has its effect.
- **Visible to the client**: yes, when a plugin denies: the slots that the client changed return
  to the stacks of the server. With no such plugin, the server behaves as the reference.
- **Test**: a differential test runs with no plugin that denies.

## 10. Experiments

Each experiment of the kind probe runs against the vanilla server with `tools/mcrs_minecraft_probe`.
The scenario `tools/mcrs_minecraft_probe/scenarios/<name>.json` gives the steps. The trace
`tools/mcrs_minecraft_probe/traces/<name>.json` holds the menu packets and the command output of
each step. One scripted client joins; the server console runs the commands. A prediction of the
scripted client carries no components. Thus the server sends the cursor again after each click with
a stack that has components. This is the correction of ITX-JAV-004, not a change of a slot. `M[i]`
and `P[i]` are as `inventory.md`, section 0.5, states. In a chest menu, the entry `i` of `0..26` is
the slot `i` of the chest, and the entries `54..62` are `P[0]` through `P[8]`.

- **E1** (probe, run; `transactions_e01`). Rules: ITX-LIM-001, ITX-LIM-002, ITX-LIM-005,
  ITX-JAV-008.
  1. Open a brewing stand. Hold 64 potions with `max_stack_size` 64 and click primary on the
     bottle slot 0. Close the brewing stand.
  2. Open a chest, hold the 63 potions and click primary on the slot 0.
  3. Open the brewing stand, hold 64 stone and click primary on the bottle slot 1, then on the
     ingredient slot 3.
  4. Open the chest, hold 64 stone and click primary on the slot 1. Hold 16 stone with
     `max_stack_size` 16 and click primary on the slot 2, then secondary on the slot 2. Hold 99
     stone with `max_stack_size` 99 and click primary on the slot 3.

  Results: the bottle slot takes 1 potion, and the cursor keeps 63. The chest slot takes the 63
  potions. The bottle slot refuses the stone; the ingredient slot takes it. The chest slot takes 64
  stone, 16 stone, and 99 stone. The secondary click takes 8 of the 16 stone.

- **E2** (probe, run; `transactions_e02`). Rules: ITX-LIM-004, ITX-LIM-005, ITX-JAV-008,
  INV-MNU-007.
  1. Open a furnace. Hold 40 stone and click primary and secondary on the fuel slot 1, then on the
     result slot 2.
  2. Hold 1 stone and click primary on the fuel slot.
  3. Hold 10 coal and click primary on the fuel slot.
  4. Put 5 iron ingots in the result slot with `item replace block`. Click primary on it with an
     empty cursor.
  5. Put 5 iron ingots in the result slot again. Hold 3 iron ingots and click primary on it.

  Results: the open of the furnace sends `container_set_content` and then four `container_set_data`
  packets. The fuel slot and the result slot refuse the 40 stone with each button. The fuel slot
  refuses the 1 stone and takes the coal. The empty cursor takes the 5 ingots. The 3 ingots in the
  cursor do not enter the result slot; the cursor takes the 5 ingots and holds 8.

- **E3** (probe, run; `transactions_e03`). Rules: ITX-OP-004, ITX-JAV-008, ITX-JAV-009.
  1. Run `item replace entity` with `stone[max_stack_size=16,max_damage=10,damage=3]`.
  2. Put an iron pickaxe with `damage` 3 in `P[0]` and one in the cursor. Click primary on `M[36]`.
  3. Put an iron pickaxe with `damage` 3 in `M[9]`. Send a quick move on `M[9]`.
  4. Repeat the steps 2 and 3 with pickaxes without `damage`.

  Results: the game refuses the stack of step 1: an item cannot be both damageable and stackable.
  The click of step 2 exchanges the two pickaxes. The quick move puts the pickaxe in `M[37]`, the
  first empty slot of the hotbar. The pickaxes without `damage` give the same results.

- **E4** (probe, run; `transactions_e04`). Rules: ITX-JAV-001, ITX-JAV-027, INV-MNU-008,
  INV-MNU-011.
  1. Put 10 stone in `P[0]` and 7 granite in the cursor. Open a chest.
  2. Send a pick up click on the entry 10 with the 7 granite as the cursor of the packet.
  3. Send a pick up click on the entry 54 with the container id 5, then with the container id 0 on
     `M[36]`.
  4. As a spectator, send a pick up click on the entry 54 with the container id 1.
  5. In survival mode, send a pick up click on the entry 500, on the entry -1 and on the entry 63.
  6. Teleport 12 blocks away, wait 1.5 seconds, and send a pick up click on the entry 54.
  7. Teleport back. Send `container_close` for the inventory menu.
  8. In creative mode, teleport to the Nether and back. Open the chest.

  Results: the chest menu starts with the empty cursor, and the click of step 2 gets
  `set_cursor_item` with the empty stack; the chest stays empty. The clicks of step 3 and of step 5
  get nothing and change nothing. The spectator gets `container_set_content` of the chest menu. The
  teleport of step 6 makes the server send `container_close` with the container id 1, and the click
  after it gets nothing. The close of step 7 puts the 7 granite in `P[1]`. The chest of step 8 gets
  the container id 2.

- **E5** (probe, run; `transactions_e05`). Rules: ITX-JAV-002, ITX-JAV-004. Put 10 stone in `P[0]`.
  1. Send a pick up click on `M[36]` that lists `M[36]` as changed with 10 stone, and the cursor
     with 10 stone.
  2. Send the click that puts the stack back, with 10 stone as the cursor of the packet.
  3. Send the pick up click again, with 2 stone for `M[36]` and 8 stone for the cursor.
  4. Send the click that puts the stack back, with the state id plus 1.
  5. Send the pick up click again. List `M[37]` with 1 dirt as changed and nothing for `M[36]`.
  6. Send the click that puts the stack back, with a correct prediction.

  Results: step 1 gets `M[36]` with the empty stack. Step 2 gets `set_cursor_item` with the empty
  stack. Step 3 gets `M[36]` and `set_cursor_item`. Step 4 applies and gets
  `container_set_content` with the next state id. Step 5 gets `M[36]` and `M[37]`. Step 6 gets
  nothing.

- **E6** (probe, run; `transactions_e06`). Rules: ITX-JAV-006, ITX-JAV-007, ITX-JAV-010,
  ITX-JAV-013, INV-MNU-003, deviation D1.
  1. For each click kind, send the click on `M[9]` with the buttons 0, 1, 2, 40 and 9. Before
     each click, put 10 stone in `M[9]` and empty the cursor.
  2. With 10 stone in the cursor, send pick up and quick move on the slot index -1. On the slot
     index -999, send quick move with the button 1, then pick up with the button 1, then pick up
     with the button 0. Then send pick up on the slot index -999 with an empty cursor.
  3. Send a swap click on the slot index -1.

  Results of step 1: pick up takes the stack with the button 0 and 5 stone with the button 1.
  Quick move puts the stack in `M[36]` with the buttons 0 and 1. Throw makes an item entity of 1
  stone with the button 0 and of 10 stone with each other button. Swap puts the stack in `M[36]`,
  `M[37]`, `M[38]` and `M[45]` with the buttons 0, 1, 2 and 40. Each other click changes nothing.
  These are the buttons 2, 40 and 9 of pick up and of quick move, and the button 9 of swap. Each
  button of clone, quick craft and pick up all also changes nothing. Results of step 2: the slot
  index -1 changes nothing. The slot index -999 drops 1 stone with the button 1 and the stack with
  the button 0. An empty cursor drops nothing. Result of step 3: the server disconnects the client
  with the reason `disconnect.packetError`.

- **E7** (probe, run; `transactions_e07`). Rules: ITX-JAV-008. With 40 stone in the cursor, send a
  primary and a secondary pick up click on `M[9]`. Do this when `M[9]` holds the empty stack, 30
  stone, 64 stone and 10 dirt. With an empty cursor, send a primary and a secondary click on 7
  stone.

  Results: the empty slot takes 40 stone, or 1 stone with the secondary click. The 30 stone become
  64 with 6 in the cursor, or 31 with 39 in the cursor. The 64 stone change nothing. The 10 dirt and
  the 40 stone exchange with each button. The empty cursor takes the 7 stone, or 4 stone with the
  secondary click, and leaves 3.

- **E9** (probe, run; `transactions_e09`). Rules: ITX-JAV-009.
  1. Put 40 stone in the slots 0 and 5 of a chest and 64 stone in `P[0]`. Open the chest and send a
     quick move on the entry 54.
  2. Fill the chest with 64 dirt in each slot. Put 64 stone in `P[0]` and send a quick move on the
     entry 54.

  Results: step 1 fills the slots 0 and 5 to 64 and puts 16 stone in the slot 1. Step 2 moves
  nothing.

- **E10** (probe, run; `transactions_e10`). Rules: ITX-JAV-010, INV-MNU-003, INV-INS-001.
  1. Put 16 potions with `max_stack_size` 16 in `P[0]` and a potion of swiftness in the bottle slot
     0 of a brewing stand. Open the brewing stand and send a swap click with the button 0 on the
     entry 0.
  2. Repeat step 1 with 64 dirt in each other main slot.
  3. Repeat step 1 with the 16 potions in `P[40]` and the button 40.
  4. With 10 stone in `P[0]` and an empty bottle slot, send a swap click with the button 0 on the
     entry 0.
  5. Empty `P[0]` and put the potion of swiftness in the bottle slot. Send a swap click with the
     button 0 on the entry 0.

  Results: step 1 puts 1 potion of the 16 in the bottle slot, leaves 15 in `P[0]` and puts the
  potion of swiftness in `P[1]`. Step 2 puts 1 potion in the bottle slot and leaves 15 in `P[0]`;
  the potion of swiftness becomes an item entity. Step 3 puts 1 potion
  in the bottle slot, leaves 15 in `P[40]` and puts the potion of swiftness in `P[0]`. Step 4
  changes nothing. Step 5 puts the potion of swiftness in `P[0]`.

- **E11** (probe, run; `transactions_e11`). Rules: ITX-JAV-011. Put 10, 64, 20 and 5 stone in the
  slots 0 to 3 of a chest. Open the chest and hold 3 stone.
  1. Send pick up all with the button 0 on the empty entry 10.
  2. Fill the chest again and send pick up all with the button 1 on the entry 10.
  3. Fill the chest again and send pick up all with the button 0 on the entry 2. Then hold 3 dirt
     and send it on the entry 10. Then send it with an empty cursor.

  Results: step 1 empties the slots 0, 2 and 3, takes 26 stone from the slot 1 and gives the cursor
  64 stone. Step 2 empties the slots 3, 2 and 0. It takes 26 stone from the last stack of the menu
  and leaves the 64 stone of the slot 1. The three clicks of step 3 change nothing.

- **E12** (probe, run; `transactions_e12`). Rules: ITX-JAV-012, ITX-JAV-020. In creative mode, put
  5 stone in the slot 0 of a chest and open it.
  1. Send a clone click on the entry 0. Send it again with the 64 stone in the cursor.
  2. Send a drag of drag kind 2 over the empty entries 1, 2 and 3.
  3. Hold 60 stone, put 62 stone in the entry 4, and send a drag of drag kind 2 over the entries 4
     and 5.
  4. Send a clone click on the empty entry 10. In survival mode, send a clone click on the entry 0.

  Results: the first clone click gives the cursor 64 stone, and the slot keeps 5. The second clone
  click changes nothing. The drag of step 2 puts 64 stone in each of the three entries, and the
  cursor becomes empty. The drag of step 3 fills the entry 4 to 64, puts 64 stone in the entry 5,
  and empties the cursor. The clicks of step 4 change nothing.

- **E13** (probe, run; `transactions_e13`). Rules: ITX-JAV-013. Put 10 stone in `M[9]`.
  1. Send a throw click with the button 0, then with the button 1.
  2. With 10 stone in `M[9]` and 5 dirt in the cursor, send a throw click with the button 0.
  3. Send a throw click on the empty `M[9]`.

  Results: the first throw leaves 9 stone and makes an item entity of 1 stone. The second throw
  empties the slot and makes an item entity of 9 stone. The throws of the steps 2 and 3 make no item
  entity and change no slot.

- **E14** (probe, run; `transactions_e14`). Rules: ITX-JAV-014, ITX-JAV-015, ITX-JAV-016,
  ITX-JAV-017, ITX-JAV-018, ITX-JAV-019, deviation D1. Before each sequence, empty the main slots
  and hold 5 stone. The start packet, the add packet and the end packet have the buttons 0, 1 and 2.
  1. Send start, add `M[9]`, `M[10]`, `M[11]`, end.
  2. Send add `M[9]`, `M[10]`, `M[11]`, end, with no start.
  3. Send start, add `M[9]`, start, add `M[10]`, `M[11]`, end.
  4. Send start, add `M[9]`, `M[10]`, a pick up click on `M[20]`, add `M[11]`, end.
  5. Send start, add `M[9]` through `M[15]`, end.
  6. Send start, add `M[9]`, `M[9]`, `M[10]`, end.
  7. With 1 dirt in `M[10]`, send start, add `M[9]`, `M[10]`, `M[11]`, end.
  8. With 64 stone in `M[10]`, send start, add `M[9]`, `M[10]`, `M[11]`, end.
  9. Send start, add `M[9]`, `M[10]`, a packet with the button 3, add `M[11]`, end.
  10. In survival mode, send the packets with the buttons 8, 9, 9 and 10 over `M[9]` and `M[10]`.
  11. Send the packets with the buttons 12, 13, 13 and 14 over `M[9]` and `M[10]`.
  12. Send start, end.
  13. Send start, add `M[9]`, add the slot index -1.

  Results: step 1 puts 1 stone in each of the three slots and leaves 2 in the cursor. Step 2
  changes nothing. Step 3 changes nothing: the second start packet resets the drag. Step 4 changes
  nothing: the pick up click gives an empty list and resets the drag. Step 5 puts 1 stone in each
  of `M[9]` through `M[13]`, and the cursor becomes empty. Step 6 puts 2 stone in `M[9]` and 2 in
  `M[10]`. Step 7 puts 2 stone in `M[9]` and 2 in `M[11]`, and the dirt stays. Step 8 puts 1 stone
  in `M[9]` and 1 in `M[11]`. The 64 stone stay, and the cursor keeps 3. The full slot is in the
  set and gets no item. Step 9 changes nothing: the packet with the button 3 resets the drag. The
  steps 10, 11 and 12 change nothing. In step 13, the server disconnects the client with the reason
  `disconnect.packetError`.

- **E15** (probe, run; `transactions_e15`). Rules: ITX-JAV-017, ITX-JAV-018. Hold 10 stone.
  1. With 62 stone in `M[10]`, send a drag of drag kind 0 over `M[9]`, `M[10]` and `M[11]`.
  2. With 62 stone in `M[10]`, send a drag of drag kind 1 over the same slots.
  3. Send a drag of drag kind 0 over `M[9]` only. Then send a drag of drag kind 1 over `M[9]`
     only. Then send a drag of drag kind 0 over `M[10]` with 62 stone only.
  4. Hold 1 stone and send a drag of drag kind 0 over `M[9]` and `M[10]`. Hold 2 stone and send it
     over `M[9]`, `M[10]` and `M[11]`.

  Results: step 1 puts 3 stone in `M[9]`, fills `M[10]` to 64, puts 3 stone in `M[11]`, and leaves
  2 in the cursor. Step 2 puts 1 stone in `M[9]` and in `M[11]`, gives `M[10]` 63, and leaves 7.
  The drags of step 3 give the pick up click of the button that is the drag kind. `M[9]` takes 10
  stone, then 1 stone. `M[10]` fills to 64 and leaves 8. The drag with 1 stone puts it in `M[9]`;
  the drag with 2 stone puts 1 in `M[9]` and 1 in `M[10]`.

- **E17** (recorder, run). The scenario `inventory_e17` of
  `tools/mcrs_minecraft_client_recorder/scenarios/` gives the steps. The proxy removes each
  `player_action` packet, so the server never gets it. In each part, `P[0]` is the selected slot.
  The input agent makes each click with no mouse press and no mouse release.

  Part 1 (`inventory_e17_drop`):
  1. Put 10 stone in `P[0]`.
  2. Press the drop key once.
  3. Open the inventory.
  4. Click with the primary button on `M[36]`.
  5. Close the inventory.

  Part 2 (`inventory_e17_swap`):
  1. Put an iron helmet in `P[0]`.
  2. Put a shield in `P[40]`.
  3. Press the swap key once.
  4. Open the inventory.
  5. Click with the primary button on `M[45]`.
  6. Close the inventory.

  Results of part 1: the client sends status 5, and the click carries a cursor of 9 stone. Thus
  the client removes the dropped item from `P[0]` before the answer (ITX-JAV-025). The server
  holds 10 stone, so it sends the cursor stack (ITX-JAV-004).

  Results of part 2: the client sends status 7, and the click carries the shield from `M[45]`.
  Thus the client does not exchange the two slots. The server sends no correction.

  At the close of each part, the cursor stack goes back as INV-CLS-001 states. In part 2, the
  shield goes to `P[1]`, not to the empty off-hand slot (INV-INS-001).

## 11. Unresolved

- **Q2**: ITX-MOD-004 and ITX-JAV-026. Is 128 the maximum for the number of operations? On a slot
  that is not a result slot, the repeat gives at most one `split` to each destination. On a result
  slot, each pass also takes a new result, so one click can need more than 128 operations. Does
  such a click use one request for each pass? The specification of the result slots owns the
  answer.
- **Q5**: ITX-BED-001. The public documentation gives the fields of each action and not its
  conditions. Does a `Take` to a slot that holds a different stack fail, and with which result?
  Does a `Swap` join two equal stacks?
- **U3**: ITX-JAV-008, row 7. The result of a secondary click on a result slot. A later
  specification owns the answer.
- **U7**: ITX-JAV-021, the rows for `player_action` with the status 4 and for
  `set_creative_mode_slot`. Experiment E17 records only the status 5 and the status 7. The two rows
  need a recording of the vanilla client, which the owner makes by hand.
