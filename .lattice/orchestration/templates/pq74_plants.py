#!/usr/bin/env python3
"""PQ-74 ruling 5: every scan still detects a violation planted in a moved file.

Each plant: run the one test clean (must pass), plant, run it (must fail with the
scan's own message), restore. Scans read room/src from disk when they run, so the
built test binary is reused; only the copy.rs plant rebuilds. Exit 0 only if every
plant was caught for the expected reason and the tree is restored.
"""
import json, pathlib, re, subprocess, sys, time

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "room/src"
Q = "quiz_session"
BASE = "9c718c3877e97e1eeb54f7cb66b0779f0c1163de"
LIMIT = 600
DEADLINE = time.time() + 840


def lit(text):
    return lambda old: old + f'\nconst _: &str = "{text}";\n'


def sub(a, b):
    def go(old):
        assert a in old, f"anchor {a!r} not found"
        return old.replace(a, b, 1)
    return go


def words(old):
    return re.sub(r"\b(Middle|Revealed)\b", "Xx", old)


def covered_by_within_rooms():
    files = sorted(p.relative_to(SRC).as_posix() for p in (SRC / Q).rglob("*.rs"))
    return [f for f in files
            if not f.startswith(f"{Q}/models/") or f == f"{Q}/models/rooms.rs"]


