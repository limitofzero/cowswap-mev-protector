# CoW MEV Defense

A tower-defense game set inside the Ethereum mempool. Protect your transactions from MEV bots using CoW Protocol mechanisms as towers.

![CoW MEV Defense](assets/cowswap_tower_batch.png)

---

## Concept

Transactions travel along the mempool path from entry to settlement. MEV bots chase them down and drain their value. Place CoW Protocol towers to protect transactions, slow bots, and settle as much value as possible before it is extracted.

---

## How to Play

| Action | Input |
|---|---|
| Place tower | Click a shop button, then left-click on the map |
| Cancel placement | Right-click or `Escape` |
| Upgrade tower | Hover a tower, left-click when the upgrade preview appears |
| Remove tower | Click the remove button, then left-click on a tower |
| Pause / unpause | `Space` |

**Win condition:** Survive all waves with your balance above zero.  
**Lose condition:** Your COW balance is depleted by extracted value.

---

## Economy

- **Starting balance:** 300 COW
- **Income:** Each settled transaction pays a 1% fee on its remaining value. A 1 000 COW transaction that settles with 800 COW remaining earns you 8 COW.
- **Costs:** Placing and upgrading towers, and removing them (10 COW to remove).
- Displayed prices convert at **1 COW = $0.15** for reference, but all game math is in COW.

---

## Transactions

Transactions spawn at the mempool entry and travel toward settlement. Each carries a value denominated in COW.

| Token | COW rate | Amount range | Value range |
|---|---|---|---|
| ETH | 5 000 | 0.05 – 5.0 | 250 – 25 000 COW |
| WBTC | 100 000 | 0.001 – 0.5 | 100 – 50 000 COW |
| USDT / USDC / DAI | 2 | 100 – 5 000 | 200 – 10 000 COW |
| COW | 1 | 100 – 10 000 | 100 – 10 000 COW |

**Travel speed** depends on network load. Bots that reach a transaction drain its value at their drain rate per second. When a transaction's value hits zero, it is considered fully extracted.

### Network Load

| Level | Label | Tx speed | Spawn interval |
|---|---|---|---|
| 0 | LOW | 1.00× | 3.0 s |
| 1 | BUSY | 0.90× | 2.0 s |
| 2 | HIGH | 0.75× | 1.0 s |

From wave 4 onward, load shifts every block: 50 % chance to increase, 25 % to stay, 25 % to decrease.

---

## Enemies

All bots spawn at one of eight zones around the map and path toward the nearest unprotected transaction.

### Types

| Bot | HP | Speed | Drain/s | Attack range | Description |
|---|---|---|---|---|---|
| **Frontrunner** | 60 | 130 | 12 % | 65 | Fast, moderate drain. The most common early threat. |
| **Backrunner** | 100 | 55 | 8 % | 65 | Slow and tanky. Hard to kill before it latches on. |
| **SandwichBot** | 80 | 90 | 18 % | 65 | Balanced but drains hard — prioritize with Solver. |
| **JitLp** | 50 | 160 | 22 % | 40 | Fastest bot in the game. Short range but devastating drain. |

### Leveled Bots

Starting from wave 8, elite variants appear with increased stats. Level multipliers apply to HP, speed, and drain rate simultaneously.

| Level | Speed | HP | Drain | Visual size | First appears |
|---|---|---|---|---|---|
| Lv 0 | 1.0× | 1.0× | 1.0× | 48 px | Wave 1 |
| Lv 1 | 1.35× | 1.8× | 1.5× | 58 px | Wave 8 |
| Lv 2 | 1.7× | 3.2× | 2.3× | 67 px | Wave 20 |
| Lv 3 | 2.1× | 5.5× | 3.5× | 77 px | Wave 28 |

Example — **SandwichBot Lv 3:** 189 speed · 440 HP · 63 % drain per second.

---

## Waves

A new wave (block) arrives every 15 seconds. The first wave starts 5 seconds after the game begins.

- Active enemy cap: 2 for waves 1–2, then grows with wave number up to 20.
- Individual bots spawn every 2.5 seconds within a wave.

### Elite bot quota per wave

| Waves | Lv 1 | Lv 2 | Lv 3 |
|---|---|---|---|
| 1 – 7 | 0 | 0 | 0 |
| 8 – 11 | 1 | 0 | 0 |
| 12 – 15 | 2 | 0 | 0 |
| 16 – 19 | 3 | 0 | 0 |
| 20 – 23 | 3 | 1 | 0 |
| 24 – 27 | 4 | 2 | 0 |
| 28 – 31 | 4 | 3 | 1 |
| 32 – 35 | 4 | 4 | 2 |
| 36 – 39 | 3 | 4 | 3 |
| 40 – 43 | 2 | 4 | 4 |
| 44 – 47 | 0 | 3 | 6 |
| 48 – 51 | 0 | 1 | 10 |
| 52+ | 0 | 0 | 20 |

