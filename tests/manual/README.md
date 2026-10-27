# Manual scripts

Not collected by pytest. These drive live services and expect credentials or
infrastructure that the CI dev chain does not provide, so they are run by hand.

| Script | Needs | Run |
|---|---|---|
| `e2e_security_check.py` | Oracle API on `:8000`, Nawal API on `:8001`, dev API keys | `python tests/manual/e2e_security_check.py` |

It previously lived at `tests/test_e2e_security.py`, where it was collected as a
test module despite defining no `test_*` functions — so it contributed zero tests
and hid the fact that the checks it claims to make were never running.