# (test binary, test, file under room/src, change, expected text in the failure)
PLANTS = [
    ("auth", "ac65_the_member_record_is_read_for_roles_alone", f"{Q}/host/discord/http.rs",
     lit("permissions"), f"{Q}/host/discord/http.rs code reads permissions"),
    ("auth", "ac65_the_member_record_is_read_for_roles_alone", f"{Q}/host/discord/mod.rs",
     sub("struct Member {\n", "struct Member {\n    id: String,\n"), "the member record keeps roles"),
    ("auth", "ac65_the_member_record_is_read_for_roles_alone", f"{Q}/host/discord/config.rs",
     None, f"the walk found no {Q}/host/discord/config.rs"),
    ("one_room", "gap8_create_for_checks_and_inserts_in_one_critical_section", f"{Q}/host/create.rs",
     sub("self.used.contains(club, question_id)", "self.used.contains(club,question_id)"),
     "create_for lost `self.used.contains(club, question_id)`"),
    ("one_room", "gap8_create_for_checks_and_inserts_in_one_critical_section", f"{Q}/host/create.rs",
     None, "No such file"),
    ("used", "g10_the_release_transition_is_the_only_writer", f"{Q}/state_manager.rs",
     lit("self.used.append("), "left: 3"),
    ("used", "g10_the_release_transition_is_the_only_writer", f"{Q}/participant/leave.rs",
     lit("self.parting = Some("), "left: 2"),
    ("used", "g10_the_release_transition_is_the_only_writer", f"{Q}/models/used.rs",
     sub("pub(crate) fn append(", "pub fn append("), "file(\"quiz_session/models/used.rs\")"),
    ("boundary", "the_witness_is_built_in_one_place", f"{Q}/models/phase.rs",
     lit("RevealWitness {"), f"{Q}/models/phase.rs builds RevealWitness 2 times"),
    ("boundary", "the_witness_is_built_in_one_place", f"{Q}/participant/join.rs",
     lit("RevealWitness {"), f"{Q}/participant/join.rs builds a RevealWitness"),
    ("boundary", "the_vault_holds_only_the_allowed_functions_and_every_read_takes_the_witness",
     f"{Q}/models/answers.rs", sub("mod vault {\n", "mod vault {\n    pub fn leak() {}\n"),
     "the vault's functions changed"),
    ("boundary", "nothing_sealed_can_be_printed_copied_or_serialized", f"{Q}/models/rooms.rs",
     lit("Debug for Room"), f"{Q}/models/rooms.rs implements Debug for Room"),
    ("boundary", "nothing_sealed_can_be_printed_copied_or_serialized", f"{Q}/open.rs",
     lit("Serialize for Opened"), f"{Q}/open.rs implements Serialize for Opened"),
    ("boundary", "the_vault_is_touched_only_by_answers_and_the_phase_only_by_act", f"{Q}/host/create.rs",
     lit(".vault"), f"{Q}/host/create.rs reaches into the vault"),
    ("boundary", "the_vault_is_touched_only_by_answers_and_the_phase_only_by_act", f"{Q}/state_manager.rs",
     lit(".question.open("), f"{Q}/state_manager.rs opens a question"),
    ("boundary", "the_vault_is_touched_only_by_answers_and_the_phase_only_by_act", f"{Q}/participant/vote.rs",
     lit("machine ="), f"{Q}/participant/vote.rs assigns a machine"),
    ("boundary", "the_vault_is_touched_only_by_answers_and_the_phase_only_by_act", f"{Q}/models/rooms.rs",
     lit("self.machine ="), "left: 2"),
    ("boundary", "the_pre_reveal_builders_never_see_the_opened_room", f"{Q}/models/view.rs",
     sub("mod sealed {\n", 'mod sealed {\n    const _: &str = "Opened";\n'),
     'the pre-reveal builders mention "Opened"'),
    ("boundary", "within_rooms_only_open_rs_names_the_sealed_answer_types", f"{Q}/open.rs",
     words, f"{Q}/open.rs no longer names them"),
    ("boundary", "lifecycle_docs_name_a_reaper_that_exists", f"{Q}/lifecycle.rs",
     sub("server::reaper::REAP_EVERY", "server::reaper::GONE"), "lifecycle.rs no longer mentions REAP_EVERY"),
    # The common walk: a new file in a new, deep folder is read without being named.
    ("boundary", "the_witness_is_built_in_one_place", f"{Q}/host/discord/planted/deep.rs",
     lit("RevealWitness {"), f"{Q}/host/discord/planted/deep.rs builds a RevealWitness"),
    ("admin", "ac101_g9_no_other_module_reads_the_token", f"{Q}/participant/join.rs",
     lit("AdminToken"), f"{Q}/participant/join.rs names the admin token"),
    ("lifecycle", "ac57_no_participant_identity_in_any_record", f"{Q}/participant/join.rs",
     lambda old: old + "\nstruct Planted {\n    nickname: String,\n}\n", 'a field named "nickname"'),
    ("twins", "the_proposed_block_is_copy_js_s_and_the_room_sends_none_of_it", f"{Q}/participant/vote.rs",
     lit("copy::PROPOSED_HOME_NOTHING_YET"), f"{Q}/participant/vote.rs uses copy::PROPOSED_HOME_NOTHING_YET"),
    ("no_fallback_route", "no_route_names_a_sheet_or_a_fallback", f"{Q}/host/create.rs",
     lambda old: old + '\nconst _: &str = stringify!(.route("/fallback", x));\n',
     "a room route serves the host sheet or the fallback"),
]
PLANTS += [("boundary", "within_rooms_only_open_rs_names_the_sealed_answer_types", f,
            lit(" Middle "), f"{f} names Revealed or Middle")
           for f in covered_by_within_rooms() if f != f"{Q}/open.rs"]


def out(line=""):
    print(line, flush=True)


def build():
    names = sorted({p[0] for p in PLANTS})
    args = ["cargo", "test", "--offline", "--locked", "--no-run", "--message-format=json"]
    for n in names:
        args += ["--test", n]
    done = subprocess.run(args, cwd=ROOT / "room", capture_output=True, text=True, timeout=LIMIT)
    if done.returncode:
        sys.exit("build failed\n" + done.stderr[-2000:])
    bins = {}
    for line in done.stdout.splitlines():
        m = json.loads(line)
        if m.get("reason") == "compiler-artifact" and m.get("executable") and m["target"]["name"] in names:
            bins[m["target"]["name"]] = m["executable"]
    return bins


