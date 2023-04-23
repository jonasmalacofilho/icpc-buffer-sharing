# Buffer Sharing in Multi-Tenant Database Environment

ICPC 2023 Online Spring Challenge powered by Huawei.

## Selected submission history

| Source code | Verdict | Max. CPU | Max. RAM | Observations |
|:--- | ---:| ---:| ---:|:--- |
| 717da76a3f8e |    [500 pt][202281228] |  5709 ms | 1 MB | "Wrong answer" in all but the first test |
| e79c255e4631 |   [4447 pt][202299987] |>15000 ms | 2 MB | "Time limit exceeded" in 60% of the tests |
| 2e7ca74e6bbf |   [4916 pt][202302355] |>15000 ms | 2 MB | "Time limit exceeded" in 53% of the tests |
| 77af96a050c7 |   [2282 pt][202306609] |>15000 ms | 9 MB | "Time limit exceeded" in 83% of the tests |
| 9ef41e695560 |   [9673 pt][202337971] | 11216 ms | 2 MB | Multi-tenant LRU using a binary heap |
| b942f2e57761 |   [7069 pt][202340052] | 11044 ms | 3 MB | Add a cost model |
| 45c8da5bda64 |   [6466 pt][202343314] | 11106 ms | 3 MB | Clamp the cost model when Qactual >= Qbase |
| 95a15d73a7dc |   [8738 pt][202349459] | 10279 ms | 2 MB | Fix cost model and break ties with used |
| d637a6d2c895 |   [9352 pt][202352504] |  9890 ms | 2 MB | Probability-weighted cost minimzation |
| 3f8aca1b7733 |   [4435 pt][202404464] | 10061 ms | 2 MB | N/A |
| 0f6058e9ccb9 |   [9676 pt][202420355] | 11575 ms | 2 MB | mtLRU with reinsertion of outdated entries |
| cf30f1620020 |  [10152 pt][202442633] | 10607 ms | 2 MB | + prioritize tenants bellow qbase |
| 8dc352a698b0 |  [10152 pt][202443198] | 10436 ms | 2 MB | (check that we're deterministic) |
| c2d348a0b12a |   [7241 pt][202444079] ||| try 1000 hit rate brackets |
| 76929737dd38 |   [9871 pt][202444509] ||| try 2 hit rate brackets |
| 4988a140319e |[(10152) pt][202495737] ||| prioritize tenants bellow/at qbase |
| 5b1e4e89bde7 |   [7571 pt][202496657] ||| try 1000 qbase/qcur brackets |
| 6fc5f1a1a205 |       [N/A][203135317] | 10779 ms | 4 MB | 9 runtime errors; 5 nil points |

[203135317]: #
[202496657]: #
[202495737]: #
[202444509]: #
[202444079]: #
[202443198]: https://codeforces.com/contest/1813/submission/202443198
[202442633]: https://codeforces.com/contest/1813/submission/202442633
[202420355]: https://codeforces.com/contest/1813/submission/202420355
[202404464]: https://codeforces.com/contest/1813/submission/202404464
[202352504]: https://codeforces.com/contest/1813/submission/202352504
[202349459]: https://codeforces.com/contest/1813/submission/202349459
[202343314]: https://codeforces.com/contest/1813/submission/202343314
[202340052]: https://codeforces.com/contest/1813/submission/202340052
[202337971]: https://codeforces.com/contest/1813/submission/202337971
[202306609]: https://codeforces.com/contest/1813/submission/202306609
[202302355]: https://codeforces.com/contest/1813/submission/202302355
[202299987]: https://codeforces.com/contest/1813/submission/202299987
[202281228]: https://codeforces.com/contest/1813/submission/202281228
