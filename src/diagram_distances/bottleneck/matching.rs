//! Degree-ordered greedy Kuhn with the C0 small-bitset / x-sweep CSR layout.

#[cfg(any(test, cocycle_distance_bench))]
use super::bytes;
use super::{Diagnostics, NONE, Pair, filled, push, reserve, size_overflow};
use crate::Result;

struct Graph {
    size: usize,
    words: usize,
    bits: Vec<u64>,
    offsets: Vec<usize>,
    neighbors: Vec<usize>,
}

#[derive(Clone, Copy)]
struct Cursor {
    row: usize,
    next: usize,
    end: usize,
    word: u64,
}

impl Graph {
    fn new(pair: &Pair<'_, '_>, radius: f64, _stats: &mut Diagnostics) -> Result<Self> {
        let size = pair.size;
        let words = if size < 128 { size.div_ceil(64) } else { 0 };
        let mut result = Self {
            size,
            words,
            bits: filled(size.checked_mul(words).ok_or_else(size_overflow)?, 0)?,
            offsets: Vec::new(),
            neighbors: Vec::new(),
        };
        if words == 0 {
            reserve(
                &mut result.offsets,
                size.checked_add(1).ok_or_else(size_overflow)?,
            )?;
        }
        let n = pair.first.points.len();
        let m = pair.second.points.len();
        for left in 0..size {
            if words == 0 {
                result.offsets.push(result.neighbors.len());
            }
            if left < n {
                if words == 0 {
                    for position in pair.second.window(pair.first.points[left][0], radius) {
                        let right = pair.second.order[position];
                        record! { _stats.adjacency_checks += 1; }
                        if pair.cross(left, right) <= radius {
                            result.add(left, right)?;
                        }
                    }
                } else {
                    for right in 0..m {
                        record! { _stats.adjacency_checks += 1; }
                        if pair.cross(left, right) <= radius {
                            result.add(left, right)?;
                        }
                    }
                }
                if pair.first.diagonals[left] <= radius {
                    result.add(left, m + left)?;
                }
            } else {
                let point = left - n;
                if pair.second.diagonals[point] <= radius {
                    result.add(left, point)?;
                }
                for right in m..size {
                    result.add(left, right)?;
                }
            }
        }
        if words == 0 {
            result.offsets.push(result.neighbors.len());
        }
        Ok(result)
    }

    fn add(&mut self, left: usize, right: usize) -> Result<()> {
        if self.words == 0 {
            push(&mut self.neighbors, right)
        } else {
            self.bits[left * self.words + right / 64] |= 1_u64 << (right % 64);
            Ok(())
        }
    }

    fn cursor(&self, row: usize) -> Cursor {
        if self.words == 0 {
            Cursor {
                row,
                next: self.offsets[row],
                end: self.offsets[row + 1],
                word: 0,
            }
        } else {
            Cursor {
                row,
                next: 0,
                end: self.words,
                word: self.bits[row * self.words],
            }
        }
    }

    fn next(&self, cursor: &mut Cursor) -> Option<usize> {
        if self.words == 0 {
            if cursor.next == cursor.end {
                return None;
            }
            let result = self.neighbors[cursor.next];
            cursor.next += 1;
            return Some(result);
        }
        while cursor.next < cursor.end {
            if cursor.word != 0 {
                let bit = cursor.word.trailing_zeros() as usize;
                cursor.word &= cursor.word - 1;
                return Some(cursor.next * 64 + bit);
            }
            cursor.next += 1;
            if cursor.next < cursor.end {
                cursor.word = self.bits[cursor.row * self.words + cursor.next];
            }
        }
        None
    }

    fn degree(&self, row: usize) -> usize {
        if self.words == 0 {
            self.offsets[row + 1] - self.offsets[row]
        } else {
            self.bits[row * self.words..(row + 1) * self.words]
                .iter()
                .map(|w| w.count_ones() as usize)
                .sum()
        }
    }

