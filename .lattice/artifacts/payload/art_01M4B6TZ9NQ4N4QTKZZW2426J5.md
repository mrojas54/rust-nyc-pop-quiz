Validation, PQ-30, HEAD ed7f0d8 vs origin/main 2877149. rustc 1.96.1, flags --edition 2021 --crate-type bin --crate-name probe. Columns: pair | A exit | B exit (first error) | normalized-equal before (origin/main, scratch git-archive copy) | after (HEAD).
C1 if-let closure | 0 | 1 E0425 | True | False
C2 item into nested mod | 0 | 1 E0425 | True | False
C3 vec! before macro_rules! vec | 0 | 1 cannot find macro zz | True | False
C4 while a == b arm | 0 | 1 E0425 | True | False
C4 match s.a arm | 0 | 1 E0425 | True | False
C4 for x in xs arm | 0 | 1 E0425 | True | False
C5 I::Item | 0 | 1 E0220 | True | False
C5 T::default() | 0 | 1 E0599 | True | False
C5 m::drop (pub use std::mem::drop) | 0 | 1 E0425 | True | False
C5 m::drop (glob re-export) | 0 | 1 E0425 | True | False
C5 crate::drop (review add) | 0 | 1 E0425 | True | False
C5 Self::default() (review add) | 0 | 1 E0599 | True | False
C6 verbatim ticket pair | 1 E0601 no main | 1 syntax | True | False  [FLAG: ticket's A does not compile; tested with fn main() {} appended]
C6 + fn main() {} | 0 | 1 syntax | True | False
C7 count_ones | 0 | 1 E0599 | True | False
Mutation-guard pairs: derive + two macros (both 0, call different macros) | True | False; generic T::default with impl Default (0 | 1 E0407) | True | False; kept tail T::Out keeps trait Tr { type Out } (0 | 1 E0220) | False | False.
Every new SCOPED_THE_SAME twin compiles on both sides (0/0) and normalizes equal.
Dismissed candidates: let/param named drop reaching a nested fn: 1/1 (E0434 both) — not a false duplicate.
Known open risk, outside C1-C7 (pre-existing): struct S; impl Iterator for S { type Item = u8; .. Self::Item .. } vs type Zz: 0 | 1 E0437 | True | True.
Suites: just test-pipeline 739 passed (710 on base); just test warm 11.9 s, green. Mutations: 12/12 killed.