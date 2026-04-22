#!/usr/bin/env python3
"""Step 2: query every pallet in the runtime for at least one real storage item."""
import sys
from collections import defaultdict

from substrateinterface import SubstrateInterface

RPC_URL = "ws://100.81.45.25:9944"

EXPECTED_CUSTOM_PALLETS = {
    "BelizeX", "Bns", "BelizeJustice", "BelizeModeration", "BelizeWhistleblower",
    "Community", "Compliance", "Consensus", "Economy", "Governance",
    "Identity", "Interoperability", "LandLedger", "Mesh", "Oracle",
    "Payroll", "Quantum", "Staking",
}


def main():
    substrate = SubstrateInterface(url=RPC_URL)
    storage_fns = substrate.get_metadata_storage_functions()

    by_mod = defaultdict(list)
    for f in storage_fns:
        if f["storage_name"].startswith(":__"):
            continue
        by_mod[f["module_name"]].append(f)

    print("Modules with real storage: {}".format(len(by_mod)))
    print("=" * 78)

    pass_ct = 0
    fail_ct = 0
    for mod in sorted(by_mod.keys()):
        entry = by_mod[mod][0]
        name = entry["storage_name"]
        kind = entry["type_class"]
        try:
            if kind == "Plain":
                result = substrate.query(mod, name)
                val = result.value if result is not None else None
                print("[ OK ] {:<22} {:<28} Plain -> {}".format(mod, name, repr(val)[:54]))
            elif kind == "Map":
                qm = substrate.query_map(mod, name, max_results=1)
                records = list(qm.records) if qm.records else []
                print("[ OK ] {:<22} {:<28} Map   -> {} rec".format(mod, name, len(records)))
            else:
                result = substrate.query(mod, name)
                print("[ OK ] {:<22} {:<28} {:<5} -> {}".format(mod, name, kind, repr(result.value)[:30]))
            pass_ct += 1
        except Exception as e:
            print("[FAIL] {:<22} {:<28} {:<5} -> {}: {}".format(mod, name, kind, type(e).__name__, str(e)[:50]))
            fail_ct += 1

    print("=" * 78)
    found = set(by_mod.keys())
    missing = EXPECTED_CUSTOM_PALLETS - found
    present = EXPECTED_CUSTOM_PALLETS & found
    print("Summary: {} ok  /  {} fail".format(pass_ct, fail_ct))
    print("Custom pallets present: {}/{}".format(len(present), len(EXPECTED_CUSTOM_PALLETS)))
    if missing:
        print("[WARN] Missing custom pallets: {}".format(sorted(missing)))
    return 0 if fail_ct == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
