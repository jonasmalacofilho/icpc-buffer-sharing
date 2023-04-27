# Buffer Sharing in Multi-Tenant Database Environment

ICPC 2023 Online Spring Challenge powered by Huawei.[^1][^2]

## Selected submission history

| Source code | Submission | Verdict | Notes |
|:--- |:---:| ---:|:--- |
| [`717da76a3f8e`] | [202281228] |   500.0 pt | "Wrong answer" in all but the first test |
| [`e79c255e4631`] | [202299987] |  4447.4 pt | "Time limit exceeded" in 60% of the tests |
| [`0f6058e9ccb9`] | [202420355] |  9676.0 pt | Multi-tenant N×LRU using a binary heap |
| [`cf30f1620020`] | [202442633] | 10152.0 pt | N×LRU + donate from Q <= Qbase then LRU |
| [`4988a140319e`] | [202495737] | 10151.8 pt | N×LRU + donate from Q < Qbase then LRU |
| [`c70b3a760658`] | [203378938] |  9202.4 pt | N×ARC |
| [`e59ccd0128ba`] | [203380111] |  9244.0 pt | N×ARC + Qmin tuning to prio ÷ 10 × 10% |
| [`5b141230745c`] | [203383211] | 10152.0 pt | `cf30f1620020` (N×LRU) + Qmin tuning (10%) |
| [`aa09bec94606`] | [203647749] |  5796.6 pt | N×LFU |
| **[`bdbdfd50532b`]** | **[203649425]** | **10151.8 pt** | **Final: N×LRU + donate from Q < Qbase then LRU** |

The full submission history can be seen in my [Codeforces submission history].

[^1]: https://codeforces.com/contest/1813/problem/A
[^2]: https://codeforces.com/blog/entry/112838

[Codeforces submission history]: https://codeforces.com/submissions/jonasmalacofilho

[202281228]: https://codeforces.com/contest/1813/submission/202281228
[202299987]: https://codeforces.com/contest/1813/submission/202299987
[202420355]: https://codeforces.com/contest/1813/submission/202420355
[202442633]: https://codeforces.com/contest/1813/submission/202442633
[202495737]: https://codeforces.com/contest/1813/submission/202495737
[203378938]: https://codeforces.com/contest/1813/submission/203378938
[203380111]: https://codeforces.com/contest/1813/submission/203380111
[203383211]: https://codeforces.com/contest/1813/submission/203383211
[203647749]: https://codeforces.com/contest/1813/submission/202420355
[203649425]: https://codeforces.com/contest/1813/submission/203649425

[`717da76a3f8e`]: https://github.com/jonasmalacofilho/icpc-buffer-sharing/blob/717da76a3f8e/src/main.rs
[`e79c255e4631`]: https://github.com/jonasmalacofilho/icpc-buffer-sharing/blob/e79c255e4631/src/main.rs
[`0f6058e9ccb9`]: https://github.com/jonasmalacofilho/icpc-buffer-sharing/blob/0f6058e9ccb9/src/main.rs
[`cf30f1620020`]: https://github.com/jonasmalacofilho/icpc-buffer-sharing/blob/cf30f1620020/src/main.rs
[`4988a140319e`]: https://github.com/jonasmalacofilho/icpc-buffer-sharing/blob/4988a140319e/src/main.rs
[`c70b3a760658`]: https://github.com/jonasmalacofilho/icpc-buffer-sharing/blob/c70b3a760658/src/main.rs
[`e59ccd0128ba`]: https://github.com/jonasmalacofilho/icpc-buffer-sharing/blob/e59ccd0128ba/src/main.rs
[`5b141230745c`]: https://github.com/jonasmalacofilho/icpc-buffer-sharing/blob/5b141230745c/src/main.rs
[`aa09bec94606`]: https://github.com/jonasmalacofilho/icpc-buffer-sharing/blob/aa09bec94606/src/main.rs
[`bdbdfd50532b`]: https://github.com/jonasmalacofilho/icpc-buffer-sharing/blob/bdbdfd50532b/src/main.rs