---

## Towers

Five towers, each modeled on a real CoW Protocol mechanism. Click a tower in the shop bar to enter placement mode. Towers cannot be placed on or too close to the mempool path, or within 40 px of another tower.

### Placement constraints

| Rule | Distance |
|---|---|
| Min distance from path | 46 px |
| Min distance between towers | 40 px |
| Remove cost | 10 COW |

---

### CoW Matcher `CoW`

> *"Finds matching orders and grants MEV immunity for 6s."*

| Stat | Value |
|---|---|
| Cost | 200 COW |
| Range | 110 px |
| Cooldown | 3.5 s |

Every activation pairs up to **2 transactions** within range and marks them immune for **6 seconds** — bots cannot drain immune transactions.

**Upgrades**

| Level | Cost | Effect |
|---|---|---|
| Lv 1 | 100 COW | +10 % drain resistance on matched txs |
| Lv 2 | 150 COW | +20 % drain resistance |
| Lv 3 | 225 COW | +30 % drain resistance |

---

### Batch Auctioneer `BA`

> *"Batches nearby txs together. Each extra tx dilutes enemy drain."*

| Stat | Value |
|---|---|
| Cost | 150 COW |
| Range | 130 px |
| Cooldown | 2.5 s |

Groups all in-range transactions into a single batch. A bot attacking a batched transaction only extracts `1 / batch_size` of its normal drain — four batched txs means 25 % drain effectiveness.

**Upgrades**

| Level | Cost | Effect |
|---|---|---|
| Lv 1 | 75 COW | −0.3 s cooldown (2.5 → 2.2 s) |
| Lv 2 | 112 COW | −0.6 s cooldown (2.5 → 1.9 s) |
| Lv 3 | 169 COW | −0.9 s cooldown (2.5 → 1.6 s) |

---

### Solver `SLV`

> *"Fires projectiles at bots to reduce their HP."*

| Stat | Value |
|---|---|
| Cost | 180 COW |
| Range | 85 px |
| Cooldown | 1.5 s |

Fires a **homing projectile** at the nearest bot in range. Deals **50 HP** base damage on contact. The only tower that directly destroys bots.

**Upgrades**

| Level | Cost | Effect |
|---|---|---|
| Lv 1 | 90 COW | +10 % damage (55 HP) |
| Lv 2 | 135 COW | +20 % damage (60 HP) |
| Lv 3 | 202 COW | +30 % damage (65 HP) |

---

### Slippage Guard `SG`

> *"Slows enemies inside its range down to 35 % movement speed."*

| Stat | Value |
|---|---|
| Cost | 130 COW |
| Range | 95 px |
| Cooldown | 0.8 s |

Applies a slow debuff to every bot in range, reducing their movement speed to **35 %** for **3 seconds**. Best paired with a Solver to give it time to shoot slowed bots.

**Upgrades**

| Level | Cost | Effect |
|---|---|---|
| Lv 1 | 65 COW | Slow to 25 % speed (+10 % intensity) |
| Lv 2 | 97 COW | Slow to 15 % speed (+20 % intensity) |
| Lv 3 | 146 COW | Slow to 10 % speed (+30 % intensity) |

---

### Dark Pool Node `DP`

> *"Hides txs from bots with a dark pool shield for 4s."*

| Stat | Value |
|---|---|
| Cost | 220 COW |
| Range | 75 px |
| Cooldown | 10.0 s |

Grants **immunity** to all transactions in range for **4 seconds**. High impact but long cooldown — position it to protect high-value transactions.

**Upgrades**

| Level | Cost | Effect |
|---|---|---|
| Lv 1 | 110 COW | −0.5 s cooldown (10.0 → 9.5 s) |
| Lv 2 | 165 COW | −0.7 s additional (→ 8.8 s) |
| Lv 3 | 247 COW | −0.9 s additional (→ 7.9 s) |

---

### Upgrade formula

`upgrade_cost = base_cost × 0.5 × 1.5^current_level`

After placing or upgrading a tower there is a **2.5 s lock** before the next upgrade can be purchased (prevents misclicks).

---

## Building

```
cargo run --release
```

Requires Rust stable. Powered by [Bevy](https://bevyengine.org/).
