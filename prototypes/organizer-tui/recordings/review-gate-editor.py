#!/usr/bin/env python3
"""A non-interactive $VISUAL for the recording: appends one sentence to explains.what."""
import json, sys
path = sys.argv[-1]
data = json.load(open(path))
data["explains"]["what"] += " (Edited during validation.)"
json.dump(data, open(path, "w"), indent=2)
