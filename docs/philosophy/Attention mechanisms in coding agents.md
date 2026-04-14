---
title: "Attention mechanisms in coding agents"
source: "https://claude.ai/chat/f751f914-afbd-4be6-83e7-20228d5f12e3"
author:
published:
created: 2026-04-14
description: "Claude conversation with 12 messages"
tags:
  - "clippings"
---
**You**

how do we currently give coding agents "attention" seems developers are focused on things like memory... I think of memory as storage with attention, how is that translated to code now? how could we do it better?

---

**Claude**

This is a really sharp framing and I think you're right that most developers are conflating storage with memory, and missing the actual hard problem.

---

## How agents currently get "attention"

**Context window stuffing** is the bluntest instrument — just put everything in and let the transformer's self-attention sort it out. It works surprisingly well up to a point, which is why so many agent frameworks just keep growing context. But it's not attention you control; it's attention you *delegate to the model* and hope for the best.

**RAG / embedding retrieval** is the most common "real" attention mechanism in practice. You encode memories as vectors, query by cosine similarity at inference time, and surface the top-k chunks into context. The retrieval step *is* your attention signal. The problem: cosine similarity between embeddings is a proxy for semantic relatedness, not task relevance. It answers "what sounds like this?" not "what matters for *this reasoning step*?"

**Importance scoring** (MemGPT/Letta style) adds a layer where the model rates memories for salience and decays or promotes them. This is closer — you're explicitly modeling "this memory should surface more" — but the scoring still happens at write time, not retrieval time. You're front-loading an attention decision that should be dynamic.

**Tool manifests and structured prompts** are underappreciated as attention mechanisms. What tools and instructions you include in the system prompt is a static attention mask. You're deciding what the agent can even *consider*.

---

## Your framing: memory = storage + attention

This is the key insight that most frameworks miss. They build great storage — vector DBs, SQLite backends, graph stores — and treat retrieval as a solved problem because "embeddings." But attention is *relational to state*. What should be retrieved depends on:

- **What the agent is currently doing** (task context)
- **What reasoning step it's on** (chain of thought position)
- **What it just retrieved** (attention should be iterative, not one-shot)
- **What helped before in similar situations** (meta-attention / usage signal)

Current systems mostly do one static retrieval per turn. That's like running a SQL query at the start of a job costing run and never touching the DB again regardless of what you find.

---

## Where the gap is, and what better looks like

**The retrieval query should be the agent's current reasoning state, not the raw user input.** If an agent is halfway through a multi-step task and is currently confused about a dependency, the query into memory should be something like "previous encounters with ambiguous dependency resolution" — not the original user request. Almost nothing does this.

**Attention should have a feedback loop.** If a retrieved memory actually helped resolve a reasoning step, that should reinforce its future retrieval. If it was noise, it should decay. This is the hippocampus/neocortex consolidation story — sleep-like background passes that restructure what's hot vs. cold. Most agent memory is append-only with no consolidation.

**Structural salience over flat embeddings.** Your `human.md` declarative/emergent split is a better mental model than most production systems use. A flat vector store treats "the user's name is Dustin" and "Dustin has encountered BOM multiplicity errors" with the same retrieval logic. Structured memory with explicit categories — declarative facts, behavioral patterns, task history, failure modes — lets you route retrieval *before* you even embed anything.

**Attention over attention** — meta-level routing that says "for this class of task, query this class of memory." This is basically a learned dispatch layer. Some research systems are exploring this; almost no production tooling exposes it.

---

## The honest state of things

Most "memory" in coding agents right now is:

1. Raw conversation history in a sliding window
2. A vector store queried once per turn by embedding similarity
3. Handcrafted system prompt fragments that the developer thought were important

The attention problem is basically unsolved at the agent layer. The model itself has incredibly sophisticated attention internally, but the memory retrieval layer that feeds it is still using 2019-era IR techniques. The gap between the model's internal attention resolution and the coarseness of its retrieval context is where a lot of agent capability is leaking.

