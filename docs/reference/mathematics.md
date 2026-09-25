# Mathematical specification

[Documentation](../README.md) / Reference

This document defines the mathematical and numerical contracts shared by the
implementation, rustdoc, and tests. It distinguishes mathematical results from
Cocycle's engineering conventions. See [references](bibliography.md) for primary
sources and [testing](../development/testing.md) for independent checks.

## 1. Scope and notation

The current algorithms compute ordinary Vietoris–Rips persistent homology of
finite inputs over a validated prime field $\mathbb F_p$ through any requested
homology dimension, with F2 the default. The legacy `RipsOptions` path remains
limited to F2 H0/H1. Optional cycle and cocycle bases are described in section 14.
Reduced homology, non-prime coefficient rings, zigzag and multiparameter
persistence are not implemented. Nonnegative scales and zero H0 births are Rips-specific; the general
interval representation is not restricted to these dimensions or scales.

Matching distances between complete diagrams are specified in section 16.

| Symbol | Meaning |
| --- | --- |
| $n,d$ | Vertex count and ambient dimension |
| $\delta_{ij}$ | Finite, nonnegative, symmetric dissimilarity; Euclidean distance for point clouds |
| $t,T$ | Filtration scale and optional cutoff |
| $q$ | Maximum requested homology dimension |
| $\sigma,f(\sigma)$ | Simplex and filtration value |
| $D,R$ | Boundary matrix and reduced matrix |
| $\mathcal D_k$ | Persistence diagram in dimension k, as a multiset |

## 2. Inputs and filtration

### 2.1 Distances

For $X=(x_0,\ldots,x_{n-1})$, with $x_i\in\mathbb R^d$,

$$\delta_{ij}=\lVert x_i-x_j\rVert_2.$$

Vertices are labeled observations. Duplicate coordinates remain distinct
vertices, so zero distance between different vertices is valid (a pseudometric).

**Engineering conventions.** Require $n\ge0$, $d>0$, row-major coordinates of
length $nd$, and finite `f64` coordinates. Dissimilarities must be finite and
nonnegative. Interpret `-0.0` as zero without modifying borrowed input; normalize
stored/output zero to positive zero. Reject unrepresentable distances instead of
substituting infinity. Scaled `hypot` avoids unnecessary square-sum overflow and
underflow. Reduction is combinatorially exact for the computed floating-point
filtration values, not exact real arithmetic.

Precomputed input is the strict lower triangle with an explicit vertex count:

$$
[\delta_{10},\delta_{20},\delta_{21},\ldots],\qquad
\operatorname{offset}(i,j)=i(i-1)/2+j\quad(i>j).
$$

The length is $n(n-1)/2$, the diagonal is implicitly zero, and size arithmetic is
checked. Zero and one vertex both need an empty buffer, distinguished by $n$.
The triangle inequality is not required: symmetric dissimilarities still define
a flag filtration, but their validation does not certify a metric.

### 2.2 Rips filtration

A simplex is a nonempty finite vertex set, with dimension $|\sigma|-1$. A
simplicial complex is closed under nonempty faces. A filtration satisfies
$\tau\subseteq\sigma\Rightarrow f(\tau)\le f(\sigma)$.

Cocycle uses edge-length scales:

$$
f(\{i\})=0,\qquad
f(\sigma)=\max_{i,j\in\sigma}\delta_{ij}\quad(|\sigma|\ge2),
$$

$$
K_t=\{\sigma:f(\sigma)\le t\}\quad(t\ge0),\qquad
K_t=\varnothing\quad(t<0).
$$