    #[cfg(any(test, cocycle_distance_bench))]
    fn bytes(&self) -> usize {
        bytes(&self.bits)
            .saturating_add(bytes(&self.offsets))
            .saturating_add(bytes(&self.neighbors))
    }
}

struct Frame {
    left: usize,
    cursor: Cursor,
    via: usize,
}

pub(super) struct Workspace {
    left: Vec<usize>,
    right: Vec<usize>,
    order: Vec<usize>,
    degrees: Vec<usize>,
    seen: Vec<usize>,
    stack: Vec<Frame>,
    #[cfg(any(test, cocycle_distance_bench))]
    used: bool,
}

impl Workspace {
    pub(super) fn new(size: usize) -> Result<Self> {
        let mut stack = Vec::new();
        reserve(&mut stack, size)?;
        Ok(Self {
            left: filled(size, NONE)?,
            right: filled(size, NONE)?,
            order: filled(size, 0)?,
            degrees: filled(size, 0)?,
            seen: filled(size, 0)?,
            stack,
            #[cfg(any(test, cocycle_distance_bench))]
            used: false,
        })
    }

    pub(super) fn within(
        &mut self,
        pair: &Pair<'_, '_>,
        radius: f64,
        _stats: &mut Diagnostics,
        _outer_bytes: usize,
    ) -> Result<bool> {
        let graph = Graph::new(pair, radius, _stats)?;
        record! { _stats.workspace(
            _outer_bytes
                .saturating_add(graph.bytes())
                .saturating_add(self.bytes()),
        ); }
        record! {
            if self.used {
                _stats.scratch_reuses += 1;
            }
            self.used = true;
        }
        self.left.fill(NONE);
        self.right.fill(NONE);
        self.seen.fill(0);
        for vertex in 0..graph.size {
            self.order[vertex] = vertex;
            self.degrees[vertex] = graph.degree(vertex);
        }
        self.order
            .sort_unstable_by_key(|&vertex| (self.degrees[vertex], vertex));
        for &left in &self.order {
            let mut cursor = graph.cursor(left);
            while let Some(right) = graph.next(&mut cursor) {
                if self.right[right] == NONE {
                    self.left[left] = right;
                    self.right[right] = left;
                    break;
                }
            }
        }
        for position in 0..graph.size {
            let left = self.order[position];
            if self.left[left] != NONE {
                continue;
            }
            record! { _stats.augment_searches += 1; }
            // position + 1 is bounded by graph.size, whose buffers were allocated.
            let generation = position + 1;
            self.stack.clear();
            self.stack.push(Frame {
                left,
                cursor: graph.cursor(left),
                via: NONE,
            });
            let mut augmented = false;
            while let Some(frame) = self.stack.last_mut() {
                let Some(right) = graph.next(&mut frame.cursor) else {
                    self.stack.pop();
                    continue;
                };
                if self.seen[right] == generation {
                    continue;
                }
                self.seen[right] = generation;
                let previous = self.right[right];
                if previous == NONE {
                    let mut assign = right;
                    for frame in self.stack.iter().rev() {
                        self.left[frame.left] = assign;
                        self.right[assign] = frame.left;
                        assign = frame.via;
                    }
                    augmented = true;
                    break;
                }
                self.stack.push(Frame {
                    left: previous,
                    cursor: graph.cursor(previous),
                    via: right,
                });
            }
            if !augmented {
                return Ok(false);
            }
        }
        Ok(true)
    }

    #[cfg(any(test, cocycle_distance_bench))]
    pub(super) fn bytes(&self) -> usize {
        bytes(&self.left)
            .saturating_add(bytes(&self.right))
            .saturating_add(bytes(&self.order))
            .saturating_add(bytes(&self.degrees))
            .saturating_add(bytes(&self.seen))
            .saturating_add(bytes(&self.stack))
    }
}
