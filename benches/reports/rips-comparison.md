# Cocycle, GUDHI and Ripser: Rips comparison

[Benchmarks](../README.md) / [Reports](README.md)

This is a two-round, single-host before/after comparison of the merged Rust
revisions, plus native GUDHI and upstream Ripser resource measurements. It updates
the maintained report in place. All numbers below come from these runs; no
historical PR timing is relabeled as a measurement of the current revision.

Prepared F2 H0-through-H1 workloads improve consistently: the nine-case,
equal-case geometric-mean time reduction is **21.77% in round 1 and 21.84% in
round 2**. Uniform H1 at 128 vertices drops from 3.975 to 3.070 ms and its observed
peak process RSS drops from 3668 to 2944 KiB. Ripser remains faster on most of the
shown H1 workloads. Several generic-field and explicit-complex workflows become
slower; sparse approximation has substantial run-to-run variation. There is no
project-wide speedup claim.

## Measured revisions and protocol

| Identity | Value |
| --- | --- |
| Baseline | `a201027682c4c8e720e951069b93a159d305669b`: merged [PR #4](https://github.com/Aequiludium/cocycle-rs/pull/4), before #5/#6 |
| Candidate | `4d7178a1c474b92cd76db4d86ea375037d307141`: main after [PR #6](https://github.com/Aequiludium/cocycle-rs/pull/6) and [PR #5](https://github.com/Aequiludium/cocycle-rs/pull/5) |
| Harness | Each checkout uses its own recorded commit above; the measured Rips controllers, workers and fixture generation are unchanged between them |
| GUDHI | `cba915e3ab8e1f5b1fe26eb44b407285f7af4e78` |
| Ripser | `01add51ff64aaf40889483260cc5c3b7d0f2a1e7` |
| Host | AMD EPYC 7H12; x86_64 Linux 5.15.0-185-generic; glibc 2.35 |
| Build | Rust 1.92.0; Cargo release and optimized workers; g++ 11.4.0, C++17 `-O3 -DNDEBUG`; no custom `RUSTFLAGS` |
| Controls | Serial fresh worker processes pinned to CPU 2; 2048 MiB address-space cap; no frequency, thermal or competing-host-load control |
| Sampling | Two rounds, 12 measured processes per backend/input/revision in each round; 24 samples per reported cell; no outlier removal |
| Version order | Round 1: baseline then candidate; round 2: candidate then baseline, separately for each suite |
| Prepared H0/H1 | `cocycle-native-v1`; no warmup; shuffled backend order seed 20260920; 60 s process timeout |
| Complete pipeline | `cocycle-rips-pipeline-v2`; one discarded warmup per backend/input/run; position-balanced backend order, seeds 7919 and 15838; 30 s timeout |

Both Rust checkouts were clean before and after every run. Source fingerprints
were unchanged within each run and identical across repeated runs of each
revision/suite. Corresponding fixture hashes matched across all four runs of each
suite. Each C++ reference executable was byte-identical across all four runs of
its suite. The two suites still have different workers and timing boundaries.

The baseline/candidate comparison includes both #5 and #6. It is not a separate
attribution experiment for each PR. #6 changes the specialized F2 H1 engine;
#5 adds diagram distances, whose performance is **not measured here**. Topp,
CGAL and the distance reference environment were not installed on this host.
No submitted PR's older performance numbers are used in these tables.

## What is compared

All timed computation executes in Rust or C++ processes. Python only orchestrates
these Rips suites. Native execution avoids Python-wrapper overhead; it does not
remove the conversions and copies required by each documented native protocol.
Prepared-input fixtures are shared float32-exact values; Cocycle/GUDHI store f64
and upstream Ripser uses f32. These widths affect time and memory interpretation.

The [H0/H1 protocol](../protocol.md) times prepared native input through owned,
normalized intervals, excluding process startup, fixture parsing and final output
destruction. An H1 request computes H0 through H1, not an isolated H1 inner loop.
The [pipeline protocol](../pipeline/README.md) includes public validation,
construction, optional expansion, computation and interval payload export,
excluding fixture parsing and final metrics transport. Do not pool these suites.

Reference workers that perform less work, such as diagram-only references for
representatives or precomputed input for point construction, remain correctness
references and are excluded from workflow rankings. Rust explicit complexes retain
boundary/cofacet incidence; GUDHI retains a simplex tree. Their rows compare the
requested workflows with different storage, not identical data representations.
Ripser has no matching sparse approximation constructor.

## Validation and coverage

| Per revision, per round | Observed result |
| --- | --- |
| Prepared H0/H1 | 12 performance workloads, 48 backend groups, 576 measured processes; all passed |
| Native semantic checks | 20 fixtures, 76 supported calls passed; 4 empty/singleton Ripser adapter exclusions |
| Complete pipeline | 25 workflows, 69 supported groups, 828 measured processes and 69 discarded warmups; all passed |
| Pipeline exclusions | 6 explicit Ripser approximation exclusions |
| All eight runs | 5616 measured processes and 276 pipeline warmups; no mismatches, process failures or timeouts |

Every measured sample passed the existing interval-multiset and coverage checks;
pipeline checks also enforce the applicable construction/representative payload
contracts. Unsupported adapters are exclusions, not successful comparisons.
These counts describe this performance study and its validation, not a fresh run
of every standalone exact/sparse correctness fixture in the repository.

## Prepared-input before/after

Times are medians over 24 samples, in milliseconds. A negative change means less
time. The last two columns retain each round's candidate/baseline median ratio:
values below 1 mean faster. No confidence interval or significance test is claimed.

| Workload | Baseline ms | Candidate ms | Time change | Round 1 ratio | Round 2 ratio |
| --- | --- | --- | --- | --- | --- |
| `uniform_h1_32` | 0.203 | 0.164 | -19.1% | 0.8086 | 0.8093 |
| `uniform_h1_64` | 0.911 | 0.697 | -23.5% | 0.7658 | 0.7664 |
| `uniform_h1_128` | 3.975 | 3.070 | -22.8% | 0.7744 | 0.7714 |
| `circle_h1_128` | 17.268 | 16.054 | -7.0% | 0.9295 | 0.9335 |
| `clusters_h1_128` | 3.009 | 2.188 | -27.3% | 0.7281 | 0.7263 |
| `duplicates_h1_128` | 4.182 | 3.402 | -18.7% | 0.8113 | 0.8154 |
| `equal_h1_128` | 2.592 | 1.376 | -46.9% | 0.5313 | 0.5314 |
| `nonmetric_h1_64` | 6.626 | 6.248 | -5.7% | 0.9419 | 0.9466 |
| `uniform_h1_128_cutoff` | 1.130 | 0.927 | -18.0% | 0.8325 | 0.8180 |
| `uniform_h0_128` | 0.442 | 0.455 | +2.9% | 1.0267 | 1.0351 |
| `uniform_h0_512` | 8.389 | 8.460 | +0.9% | 1.0130 | 1.0051 |
| `uniform_h0_1024` | 37.419 | 37.323 | -0.3% | 1.0008 | 0.9880 |

The nine H1 cases are equally weighted in the geometric means quoted above;
uniform input appears at three sizes, so this is not an equal-family summary.
It is also not the historical 23-case mix in PR #6. H0 is a control and is excluded
from that aggregate: the 128-vertex row is about 2.9% slower, while larger H0 rows
are close. No favorable-row-only aggregate is used.

## Current native comparison

Candidate revision only. Times are **median [minimum, maximum]**, in milliseconds,
across the 24 measured samples. All 12 prepared-input workloads are shown.

| Workload | Cocycle | GUDHI direct | GUDHI collapse | Ripser |
| --- | --- | --- | --- | --- |
| `uniform_h1_32` | 0.164 [0.158, 0.173] | 1.459 [1.448, 1.492] | 0.313 [0.307, 0.327] | 0.140 [0.137, 0.165] |
| `uniform_h1_64` | 0.697 [0.687, 0.723] | 11.877 [11.771, 12.072] | 1.494 [1.472, 1.561] | 0.627 [0.616, 0.652] |
| `uniform_h1_128` | 3.070 [3.037, 3.234] | 120.561 [118.118, 121.876] | 9.076 [9.038, 9.345] | 2.699 [2.659, 2.876] |
| `circle_h1_128` | 16.054 [15.944, 16.313] | 107.661 [106.547, 113.633] | 176.157 [174.062, 180.061] | 13.586 [13.437, 14.005] |
| `clusters_h1_128` | 2.188 [2.172, 2.289] | 119.638 [118.199, 124.040] | 6.178 [6.136, 6.427] | 1.947 [1.929, 2.019] |
| `duplicates_h1_128` | 3.402 [3.362, 3.614] | 122.528 [120.065, 146.622] | 7.482 [7.432, 7.667] | 3.014 [2.984, 3.140] |
| `equal_h1_128` | 1.376 [1.366, 1.421] | 98.698 [96.659, 101.684] | 8.178 [8.080, 8.344] | 0.858 [0.836, 0.885] |
| `nonmetric_h1_64` | 6.248 [6.203, 6.607] | 13.987 [13.888, 14.340] | 12.043 [11.872, 12.198] | 6.391 [6.336, 6.753] |
| `uniform_h1_128_cutoff` | 0.927 [0.917, 0.955] | 1.505 [1.448, 1.538] | 0.723 [0.715, 0.746] | 0.567 [0.548, 0.585] |
| `uniform_h0_128` | 0.455 [0.448, 0.513] | 2.361 [2.342, 2.560] | 8.851 [8.765, 9.136] | 1.029 [0.996, 1.048] |
| `uniform_h0_512` | 8.460 [8.368, 9.814] | 47.763 [47.227, 48.864] | 473.351 [469.873, 483.682] | 19.337 [18.975, 19.861] |
| `uniform_h0_1024` | 37.323 [36.720, 38.764] | 268.996 [266.473, 288.634] | 3680.067 [3653.769, 3709.967] | 84.068 [83.094, 85.914] |

Cocycle's H0 path is faster than the measured references, but it does not always
use less memory. For H1, GUDHI direct is substantially slower on these inputs;
Ripser remains faster on most workloads. The nonmetric64 row is close, with a
small observed Cocycle advantage that should not be generalized. On the cutoff
row, both Ripser and GUDHI collapse beat Cocycle. GUDHI collapse is forced exactly
once; its H0 overhead does not represent GUDHI's best H0 strategy.

## Complete workflow before/after

Every pipeline workflow is retained below, including slowdowns. Names identify
the exact fixed fixture, API path, layout and requested output. Times are pooled
24-sample medians in milliseconds; the round ratios reveal variation hidden by
pooling. These are separate workloads from the prepared-input table.

| Workflow | Baseline ms | Candidate ms | Time change | Round 1 ratio | Round 2 ratio |
| --- | --- | --- | --- | --- | --- |
| `circle64_dense_lower_diagram` | 2.403 | 2.286 | -4.9% | 0.9399 | 0.9663 |
| `circle64_dense_upper_diagram` | 2.535 | 2.420 | -4.5% | 0.9513 | 0.9715 |
| `circle64_dense_square_diagram` | 2.293 | 2.214 | -3.4% | 0.9602 | 0.9864 |
| `circle64_cutoff_threshold_lower_diagram` | 0.147 | 0.117 | -20.6% | 0.8132 | 0.7829 |
| `circle64_cutoff_expanded_lower_diagram` | 0.752 | 0.812 | +8.1% | 1.0543 | 1.0782 |
| `circle64_cutoff_f3_threshold_lower_diagram` | 0.404 | 0.431 | +6.5% | 1.0515 | 1.0713 |
| `nonmetric24_dense_lower_diagram` | 8.789 | 9.427 | +7.3% | 1.0741 | 1.0699 |
| `nonmetric24_expanded_lower_diagram` | 22.473 | 23.667 | +5.3% | 1.0580 | 1.0463 |
| `sphere_h2_threshold_lower_diagram` | 0.038 | 0.036 | -5.0% | 0.9786 | 0.9539 |
| `sphere_h2_expanded_lower_diagram` | 0.056 | 0.054 | -3.4% | 0.9792 | 0.9318 |
| `sphere_h2_threshold_lower_bases` | 0.053 | 0.053 | -1.3% | 0.9639 | 1.0074 |
| `circle64_cutoff_threshold_lower_bases` | 17.226 | 17.783 | +3.2% | 1.0314 | 1.0332 |
| `bipartite16_flag_lower_diagram` | 0.107 | 0.094 | -12.3% | 0.8586 | 0.8873 |
| `bipartite16_f3_flag_lower_diagram` | 0.235 | 0.238 | +1.2% | 1.0256 | 0.9882 |
| `bipartite_isolates_flag_lower_diagram` | 2.700 | 2.661 | -1.4% | 0.9748 | 0.9845 |
| `uniform32_h0_dense_lower_diagram` | 0.053 | 0.050 | -5.7% | 0.9219 | 0.9863 |
| `metric32_approximate_lower_diagram` | 3.234 | 1.574 | -51.3% | 0.7128 | 0.4410 |
| `uniform64_h0_dense_lower_diagram` | 0.138 | 0.141 | +2.1% | 1.0154 | 1.0315 |
| `metric64_approximate_lower_diagram` | 7.389 | 8.659 | +17.2% | 1.2532 | 0.8734 |
| `metric64_approximate_checked_lower_diagram` | 8.910 | 8.768 | -1.6% | 1.1273 | 0.8780 |
| `metric64_approximate_expanded_lower_diagram` | 9.933 | 10.425 | +4.9% | 1.1397 | 0.9482 |
| `metric64_approximate_lower_bases` | 27.540 | 27.558 | +0.1% | 0.9761 | 1.0145 |
| `uniform128_h0_dense_lower_diagram` | 0.490 | 0.487 | -0.7% | 0.9839 | 0.9995 |
| `metric128_approximate_lower_diagram` | 18.929 | 18.354 | -3.0% | 1.0374 | 0.9406 |
| `points_line_points_lower_diagram` | 0.088 | 0.078 | -11.3% | 0.8985 | 0.8912 |

The explicit cutoff circle is slower in both rounds, as are the F3 cutoff circle
and nonmetric F3/H2 computation. Their pooled increases are 8.1%, 6.5% and 7.3%.
The nonmetric expanded workflow rises 5.3%; circle representative extraction rises
3.2%. These observations prevent an all-workflow improvement claim. They need
separate profiling and attribution before assigning a cause to either PR.

Approximation is particularly variable: metric64's time ratio switches from 1.2532
to 0.8734, and checked/expanded variants also switch direction. Metric32 is faster
in both rounds, but its ratios differ greatly (0.7128 versus 0.4410). Do not treat
its pooled 51.3% decrease as a stable algorithmic speedup. Both versions use the
same fixtures; phase samples and all slower observations remain in local evidence.

Selected current workflow comparisons, in milliseconds:

| Workflow | Cocycle | GUDHI | Ripser |
| --- | --- | --- | --- |
| Circle F2/H1, n=64 | 2.286 [2.257, 2.704] | 10.547 [10.433, 10.973] | 2.205 [2.179, 4.773] |
| Nonmetric F3/H2, n=24 | 9.427 [9.325, 9.857] | 4.882 [4.851, 5.078] | 1.018 [1.002, 1.197] |
| Same nonmetric input, explicit complex | 23.667 [23.291, 23.998] | 4.897 [4.828, 5.066] | Reference only |
| Sparse approximation F3/H1, n=128 | 18.354 [17.600, 24.373] | 7.979 [7.899, 9.024] | Unsupported |

The generic F3/H2 row remains about 1.9 times GUDHI's time and 9.3 times Ripser's.
Retained explicit expansion is about 4.8 times GUDHI's workflow time, under the
different storage contracts above. Approximation at n=128 is about 2.3 times
GUDHI in this snapshot. These are more important remaining gaps than the now
smaller exact F2/H1 gap on the circle workflow.

## Process memory

Maximum observed process peak RSS over 24 samples, in KiB. This includes runtime,
input buffers, allocator retention and output; it is not live kernel capacity.
The input widths and retained structures differ as described above.

| Prepared workload | Cocycle before | Cocycle now | GUDHI direct now | GUDHI collapse now | Ripser now |
| --- | --- | --- | --- | --- | --- |
| `uniform_h1_32` | 1408 | 1388 | 4032 | 2080 | 2016 |
| `uniform_h1_64` | 3000 | 1408 | 6396 | 3808 | 3724 |
| `uniform_h1_128` | 3668 | 2944 | 25140 | 4232 | 3768 |
| `circle_h1_128` | 4912 | 4148 | 25148 | 24216 | 5804 |
| `clusters_h1_128` | 3672 | 2952 | 25116 | 4244 | 3808 |
| `duplicates_h1_128` | 3668 | 3032 | 25136 | 4276 | 3944 |
| `equal_h1_128` | 3828 | 2980 | 25148 | 4304 | 3764 |
| `nonmetric_h1_64` | 3520 | 3208 | 6404 | 5528 | 4404 |
| `uniform_h1_128_cutoff` | 2936 | 1408 | 4012 | 3716 | 3760 |
| `uniform_h0_1024` | 22168 | 22064 | 54648 | 50204 | 17828 |

| Current pipeline workflow | Cocycle KiB | GUDHI KiB | Ripser KiB |
| --- | --- | --- | --- |
| Circle F2/H1, n=64 | 3236 | 6464 | 4212 |
| Nonmetric F3/H2, n=24 | 3636 | 4736 | 3848 |
| Same nonmetric input, explicit complex | 8320 | 4736 | Reference only |
| Sparse approximation F3/H1, n=128 | 4252 | 5112 | Unsupported |

## Reproduction and evidence

Raw evidence is **local-only**. There is no public artifact URL or guaranteed
retention, and a checksum does not make the data publicly accessible. The report
preserves a reviewed summary; generated samples, logs and builds remain outside
source Git.

The ignored local evidence roots are:

- `target/benchmarks/commit-a201027682c4/{native-baseline,pipeline}/run-001/` and `run-002/`.
- `target/benchmarks/commit-4d7178a1c474/{native-baseline,pipeline}/run-001/` and `run-002/`.
- `target/benchmarks/commit-4d7178a1c474/comparison/run-001/`: study plan, exact commands, version order, analysis, analysis script and evidence manifest.

Each run has `study.json` with the full measured commit, commands, UTC metadata,
load observations and clean-tree audit. Suite outputs retain every sample,
fixture hash, worker/header identity, source fingerprint and validation result.
The comparison audit verifies matching fixtures and unchanged C++ binaries.

| Source fingerprint scope | SHA256 |
| --- | --- |
| native-baseline-before | `36de97c34a9100372d9900938fc9ce350ac4af1c81bad81efac1afc68e34aa15` |
| native-baseline-after | `6e69e6ded68411aa8a5e14a8659e42751528a3b6198f4cfba97a122bb6ca131a` |
| pipeline-before | `2aedd4af3c457ce9cdf6c4df93b5fd39c66a1d2af1c3545eaa425684843c5c95` |
| pipeline-after | `2f90e9e46669c89cc321e619b20389e4902d8d1a4ca7878c505bc93681a2f432` |

Evidence manifest SHA256: `4425791ab432478678d0bf4ef3188ba660915982d69036d51c71fc3b3356a0f3`.

Use clean isolated checkouts of the two full revisions above, the same native
[source pins and setup](../native/README.md#setup), and the recorded compiler.
The commands below assume those sources and Boost headers under the checkout's
`target/native-sources/`; explicit absolute source paths are also supported.
Choose new attempt directories, never overwrite prior evidence. For each suite,
run baseline then candidate in round 1 and candidate then baseline in round 2.
Set `round=1` then `round=2`, producing pipeline order seeds 7919 and 15838.

```sh
revision=$(git rev-parse --short=12 HEAD)
round=1
attempt=$(printf '%03d' "$round")
python3 tools/benchmark_native.py --samples 12 --cpu 2 \
  --boost-include target/native-sources/boost/usr/include \
  --output "target/benchmarks/commit-${revision}/native-baseline/run-${attempt}"
python3 tools/benchmark_rips_pipeline.py --samples 12 --cpu 2 \
  --kernel-revision HEAD --order-seed "$((7919 * round))" \
  --boost-include target/native-sources/boost/usr/include \
  --output "target/benchmarks/commit-${revision}/pipeline/run-${attempt}"
```

This single-host experiment has no large-scale stress sweep or independent-machine
confirmation. Most H1 workloads have at most 128 vertices, nonmetric H1 uses 64,
and the high-dimensional comparison uses 24. Sub-millisecond costs and process RSS
floors need particular care. It neither establishes an all-input optimum nor
benchmarks the latest upstream releases. Keep future reruns on this same page,
updating revisions, evidence status, limits and conclusions together under the
[reporting rules](../reporting.md).
