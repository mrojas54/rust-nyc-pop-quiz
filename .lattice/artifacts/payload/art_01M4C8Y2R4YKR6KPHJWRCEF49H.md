Addendum to Validation, HEAD 6c589fa vs ed7f0d8. rustc 1.96.1 --edition 2021 --crate-type bin. pair | A | B | normalized-equal at ed7f0d8 | at HEAD.
R1 type Item in impl Iterator | 0 | 1 E0437 | True | False
R2 B::try_from beside inherent A::try_from | 0 | 1 E0599 | True | False
R3 Self::try_from in impl From<u8> for B | 0 | 1 E0599 | True | False
Guard pair: generic T beside struct T with fn default | 0 | 1 E0599 | False | False (pins the C5 generic guard, which 8a made redundant for its old pair).
Twins (SCOPED_THE_SAME): program trait impl go->run 0/0 equal; each type's own associated fns via path and Self 0/0 equal.
All 15 earlier C1-C7 pairs still unequal. Mutations 15/15 killed (12 earlier + 8a library-impl, 8b type-owner, 8b Self-owner). just test-pipeline 745 passed; just test warm 11.1 s green.