# greentic-dw-memory-long-term

Long-term (episodic) memory contract for the Greentic Agentic Worker runtime:
the `LongTermMemory` trait and its DTOs. Lightweight — the heavy Chronicle
backend implementation is injected at the runner-host edge, never compiled
into this crate.
