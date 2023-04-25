# Buffer Sharing in Multi-Tenant Database Environment

ICPC 2023 Online Spring Challenge powered by Huawei.

## Selected submission history

| Source code | Submission | Verdict | Max. CPU | Max. RAM | Observations |
|:--- |:---:| ---:| ---:| ---:|:--- |
| 717da76a3f8e | 202281228 |    500 pt |  5709 ms | 1 MB | "Wrong answer" in all but the first test |
| e79c255e4631 | 202299987 |   4447 pt |>15000 ms | 2 MB | "Time limit exceeded" in 60% of the tests |
| 2e7ca74e6bbf | 202302355 |   4916 pt |>15000 ms | 2 MB | "Time limit exceeded" in 53% of the tests |
| 77af96a050c7 | 202306609 |   2282 pt |>15000 ms | 9 MB | "Time limit exceeded" in 83% of the tests |
| 9ef41e695560 | 202337971 |   9673 pt | 11216 ms | 2 MB | Multi-tenant LRU using a binary heap |
| b942f2e57761 | 202340052 |   7069 pt | 11044 ms | 3 MB | Add a cost model |
| 45c8da5bda64 | 202343314 |   6466 pt | 11106 ms | 3 MB | Clamp the cost model when Qactual >= Qbase |
| 95a15d73a7dc | 202349459 |   8738 pt | 10279 ms | 2 MB | Fix cost model and break ties with used |
| d637a6d2c895 | 202352504 |   9352 pt |  9890 ms | 2 MB | Probability-weighted cost minimzation |
| 3f8aca1b7733 | 202404464 |   4435 pt | 10061 ms | 2 MB | N/A |
| 0f6058e9ccb9 | 202420355 |   9676 pt | 11575 ms | 2 MB | mtLRU with reinsertion of outdated entries |
| cf30f1620020 | 202442633 |  10152 pt | 10607 ms | 2 MB | + prioritize tenants bellow qbase |
| 8dc352a698b0 | 202443198 |  10152 pt | 10436 ms | 2 MB | (check that we're deterministic) |
| c2d348a0b12a | 202444079 |   7241 pt |      N/A |  N/A | try 1000 hit rate brackets |
| 76929737dd38 | 202444509 |   9871 pt |      N/A |  N/A | try 2 hit rate brackets |
| 4988a140319e | 202495737 |(10152) pt |      N/A |  N/A | prioritize tenants bellow/at qbase |
| 5b1e4e89bde7 | 202496657 |   7571 pt |      N/A |  N/A | try 1000 qbase/qcur brackets |
| 6fc5f1a1a205 | 203135317 |       N/A | 10779 ms | 4 MB | 9 runtime errors; 5 nil points |
| c70b3a760658 | 203378938 |   9202 pt | 10716 ms | 4 MB | ARC |
| e59ccd0128ba | 203380111 |   9244 pt |      N/A |  N/A | ARC + qmin tuning to prio × 10% |
|(e59ccd0128ba)| 203380128 |   9206 pt |      N/A |  N/A | ARC + qmin tuning to prio × 5% |
|(e59ccd0128ba)| 203380416 |   9173 pt |      N/A |  N/A | ARC + qmin tuning to prio × 15% |
