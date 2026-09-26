# bank/audit

Where `just bank-audit` writes its run report, `<date>.json`, and nowhere else.

**Organizer only.** The report names answer categories and how often a rule an
attendee might learn would have hit — the very thing AC-25 and G-11 keep away from
attendees. Nothing participant-facing reads this directory, and the audit checks
that (`bank_audit_references`: no file under `web/` or `room/` mentions it).

**Not committed.** The reports are gitignored. Each is derived from the bank at the
moment it ran — it records the sha256 of every question file it read — so a
committed copy would go stale against the bank it describes. Run `just bank-audit`
to get a current one.

What the report holds, and why none of it can move an answer, is in
[`../README.md`](../README.md#what-bank-audit-checks-and-why-it-can-never-touch-a-real-night).
