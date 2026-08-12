#!/usr/bin/env python3
"""Author + verify the Aug 12 question set.

Verification contract, stated honestly:
  * every program is compiled with a PINNED rustc
  * programs expected to compile must compile with zero errors
  * programs expected to compile are run N times; output must be BYTE-IDENTICAL
    across all runs, otherwise the question is rejected
  * programs expected NOT to compile must fail, and we record the error code
  * the recorded stdout IS the answer -- no answer is written by hand

This does NOT prove absence of undefined behaviour. No UB-category question is
in this set precisely because that claim needs Miri and Miri needs nightly.
"""

import json
import pathlib
import subprocess
import sys
import tempfile

RUNS = 5
HERE = pathlib.Path(__file__).resolve().parent
QDIR = HERE.parent / "2026-08-12" / "questions"

QUESTIONS = [
    {
        "id": "q1",
        "topic": "Ownership and Drop",
        "difficulty": 2,
        "expect": "compiles",
        "hint": "Two things decide the order: when each value goes out of scope, and what order values in the SAME scope are dropped in.",
        "source": '''struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    let _a = Noisy("a");
    let _b = Noisy("b");
    {
        let _c = Noisy("c");
    }
    println!("end of main");
}
''',
    },
    {
        "id": "q2",
        "topic": "Iterators and closures",
        "difficulty": 2,
        "hint": "Adaptors don't run on their own. Something has to pull values through -- and watch how many get pulled.",
        "expect": "compiles",
        "source": '''fn main() {
    let v = vec![1, 2, 3];
    let it = v.iter().map(|x| {
        println!("mapping {}", x);
        x * 2
    });
    println!("created");
    let total: i32 = it.take(2).sum();
    println!("total {}", total);
}
''',
    },
    {
        "id": "q3",
        "topic": "Collections",
        "difficulty": 2,
        "hint": "Read dedup's documentation in your head, then read it again. The key word is one you might skip over.",
        "expect": "compiles",
        "source": '''fn main() {
    let mut v = vec![1, 2, 2, 3, 2, 1, 1];
    v.dedup();
    println!("{:?}", v);
}
''',
    },
    {
        "id": "q4",
        "topic": "Integer arithmetic",
        "difficulty": 3,
        "hint": "Rust has two different notions of division for signed integers, and they disagree here. Which one do / and % use?",
        "expect": "compiles",
        "source": '''fn main() {
    let a: i32 = -7;
    let b: i32 = 2;
    println!("{} {}", a / b, a % b);
    println!("{} {}", a.div_euclid(b), a.rem_euclid(b));
}
''',
    },
    {
        "id": "q5",
        "topic": "Strings and UTF-8",
        "difficulty": 3,
        "hint": "len() is not a character count. And the second string may not contain the character you think it does.",
        "expect": "compiles",
        "source": '''fn main() {
    let s = "caf\\u{e9}";
    println!("{} {}", s.len(), s.chars().count());

    let t = "cafe\\u{301}";
    println!("{} {}", t.len(), t.chars().count());

    println!("{}", s == t);
}
''',
    },
    {
        "id": "q6",
        "topic": "Bindings and shadowing",
        "difficulty": 2,
        "hint": "There are two different variables called count here. Only one of them survives the loop.",
        "expect": "compiles",
        "source": '''fn main() {
    let mut count = 0;
    for i in 0..3 {
        let count = i * 10;
        println!("{}", count);
    }
    count += 1;
    println!("final {}", count);
}
''',
    },
    {
        "id": "q7",
        "topic": "Sorting",
        "difficulty": 2,
        "hint": "Two entries share a key. What does Rust's sort promise about their relative order?",
        "expect": "compiles",
        "source": '''fn main() {
    let mut v = vec![("bob", 2), ("amy", 1), ("cal", 2), ("dan", 1)];
    v.sort_by_key(|&(_, k)| k);
    println!("{:?}", v);
}
''',
    },
    {
        "id": "q8",
        "topic": "Borrow checking",
        "difficulty": 3,
        "hint": "Ask what push might need to do to the vector's storage, and what that would mean for anything already pointing into it.",
        "expect": "fails",
        "source": '''fn main() {
    let mut v = vec![1, 2, 3];
    let first = &v[0];
    v.push(4);
    println!("{}", first);
}
''',
    },
]


def rustc_version() -> str:
    return subprocess.run(
        ["rustc", "--version"], capture_output=True, text=True, check=True
    ).stdout.strip()


def verify(q: dict, workdir: pathlib.Path) -> dict:
    src = workdir / f"{q['id']}.rs"
    src.write_text(q["source"])
    binpath = workdir / q["id"]

    comp = subprocess.run(
        ["rustc", "--edition", "2021", "-o", str(binpath), str(src)],
        capture_output=True,
        text=True,
    )

    result = {**q, "compiled": comp.returncode == 0}

    if comp.returncode != 0:
        codes = sorted(
            {
                tok.split("]")[0]
                for tok in comp.stderr.split("error[")[1:]
                if "]" in tok
            }
        )
        result["error_codes"] = codes
        result["stderr_head"] = "\n".join(
            [ln for ln in comp.stderr.splitlines() if ln.strip()][:6]
        )
        result["verified"] = q["expect"] == "fails"
        result["answer"] = "does not compile"
        return result

    if q["expect"] == "fails":
        result["verified"] = False
        result["reject_reason"] = "expected a compile failure, but it compiled"
        return result

    outs = []
    for _ in range(RUNS):
        run = subprocess.run([str(binpath)], capture_output=True, text=True, timeout=10)
        if run.returncode != 0:
            result["verified"] = False
            result["reject_reason"] = f"nonzero exit {run.returncode}"
            return result
        outs.append(run.stdout)

    identical = len(set(outs)) == 1
    result["runs"] = RUNS
    result["byte_identical"] = identical
    result["verified"] = identical
    result["answer"] = outs[0]
    if not identical:
        result["reject_reason"] = "output was not byte-identical across runs"
    return result


def main() -> int:
    QDIR.mkdir(parents=True, exist_ok=True)
    version = rustc_version()
    results = []

    with tempfile.TemporaryDirectory() as td:
        workdir = pathlib.Path(td)
        for q in QUESTIONS:
            r = verify(q, workdir)
            results.append(r)
            mark = "PASS" if r["verified"] else "REJECT"
            print(f"[{mark}] {r['id']:3} {r['topic']}")
            if not r["verified"]:
                print(f"        {r.get('reject_reason', 'unknown')}")
            else:
                preview = r["answer"].replace("\n", " | ").strip()
                print(f"        -> {preview[:96]}")

    payload = {
        "rustc": version,
        "edition": "2021",
        "runs_per_question": RUNS,
        "generated_for": "Rust NYC meetup 2026-08-12",
        "questions": results,
    }
    out = QDIR.parent / "verified.json"
    out.write_text(json.dumps(payload, indent=2))

    passed = sum(1 for r in results if r["verified"])
    print(f"\n{passed}/{len(results)} verified with {version}")
    print(f"wrote {out}")
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
