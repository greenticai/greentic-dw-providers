# memory/short-term

Shared short-term memory provider contract.

This family now contains one shared contract crate plus two backend variants:

- core
- in-memory
- redis

The shared crate owns the short-term memory trait, config surface, and `StateStore`-backed
implementation logic. Both backend crates implement the same short-term memory capability
contract and expose the same pack-level metadata.