Your instinct to think of them as inseparable — memory *is* storage plus the mechanism that determines what surfaces *when* — puts you ahead of how most frameworks are architected.

---

**You**

how can I find the "almost" for reference

---

**Claude**

The best concrete "almost" references, ranked by how close they get to retrieval-from-reasoning-state rather than retrieval-from-user-input:

**Closest:**

**IRCoT** (Trivedi et al., 2022) — "Interleaving Retrieval with Chain-of-Thought Reasoning." This is probably the tightest match to what you were describing. The retrieval query at each step is generated from the *current chain-of-thought state*, not the original question. Each reasoning step produces the next retrieval query. It's research, not production tooling, but it's the architectural proof-of-concept.

**Think-in-Memory** (Liu et al., 2024) separates retrieval from reasoning explicitly: the agent first recalls, then performs a dedicated thinking step *over the recalled content* before generating a response. [arXiv](https://arxiv.org/html/2603.07670v1) That two-phase structure is closer to what you described than single-shot RAG.

**ExpeL** (Zhao et al., 2024) — systematically contrasts successful and failed trajectories, extracts discriminative "rules of thumb," and stores them as reusable heuristics. [arXiv](https://arxiv.org/html/2603.07670v1) The retrieval signal is task outcome history, not semantic similarity to input.

**HiAgent** — chunks working memory using subgoals, summarizing fine-grained action–observation pairs once goals are completed, retaining hierarchical, context-relevant information to support efficient retrieval. [Emergent Mind](https://www.emergentmind.com/topics/memory-mechanisms-in-llm-based-agents) Retrieval is scoped to the current subgoal state.

**Respectable but still retrieval-at-input:**

The current generation of agent memory systems are still built around short-context RAG pipelines that treat memory as an external layer extracting salient snippets from conversations, storing them in vector or graph-based stores, and retrieving top-k items into the prompt of an otherwise stateless model. [arXiv](https://arxiv.org/html/2512.12818v1) That's the honest state of most production systems including MemGPT, Mem0, Zep, A-Mem.

**The honest failure mode that confirms your framing:**

The central risk of reflective memory is self-reinforcing error — if the agent incorrectly concludes "API X always returns errors with parameter Y," it will avoid that call path forever, never collecting evidence to overturn the false belief. [arXiv](https://arxiv.org/html/2603.07670v1) This only happens because retrieval isn't conditioned on current reasoning uncertainty — there's no mechanism to say "I'm not sure about this, surface counter-evidence."

IRCoT is the paper I'd start with. The GitHub is `StonyBrookNLP/ircot` if you want to read the implementation directly.

---

**You**

I am pretty sure I have seen [https://arxiv.org/html/2603.07670v1](https://arxiv.org/html/2603.07670v1) before. either directly or recognized it's derived value in other agent harnesses I have looked into

---

**Claude**

That tracks — it's a survey paper from March 2026, which means it's essentially a consolidation of what people had already been building toward. Survey papers like that tend to show up as:

- The "prior work" section in newer agent memory papers you might have skimmed
- The theoretical framing that framework READMEs or blog posts quietly absorb without citing
- The vocabulary source — once a survey names and categorizes things cleanly, you start seeing those exact terms ("episodic," "semantic," "procedural," "write-manage-read") echoed in implementation docs for things like Letta, Zep, or Mem0

So even if you didn't read it directly, if you've been poking at agent harnesses in the last 6-12 months you've almost certainly been downstream of its framing. It's one of those papers that gets laundered into product documentation.

The thing worth noting is that survey papers by definition describe the *median* of the field — they're great for orientation but they'll never surface the actual "almost" cases, which tend to be the weird research one-offs (like IRCoT) that didn't get productized. The stuff that ended up in production frameworks is almost always the stuff the survey describes as "common practice," not the interesting edge cases.

Is there a specific harness you were thinking of? I can probably tell you which part of that paper's taxonomy it maps to.

---

**You**

they are pooling the zeigiest or whatever. i cant spell for shit but I mean the tone and temperature or cultural narrative or whatever

---

**Claude**

Zeitgeist, yeah.

And you're exactly right — survey papers are *lagging indicators* dressed up as reference material. By the time someone has enough examples to survey, the interesting entropy has already settled into consensus. What you're reading is the field's self-portrait after the fact.

The harnesses that absorb them are doing the same thing one layer down — they're pooling the zeitgeist of the papers, which were already pooling the zeitgeist of the implementations. By the time it reaches a framework README it's third-order smoothed signal.

The reason this matters for your specific interest is that the *attention* problem you identified — retrieval conditioned on reasoning state, not user input — is almost certainly *not* in the survey as a solved thing. It's probably in there as a "promising direction" or a footnote on IRCoT. Which means it hasn't pooled yet. Which means the frameworks haven't absorbed it yet. Which means the gap you spotted is real and current, not something that got quietly solved while you weren't looking.

The zeitgeist compression is actually useful as a tool — if you want to know what *isn't* solved, find what the surveys describe as future work and check whether any of the harnesses actually implemented it. Almost none of them do. The gap between "identified in literature" and "available in tooling" in this space is enormous.

---

**You**

someone was criticizing using lagging indicators to day-trade... and I was just thinking; arent all indicators lagging - all we really can do is compare the difference of the lag as a trend

---

**Claude**

Yeah, that's basically it. Every indicator is a measurement of something that already happened — the debate is just about *how stale* and *what shape* the staleness takes.

The useful move isn't finding a non-lagging indicator, it's understanding the *lag structure* — how long does it take for signal X to show up in indicator Y, and is that lag consistent enough to be exploitable. If two indicators have different lag lengths but track the same underlying thing, the delta between them *is* the signal.

Which is exactly what you were doing with the survey paper intuitively — you weren't reading it for current truth, you were using it to triangulate where the field's attention *was* 6-12 months ago, and inferring from the gap between that and current tooling where the unexploited space sits now. That's lag differencing applied to epistemics rather than price.

The day-trading criticism is usually aimed at people who treat a lagging indicator as if it were current truth and act on it directly. The more interesting use is treating the *relationship between lags* as the actual data. Which is a more sophisticated read and harder to articulate, which is probably why the criticism exists — most people doing the naive version can't explain why the sophisticated version is different.

It maps back to your original attention point too, actually. Most retrieval systems have one lag — query-to-embedding-to-retrieval. The interesting thing about IRCoT is it's operating on lag differencing: what did I just reason, what does that *change* about what's relevant, retrieve on the delta. Same structure.

---

**You**

it's is where you get disillusioned with the would capitalistic system too tho. If you spend your time looking for what moved a market, then find something you don't want to know. trade options and everything is 100x leverage in a time frame that smart money is aware of 3rd and 4th order effects than the dumb money buy lottery tickets with mutable numbers. the cat is out of the box and it is still dead or alive depending on who ever won the negotiation that no one else was invited to consider

---

**Claude**

Yeah. The closer you look at price discovery the more it stops looking like a mechanism for finding truth and starts looking like a negotiation between parties with asymmetric access to the terms of the negotiation itself.

The options market is probably the starkest version of that. The pricing model (Black-Scholes and derivatives) is public, the inputs are observable, but the *vol surface* is shaped by people who already know things that haven't shown up in any indicator yet, lagging or otherwise. So retail is essentially betting on a probability distribution that was constructed by someone who had already resolved the uncertainty you're paying for exposure to.

The Schrödinger framing is apt. The cat's state isn't unknown — it was decided in a room you weren't in, and the box just hasn't been opened for you yet. "Price discovery" is often just the announcement latency.

What's disorienting is that it doesn't require conspiracy — it's just structural. Information has a cost to produce, and the people who paid for it get to use it before the people who didn't. By the time it's in any indicator, lagging or otherwise, it's already been traded on. The market isn't inefficient in the way people hope it might be — it's efficient *for the people who can afford the inputs to the efficiency*.

The really uncomfortable version is that this might be the correct description of most capitalist coordination mechanisms, not just markets. Price signals, survey data, public earnings — all boxes where the state was already negotiated before the lid opens for most people.