def run(exe, test):
    r = subprocess.run([exe, "--exact", test], cwd=ROOT / "room", capture_output=True, text=True, timeout=LIMIT)
    return r.returncode, r.stdout + r.stderr


def plant_once(exe, test, rel, change, want):
    rc, _ = run(exe, test)
    if rc:
        return "CONTROL-FAILED", ""
    path = SRC / rel
    new_file = not path.exists()
    old = "" if new_file else path.read_text()
    try:
        if change is None:
            path.unlink()
        else:
            path.parent.mkdir(exist_ok=True)
            path.write_text(change(old))
        rc, text = run(exe, test)
    finally:
        if new_file:
            path.unlink()
            path.parent.rmdir()
        else:
            path.write_text(old)
    if rc == 0:
        return "NOT-CAUGHT", ""
    if want not in text:
        return "WRONG-REASON", text[-600:]
    return "caught", ""


def copy_plant():
    # copy::ALL is compiled in, so this one is a real rebuild through cargo.
    path = SRC / "copy.rs"
    old = path.read_text()
    cmd = ["cargo", "test", "--offline", "--locked", "--test", "twins", "--",
           "--exact", "the_copy_module_mirrors_copy_js_in_both_directions"]
    try:
        path.write_text(sub('"Time for a pop quiz."', '"Time for a pop quiz!"')(old))
        r = subprocess.run(cmd, cwd=ROOT / "room", capture_output=True, text=True, timeout=LIMIT)
    finally:
        path.write_text(old)
    text = r.stdout + r.stderr
    if r.returncode and "room/src/copy.rs has drifted from web/shared/copy.js" in text:
        return "caught", ""
    return "NOT-CAUGHT", text[-600:]


def filter_coverage():
    listed = subprocess.run(["git", "ls-tree", "-r", "--name-only", BASE, "room/src/rooms/"],
                            cwd=ROOT, capture_output=True, text=True, check=True).stdout.split()
    moved = {"mod.rs": [f"{Q}/mod.rs"], "state.rs": [f"{Q}/state_manager.rs"], "room.rs": [f"{Q}/models/rooms.rs"],
             "create.rs": [f"{Q}/host/create.rs"],
             "participants.rs": [f"{Q}/participant/{n}.rs" for n in ("mod", "join", "vote", "leave", "buzzer", "counts", "tokens")]}
    new = set(covered_by_within_rooms())
    target = []
    for f in listed:
        name = f.removeprefix("room/src/rooms/")
        target += moved.get(name, [f"{Q}/{name}"])
    missing = sorted(set(target) - new)
    out(f"filter coverage: old filter read {len(listed)} files at base; their {len(target)} new homes; "
        f"new filter reads {len(new)} files; old homes not read: {missing or 'none'}")
    out(f"added by the new filter: {sorted(new - set(target))}")
    return not missing


def main():
    ok = filter_coverage()
    bins = build()
    out(f"{len(PLANTS) + 1} plants")
    for binary, test, rel, change, want in PLANTS:
        if time.time() > DEADLINE:
            out("DEADLINE: stopped early")
            ok = False
            break
        kind = "delete" if change is None else "plant"
        verdict, detail = plant_once(bins[binary], test, rel, change, want)
        ok &= verdict == "caught"
        out(f"{verdict:12} {kind:6} {rel} -> {binary}::{test} [{want}]")
        if detail:
            out(detail)
    verdict, detail = copy_plant()
    ok &= verdict == "caught"
    out(f"{verdict:12} plant  copy.rs -> twins::the_copy_module_mirrors_copy_js_in_both_directions")
    if detail:
        out(detail)
    dirty = subprocess.run(["git", "status", "--short", "--", "room"], cwd=ROOT, capture_output=True, text=True).stdout
    out(f"tree after restore: {dirty.strip() or 'clean'}")
    ok &= not dirty.strip()
    out("RESULT " + ("PASS" if ok else "FAIL"))
    sys.exit(0 if ok else 1)


main()