The threshold is closed. The parameter is neither a ball radius nor a squared
distance; compare [B21 §2](bibliography.md#b21). A face uses a subset of the vertex
pairs, so its maximum edge cannot be larger. This proves face closure and
$K_s\subseteq K_t$ for $s\le t$ without requiring the triangle inequality.

The explicit reference sorts by `(value, dimension, sorted_vertex_ids)` ascending.
The implicit path uses the descending combinatorial IDs in section 9. Both put
faces before cofaces. Tie refinements affect internal pair IDs but must preserve
the positive-lifetime diagram multiset. Sorting never merges nearby values using
an epsilon.

## 3. Chains, boundaries, and homology

First consider the F2 specialization; section 13 gives oriented prime-field
coefficients. Let $C_k(K;\mathbb F_2)$ have the k-simplices as a basis. A chain can be represented
by a set of simplex indices, with addition given by symmetric difference.

$$
\partial_k[v_0,\ldots,v_k]
=\sum_{r=0}^{k}[v_0,\ldots,\widehat v_r,\ldots,v_k]
\quad(k\ge1),\qquad \partial_0=0.
$$

Each codimension-two face occurs twice when deleting two vertices, and cancels
over F2. Hence $\partial_{k-1}\partial_k=0$, and

$$
Z_k=\ker\partial_k,\quad B_k=\operatorname{im}\partial_{k+1},\quad H_k=Z_k/B_k,
$$

$$
\beta_k=\dim C_k-\operatorname{rank}\partial_k-
\operatorname{rank}\partial_{k+1}.
$$

This rank formula supplies an oracle independent of persistence pairing.
Computing H1 needs triangles: edges determine $\ker\partial_1$, but triangle
boundaries determine which cycles are filled. A $(q+1)$-skeleton suffices for
H0 through Hq; its unpaired higher-dimensional chains are not full Rips output.
See [ZC05](bibliography.md#zc05) for the algebraic background.

## 4. Persistence and reference boundary reduction

Inclusions $K_s\hookrightarrow K_t$ induce homology maps. A finite filtration
over a field admits an interval-multiset description. Finite intervals use
$[b,d)$: present at birth, absent at death.

For ordered simplices $\sigma_0,\ldots,\sigma_{m-1}$, set $D_{ij}=1$ exactly when
$\sigma_i$ is a codimension-one face of $\sigma_j$. Nonzero entries satisfy
$i<j$ and $D^2=0$. `low` is the largest row index of a nonempty column; an empty
column has no pivot. Row zero is valid and must not be an empty sentinel.

```text
pivot_owner = empty map
R = empty columns
for j in increasing filtration order:
    column = boundary(sigma[j])
    while column is nonempty:
        i = low(column)
        if pivot_owner has no i:
            break
        column = xor(column, R[pivot_owner[i]])
    R[j] = column
    if column is nonempty:
        i = low(column)
        pivot_owner[i] = j
        record pair (i, j)

after all columns:
    unpaired_births = {i: R[i] is empty and i is not in pivot_owner}
```

Only earlier columns are added, so $R=DV$ for an invertible upper-triangular V,
preserving every prefix boundary-column space. Each cancellation strictly lowers
the pivot, ensuring termination; nonzero reduced columns have distinct pivots.
The pairing theorem then identifies `(low(R[j]), j)` with persistence pairs and
unpaired empty columns with surviving births. Equal rank alone is not a proof of
this pairing theorem: the direct source is [B21 Proposition 3.1 and Algorithm 1](bibliography.md#b21).

A pair `(i,j)` gives dimension $\dim\sigma_i$ and endpoints $f(\sigma_i),f(\sigma_j)$.
Determine unpaired births only after reduction, and output only dimensions through q.
Internal validation retains zero-length index pairs; public diagrams omit $b=d$
because $[b,b)$ is empty. There is no additional minimum-persistence filter.
The independent F2 test oracle described here does not export representatives;
section 14 specifies the production representative path.

Sparse columns use ordered symmetric difference with reusable buffers. An empty
reduced column retains its index marker while its allocation may be reused: an
empty column cannot own a pivot and will not be read for later elimination.
This preserves order, pairs, and the $R=DV$ invariant.

## 5. Complete and truncated computations

Let $\Delta=\max_{i<j}\delta_{ij}$, with $\Delta=0$ if there are no vertex pairs.
At $t\ge\Delta$, the full flag complex of a nonempty input is one full simplex.
Its ordinary persistence has one essential H0 interval and no essential H1.
This is a property of these Rips inputs, not of arbitrary complexes.

`max_edge=None` requests the complete filtration. A finite $T\ge0$ includes all
necessary simplices with $f(\sigma)\le T$; $T\ge\Delta$ also establishes complete
coverage. A computation failure is an error, never a successful truncation.

| Endpoint | Interpretation |
| --- | --- |
| `Finite(d)` | Observed death; interval $[b,d)$ |
| `Essential` | Unpaired in a complete computation; death $+\infty$ |
| `RightCensored { through: T }` | Alive at T; full death is unknown |

In incomplete coverage, all surviving births, including H0, are conservatively
right-censored. The result retains computed dimensions and coverage. A single
optional death value cannot express these distinctions.

One could extend the truncated filtration constantly and write infinite intervals
for its survivors. Censoring instead describes uncertainty about the original
full filtration. Surviving through T is not dying at T.

## 6. H0 by union-find

All vertices are born at zero. Process edges by `(weight, vertex_ids)` and merge
connected components using a deterministic representative rule. An edge joining
two components kills one H0 class at its weight; an edge within a component does
not change H0. Higher-dimensional simplices do not merge additional components.

For complete input, selected edges form a minimum spanning tree; repeated points
allow zero-weight edges. At a cutoff, they form a minimum spanning forest with
surviving components. Test this path against boundary reduction. See
[B21 §4](bibliography.md#b21) for the H0 algorithm context.

## 7. Diagram descriptors

For one dimension, select finite positive lifetimes $\ell_i=d_i-b_i>0$. Let their
count be N and total $L=\sum_i\ell_i$.

| Descriptor | Definition | No finite positive lifetimes |
| --- | --- | --- |
| Finite count | $N$ | 0 |
| Total persistence | $L$, power one without a root | 0 |
| Maximum persistence | $\max_i\ell_i$ | `None` |
| Persistence entropy | $-\sum_i p_i\ln p_i$, $p_i=\ell_i/L$ | `None` |

[A20 Definition 3.1](bibliography.md#a20) uses base-two logarithms. Cocycle explicitly
uses natural logarithms (nats), without division by $\ln N$. One positive interval
has entropy zero; an empty collection has no normalized lifetime distribution.

The summary reports excluded essential and censored counts. It describes observed
finite lifetimes and does not estimate censored deaths or replace them with T.
Non-finite arithmetic results are errors. Total persistence uses compensated
summation. If an extreme probability underflows to zero, its $p\ln p$ contribution
is taken as zero, its limiting value.

Betti curves use all intervals:

$$\beta_k(t)=\#\{[b,d)\in\mathcal D_k:b\le t<d\}.$$

Essential intervals count for $t\ge b$. Censored intervals count throughout the
known range $b\le t\le T$; truncated diagrams reject queries beyond T. Caller-supplied
grid values must be finite, nonnegative, and strictly increasing. Omitting
zero-length pairs does not change Betti numbers at any scale.

## 8. Stability assumptions

For finite metric spaces, Rips diagrams satisfy

$$d_B(\mathcal D_k(X),\mathcal D_k(Y))\le2d_{GH}(X,Y),$$

by [CSO13 Theorem 5.2](bibliography.md#cso13). Bottleneck distance $d_B$ allows
matching to the diagonal; $d_{GH}$ is Gromov–Hausdorff distance. Do not apply this
statement directly to other filtrations, arbitrary nonmetric inputs, or censored
diagrams without establishing the required assumptions.

**Fixed-vertex derivation used in tests.** If
$\max_{ij}|\delta_{ij}-\delta'_{ij}|\le\varepsilon$, each simplex's maximum edge
changes by at most $\varepsilon$. The complete filtrations include into each
other's $\varepsilon$ shifts, giving an $\varepsilon$-interleaving. Applying
[CSO13 Theorem 2.3](bibliography.md#cso13) yields $d_B\le\varepsilon$.
This argument also holds for finite symmetric dissimilarities on the same vertices.

If corresponding Euclidean points move by at most $\eta$, the triangle inequality
bounds each edge change by $2\eta$, giving the corresponding $2\eta$ bound.
Numerical tests additionally allow explicit rounding tolerance. This does not
imply continuity of thresholded interval counts.

## 9. Implicit Rips persistent cohomology

This section specifies the production H1 path. The explicit boundary oracle
remains independent. Cohomology, clearing, implicit columns, and shortcut pairs
have separate conditions; see [B21 §§3.2–3.5 and §4](bibliography.md#b21).

### Numbering and order

For $a<b<c$,

$$
\operatorname{id}(a,b)=\binom b2+a,\qquad
\operatorname{id}(a,b,c)=\binom c3+\binom b2+a.
$$

Forward filtration order is maximum edge ascending, dimension ascending, and
combinatorial ID descending. Faces precede cofaces. Use descending comparison,
not unsigned negation. Exhaustive small-set tests check IDs and inversion.
Changed tie order may change index pairs but not the positive-lifetime multiset.

Cancel binomial denominators before checked multiplication. H1 requires
$\binom n3$ to fit the platform's `usize`, even when a cutoff retains few triangles.
Return `SizeOverflow` otherwise; 32-bit and 64-bit indexing limits differ.

### Coboundaries and pairing direction

The F2 coboundary of an edge consists of triangles containing it within the
cutoff. A transpose alone is insufficient: the small matrix oracle uses
$C=JD^\top J$, reversing both rows and columns. With S simplices, a C pair `(i,j)`
corresponds to the D pair `(S-1-j,S-1-i)`. Restore original dimensions and
filtration values, and also check unpaired indices and censored H1 classes.

Scan edges forward to obtain union-find merges and cycle births. Then process
edge coboundary columns in reverse, with the earliest forward triangle as pivot
(the highest reversed row). A paired edge/triangle yields an H1 birth/death;
an unpaired cycle-birth edge survives. H0 comes from union-find independently.
Clearing skips H0 merge edges: the adjacent-dimension pairing guarantees their
coboundaries reduce to zero. Tests disabling clearing still reduce them, verify
that result, and never mistake them for H1 births. Collect edges even after the
graph becomes connected.

### Implicit reconstruction invariant

Maintain $R=CV$. For each pivot, store
$V_j=e_j+\sum_{k\in A_j}e_k$ rather than retaining $R_j$. To cancel a pivot,
regenerate $Ce_j$ and all $Ce_k$, adding them to the working column while adding
their edge indices to the current transformation. The edges in $A_j$ precede j
in reverse computation order, so V stays unit upper triangular.

Repeated triangles and transformation indices cancel by F2 parity. Cancel all
copies of the candidate pivot before using it; ordinary set deduplication is
incorrect. Each elimination strictly lowers the pivot in reversed row order.
Stored transformations are internal machinery, not public representative cocycles.

### Clearing and shortcut pairs

H0 pairings and H1 clearing must use the same total order. An apparent pair
$(\sigma,\tau)$ requires sigma to be tau's latest facet and tau to be sigma's
earliest cofacet. Omitting its public interval also requires equal filtration
values. Do not indiscriminately remove columns that eventually reduce to zero.
The kernel omits stored zero-lifetime apparent pairs and reconstructs them when
later columns need their pivots. Other shortcut pairs retain their pivot owners
and transformation columns.

Initialize the column and inspect shortcut candidates in one traversal, before
any column addition. Cofacets are enumerated by descending ID,
and their values are at least the edge value. The first equal-valued triangle is
therefore the original column's earliest cofacet. Retain preceding cofacets in a
temporary buffer. If no shortcut applies, continue the same traversal and
build the working heap from the complete buffer; do not enumerate the prefix or
an empty column again. Reuse the buffer only after its previous contents have
been consumed or discarded.

An emergent shortcut accepts this first equal-valued pivot only when it has no
stored or virtual owner. Ordinary reduction would accept it immediately; retain
$V_j=e_j$ without generating remaining rows. If owned, fall back to ordinary
reduction, not the next equal-valued triangle. Apparent-only mode additionally
checks the latest-facet condition. Mid-reduction emergent shortcuts are not used.

For a zero-lifetime apparent pair $(\sigma,\tau)$, the same initial traversal
certifies both conditions: tau is sigma's first equal-valued cofacet, and sigma
is tau's latest facet. Skip this pair without storing its pivot owner or
transformation column. It remains a virtual column $V_\sigma=e_\sigma$;
omitting its storage does not remove its role in later elimination.

When a working pivot has no stored owner, test for that virtual pair on demand:
take the triangle's latest facet sigma, require equal filtration values, and
check that the triangle is sigma's earliest equal-valued cofacet. Only an edge
preceding the active column in reverse computation order may serve as its
virtual owner. Add $Ce_\sigma$ to cancel the pivot and include sigma in the
active transformation, with F2 parity as for an ordinary stored owner. This
preserves $R=CV$ and triangularity; every elimination strictly lowers the pivot.
A triangle that fails these conditions cannot be treated as a virtual owner.

Zero-lifetime H1 pairs are omitted before raw interval storage; the shared result
assembler still removes zero-lifetime H0 pairs. Union-find, clearing, and the
stored or virtual reduction state still account for those pairs. In particular,
zero-lifetime emergent pairs that are
not apparent retain their pivot/transform entries. Positive-lifetime, essential,
and right-censored intervals retain their multiplicities. Test ordinary,
apparent, emergent, and combined configurations against the independent oracle.
Independent two-pass tests also check storage omission independently of
single-pass caching, including the invariant $R=CV$ on dense and sparse inputs.

### Cone stopping bound

For a nonempty finite symmetric nonnegative input, define

$$r_*=\min_v\max_u\delta(v,u).$$

Choose a minimizing vertex $v_*$. At $t\ge r_*$, all edges to $v_*$ exist.
For every Rips simplex sigma, its union with $v_*$ is also a simplex. Thus the
complex is a cone: one ordinary H0 class and no positive-dimensional homology.
This derivation uses only the maximum-edge rule, not the triangle inequality.

A complete computation may stop at $r_*$; with cutoff T, use $\min(T,r_*)$.
Include all events exactly at the stop. Handle empty input separately and set
$r_*=0$ for a singleton.

Internal stopping does not replace public coverage. No user cutoff still means
`Complete`. If $T<\Delta$, retain `Through(T)` and its censoring convention even
when the cone is reached earlier; never substitute $r_*$ for T.

## 10. Analytic complete-bipartite filtration

Partition n vertices into groups of sizes $a,b>0$, with $a+b=n$. Assign cross-group
distance 1, within-group off-diagonal distance 2, and diagonal zero.

- For $0\le t<1$: vertices only; $\beta_0=n$, $\beta_1=0$.
- For $1\le t<2$: the connected triangle-free graph $K_{a,b}$;
  $\beta_1=E-V+1=ab-a-b+1=(a-1)(b-1)$.
- For $t\ge2$: the full simplex; $\beta_0=1$, $\beta_1=0$.

The complete diagram contains n−1 H0 intervals $[0,1)$ and one $[0,\infty)$;
H1 contains $(a-1)(b-1)$ copies of $[1,2)$. At T=1, H1 survivors and the remaining
H0 class are censored. Exception: a=b=1 has actual diameter 1, so T=1 is complete,
the H0 survivor is essential, and H1 is empty.

This independent derivation checks repeated endpoints, quadratic output
multiplicity, and triangle-free enumeration. The output itself can contain
$\Theta(n^2)$ intervals; do not attribute all their storage to reduction fill-in
or deduplicate intervals to improve measured cost.

## 11. Exact graph inputs and shared flag computation

The [construction guide](../guides/rips-construction.md) describes the public APIs.
A finite graph G has vertices at zero, edge weights w >= 0, and a simplex for
each clique, born at its maximum edge weight. Missing edges never enter. This
complete flag filtration can have essential H1. A cutoff below its maximum edge
conservatively gives right-censored survivors under the existing diagram contract.

An exact threshold graph constructed from a full pairwise input instead certifies
only the original Rips through its recorded cutoff, unless every original pair
was included. Its maximum retained edge cannot establish original completeness.
Requesting computation beyond that certified range is an error. With no new
cutoff, computation uses the construction's available range.

The shared H1 engine requires forward edge order and decreasing combinatorial ID
order for triangle cofacets. For a fixed edge, inserting increasing third vertices
increases the triangle ID. Reverse intersection of sorted neighbor lists therefore
satisfies the dense enumerator's order. Every emitted triangle has all three edges,
and latest-facet lookup uses the same value/ID order. These facts preserve the
original-column apparent/emergent-pair tests on supplied flag filtrations. No
complete-distance cone bound is inferred from missing sparse edges. Independent
vertex-triple enumeration and ordinary reduction validate the sparse path.

Lower/upper/square layouts borrow original f64 values. Exact square symmetry and
zero diagonal are checked without tolerance repairs. Exact construction needs no
triangle inequality. Custom callbacks supply one orientation of each pair and
must respect the documented symmetric, stable-function contract. Work limits and
cancellation indicate execution failure; they do not change a filtration cutoff
or return a successful partial diagram.

## 12. Explicit skeletons and dimension-generic cohomology

For a weighted graph, each stored clique has increasing vertices and value equal
to its largest edge (vertices enter at zero). Production filtration order is
increasing value, increasing dimension, then decreasing colexicographic order.
For equal-sized simplices this last order agrees with decreasing combinatorial
rank from section 9, without requiring that rank to fit a machine integer.
An increasing-vertex simplex has oriented boundary obtained by omitting vertex
position i with coefficient (-1)^i. Each codimension-two face appears twice with
opposite signs, so the stored integer boundary squares to zero before reduction
modulo two.

An explicit p-skeleton suffices for Hq only when p >= q+1, or when construction
certifies that it already contains the whole threshold clique complex. Otherwise
missing cofaces can create artificial top-dimensional survivors. The expansion
probes unique extensions of its last dimension to establish exhaustion. This
certificate concerns only dimensions at the constructed scale; it does not extend
original-input scale coverage. Dimension and scale checks are independent.

For dimension-generic F2 computation, classify edges by forward union-find, then
process remaining edges in reverse order. At each dimension, the working column
is a set of cofacets; symmetric difference performs F2 addition. Its earliest
forward cofacet is the reverse coboundary pivot. An owned pivot triggers addition
of the owner's transformation column, regenerating its coboundary on demand. A
new pivot pairs the current simplex with that cofacet; an empty reduced column
records an unpaired class unless the simplex was cleared. Pivot simplices become
cleared columns in the next dimension. Cleared simplices still participate in
clique generation: clearing skips algebra, not topology. This is the dimensional
extension of the reversed-transpose duality in section 9 and [B21](bibliography.md#b21).

Only the current dimension, transformations, pivot ownership and a working
coboundary are needed on the implicit path. These can still be exponentially
large. The explicit path uses the same reducer over stored incidence and thus
also pays for the already materialized skeleton. H0/H1-only implicit requests
retain the specialized compact-index engine and its pair shortcuts.

As an independent high-dimensional fixture, partition 2r vertices into r pairs.
Give opposite partners distance 2 and all other pairs distance 1. Between scales
1 and 2 the clique complex is the join of r copies of S0, hence S^(r-1); at scale
2 it becomes a full simplex. Its positive-dimensional diagram consists of one
H_(r-1) interval [1,2) for r >= 2. Omitting opposite edges permanently instead
makes that sphere essential in the supplied graph filtration. Tests exercise
r=3,4,5, alongside independent forward boundary reduction of all small vertex
subsets and native C++ comparisons.

## 13. Prime fields and oriented reduction

`PrimeField` accepts exactly prime u32 characteristics. Arithmetic outputs residues
in 0..p; nonzero sparse entries lie in 1..p. The largest accepted characteristic
is 4294967291. Exact trial division validates primality, u64 intermediates cover
every u32 product and sum, and modular exponentiation computes inverses. Zero
has no multiplicative inverse and is rejected. These arithmetic choices are
implementation contracts, independent of filtration precision.

For increasingly oriented vertices, the coefficient on the facet omitting position
i is (-1)^i modulo p. Coboundary incidence uses the same coefficient. Normalizing
a pivot c multiplies its column and transformation by c^(-1); eliminating a
coefficient a against a normalized pivot adds -a times that column. F2 reduces
to parity cancellation. Dimension progression and clearing do not depend on p.
H0 connectivity uses union-find over every supported field.

Independent dense rank checks use separate arithmetic and enumeration. A
barycentric subdivision of the [six-vertex RP2 triangulation](bibliography.md#rp2)
is flag: vertices represent nonempty faces, and edges join comparable faces.
Pairwise comparable faces form chains, so its clique complex is the subdivision.
This fixture has one H1 and one H2 class over F2 and neither over odd primes.
The test independently checks those ranks, and native C++ comparisons check the
same weighted flag filtration with matching coefficient fields. Reference modulus
limits are recorded as exclusions, never silent narrowing.

## 14. Persistent representative bases

For a full forward boundary reduction R=DV in filtration order, a finite pair
(i,j) supplies cycle R_j with leading simplex i. It is available at birth i and
becomes a boundary at death j. An unpaired birth i supplies V_i. The active
subset of these cycles gives a homology basis at every scale; see
[B21, section 3.1](bibliography.md#b21). Using V_i for every finite pair would
not establish its death association. Production normalizes nonzero pivot columns,
which only rescales the selected basis vectors.

The caller requests representatives after all simplices at a finite query scale
t have entered. Active intervals satisfy birth <= t < death, with an unpaired
interval alive through an inclusive known cutoff. Zero-length pairs are omitted.
At t, reduced boundary columns whose source simplices have entered span B_q(K_t).
For active cycle basis z_i, solve cochain constraints

$$
\varphi_i(b)=0\quad(b\in B_q(K_t)),\qquad
\varphi_i(z_j)=\delta_{ij}.
$$

Sparse row elimination normalizes pivot constraints and then back-substitutes
with free coordinates set to zero. Annihilating boundaries is exactly the
cocycle equation; identity pairing proves nontriviality and independence modulo
coboundaries. This construction defines a dual basis for the requested scale,
not a promise that the returned cochain entries remain unchanged at another
scale. No shortest-support or canonical-across-algorithms claim is made.

Output identities are indices into one result's sorted interval multiset.
Repeated intervals retain distinct indices; request positions distinguish repeated
queries. Terms contain original, increasing simplex vertices and canonical
nonzero coefficients. Queries outside known coverage or computed dimensions fail.
Requested representatives own their terms and do not borrow a reduction workspace.

Only nonempty request slices materialize a representative skeleton and retain
forward transformations. All relevant exact paths produce the same diagram with
or without requests, while resource costs differ. Tests independently verify
closure, boundary/coboundary quotient ranks, identity pairings, birth support,
finite-death association, coefficient ranges, repeated intervals and censoring.
Native references validate prime-field interval multisets; these adapters do not
claim an upstream representative-vector oracle.

## 15. Sparse Rips approximation

Sparse Rips is a different filtration from exact threshold Rips. The construction
follows the pinned GUDHI C++ sparse edge/blocker convention, with deterministic
sampling instead of its random initial landmark. See [CJS15](bibliography.md#cjs15)
and the [source study](../design/rips.md#three-project-baseline).

Let `p_0,...,p_(n-1)` be a greedy farthest-point permutation. Its insertion radii
are `lambda_0 = infinity` and `lambda_j = min_(i<j) d(p_i,p_j)`. Ties choose the
smallest original vertex ID. Retain the initial vertex and the prefix of positive
radii at least the optional minimum insertion radius. A pseudometric may have
zero-distance duplicates; identifying these preserves its Rips persistence.

For retained `i < j`, write `d = d(p_i,p_j)`, `li = lambda_i`, `lj = lambda_j`.
The modified edge value, in edge-length units, is:

```text
alpha = d                    if d * epsilon <= 2 * lj
edge absent                  if d * epsilon > li + lj (and the first case failed)
alpha = 2 * (d - lj/epsilon)  otherwise
```

For `epsilon < 1`, the last branch is additionally absent if
`alpha * c > lj`, where `c = epsilon * (1-epsilon)/2`. Retain only values at most
the construction threshold. Vertices have value zero. A higher simplex is a
clique whose value `f` is its largest edge value and which satisfies
`lambda_v >= f*c` at every vertex. For epsilon at least one this additional
constraint is disabled. This is not ordinary flag expansion for epsilon below one.
The constraint is hereditary: a face has a no-larger value and no-smaller minimum
insertion radius. Therefore rejecting a simplex cannot remove an admissible coface.

The conditional metric theorem uses `0 < epsilon < 1`, exact arithmetic and a
greedy permutation. GUDHI states a `(1, 1/(1-epsilon))` interleaving with exact
Rips. In [CJS15](bibliography.md#cjs15), set the paper's parameter to
`delta = epsilon/(1-epsilon)`: the ball cap is `lambda/epsilon`, disappearance
radius is `lambda/[epsilon*(1-epsilon)]`, and the factor is `1+delta`.
GUDHI doubles radius scales to edge-length scales. CJS15 §2 embeds finite metrics
isometrically in the max norm, allowing its nerve argument to cover metric Rips;
§4 states the approximation theorem and §5 explains the higher-simplex test.
Bounds apply to the retained metric subset after positive-radius subsampling.
No unchanged multiplicative guarantee to the full input is claimed then.

The library records checked, caller-assumed or absent metric hypotheses. Exact
checking treats finite binary64 inputs as real dyadic numbers. FastTwoSum
recovers the sign of a rounded sum's error to test triangle equality correctly;
no tolerance silently changes the hypothesis. A pseudometric with zero-distance
pairs is allowed. The reported factor is a nominal binary64 evaluation of the
ideal theorem, not a certificate for accumulated construction rounding. Extreme
arithmetic that overflows, or epsilon causing a zero blocker factor by underflow,
returns a numerical error. Other operations use ordinary binary64 rounding.

Coverage describes the approximate filtration. Edges excluded by its rules never
enter; those omitted solely by a finite threshold make the range incomplete.
Dimension truncation is separate: Hq requires construction through q+1 unless
expansion certified exhaustion. All public simplex/representative vertex labels
refer to original input IDs, while the exposed graph uses a documented compact
map. Representative bases describe this approximate filtration and do not claim
a chain map into original Rips at the same scale.

## 16. Diagram matching distances

The `diagram_distances` domain compares one explicitly computed homology dimension
of two complete diagrams. Points are a multiset: repetitions remain separate
matching obligations. For finite points, allow partial bijections between the
two multisets and match every unused point to the diagonal.

| Operation | Ground cost between finite points | Diagonal cost | Objective |
| --- | --- | --- | --- |
| `bottleneck_distance` | L-infinity | `(death-birth)/2` | Minimum largest cost |
| `wasserstein_1_infinity` | L-infinity | `(death-birth)/2` | Minimum sum of costs |
| `wasserstein_2_euclidean` | Euclidean | `(death-birth)/sqrt(2)` | Square root of minimum sum of squared costs |

Only `(finite birth, positive infinity)` essential points are representable.
They match essential points in sorted birth order, with absolute birth difference
as cost. Unequal counts give positive infinity. Combine finite and essential
contributions by maximum, sum, or Euclidean norm for the three operations.
Empty computed dimensions are valid; uncomputed dimensions are errors. Reject
all `Coverage::Through` inputs, even if no current interval is censored: future
births and deaths are unknown. A cutoff must not replace a death or certify
essentiality. Diagonal points, non-finite births and other infinite endpoint
categories remain excluded by the existing interval constructors.

The `_results` functions additionally require equal coefficient characteristics.
Every current computation context uses edge-length scales. Vertex counts,
requested cutoffs and filtration kinds need not match when coverage is complete.
Approximate constructions retain their provenance in the borrowed results;
the returned scalar measures their actual diagrams without certifying a distance
between the original datasets. Raw-diagram calls cannot establish provenance.

Exactness excludes algorithmic approximation; it does not mean exact real
arithmetic. Use binary64, deterministic tie handling and stable cost aggregation.
W2 is recomputed from the chosen original cross/diagonal costs, rather than by
subtracting two nearly equal total savings. Unrepresentable required arithmetic
returns `NumericalFailure`, distinct from the mathematical infinity caused by
unequal essential counts. Underflow must not silently erase a required nonzero
cost. Scaling used by a solver must preserve representable input distinctions
or fail explicitly.

The native solvers are adapted from the MIT-licensed
[Topp source](https://github.com/proffitteoy/Topp/tree/ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234).
Implementation-local license notices retain attribution. Their private routing
and benchmark switches are not public API. Independent exhaustive partial
matching, hand-derived cases and Topp/GUDHI comparisons validate the supported
domain; source translation alone is not independent evidence.
