"""Compare the same snapshot probe built against official and fork sources."""
import json
import subprocess
import sys


def without_provenance(value):
    if isinstance(value, list):
        return [without_provenance(item) for item in value]
    if isinstance(value, dict):
        return {key: without_provenance(item) for key, item in value.items()
                if not (key == "source" and isinstance(item, dict)
                        and "part" in item and "path" in item)}
    return value


baseline, fork, *fixtures = sys.argv[1:]
assert fixtures, "Pass at least one DOCX fixture"
for fixture in fixtures:
    official = json.loads(subprocess.check_output([baseline, fixture]))
    extended = json.loads(subprocess.check_output([fork, fixture]))
    assert official == without_provenance(extended), fixture
    print("PASS", fixture)
