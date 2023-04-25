# Buffer Sharing in Multi-Tenant Database Environment

ICPC 2023 Online Spring Challenge powered by Huawei.

## Selected submission history

| Source code | Submission | Verdict | Observations |
|:--- |:---:| ---:|:--- |
| 717da76a3f8e | 202281228 |    500 pt | "Wrong answer" in all but the first test |
| e79c255e4631 | 202299987 |   4447 pt | "Time limit exceeded" in 60% of the tests |
| 2e7ca74e6bbf | 202302355 |   4916 pt | "Time limit exceeded" in 53% of the tests |
| 77af96a050c7 | 202306609 |   2282 pt | "Time limit exceeded" in 83% of the tests |
| 9ef41e695560 | 202337971 |   9673 pt | Multi-tenant LRU using a binary heap |
| b942f2e57761 | 202340052 |   7069 pt | Add a cost model |
| 45c8da5bda64 | 202343314 |   6466 pt | Clamp the cost model when Qactual >= Qbase |
| 95a15d73a7dc | 202349459 |   8738 pt | Fix cost model and break ties with used |
| d637a6d2c895 | 202352504 |   9352 pt | Probability-weighted cost minimzation |
| 3f8aca1b7733 | 202404464 |   4435 pt | N/A |
| 0f6058e9ccb9 | 202420355 |   9676 pt | mtLRU with reinsertion of outdated entries |
| cf30f1620020 | 202442633 |  10152 pt | + prioritize tenants bellow qbase |
| 8dc352a698b0 | 202443198 |  10152 pt | (check that we're deterministic) |
| c2d348a0b12a | 202444079 |   7241 pt | try 1000 hit rate brackets |
| 76929737dd38 | 202444509 |   9871 pt | try 2 hit rate brackets |
| 4988a140319e | 202495737 |(10152) pt | prioritize tenants bellow/at qbase |
| 5b1e4e89bde7 | 202496657 |   7571 pt | try 1000 qbase/qcur brackets |
| 6fc5f1a1a205 | 203135317 |       N/A | 9 runtime errors; 5 nil points |
| c70b3a760658 | 203378938 |   9202 pt | ARC |
| e59ccd0128ba | 203380111 |   9244 pt | ARC + qmin tuning to prio ÷ 10 × 10% |
|(e59ccd0128ba)| 203380128 |   9206 pt | ARC + qmin tuning to prio ÷ 10 × 5% |
|(e59ccd0128ba)| 203380416 |   9173 pt | ARC + qmin tuning to prio ÷ 10 × 15% |
| 7f50e2096e05 | 203381995 |   9215 pt | ARC + qmin tuning to prio ÷ total_prio × 20% |
| 22c0b4c2aeb7 | 203382418 |   9152 pt | ARC + prioritize tenants bellow qbase |
| dc9b8bcb1185 | 203382608 |   9171 pt | ARC + qmin tuning + prioritize <qbase |
