// Isolated Topp C++20 raw-diagram worker. The pinned source is never modified.
#include <bottleneck/core.hpp>
#include <bottleneck/wasserstein.hpp>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>

namespace {
// The four-row adaptive-density calculation below is adapted from pinned Topp
// src/wasserstein.cpp. Only its parallel candidate path is replaced with the
// existing serial blocked path; other adaptive decisions remain unchanged.
//
// Copyright (c) 2026 Topp contributors
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
bool configure_serial(const bottleneck::PreparedDiagram& first,
                      const bottleneck::PreparedDiagram& second,
                      bottleneck::WassersteinConfig& options) {
  const auto rows = first.finite_points().size(), columns = second.finite_points().size();
  if (std::uint64_t(rows) * columns < 262144) return false;
  const auto sample_rows = std::min(std::size_t{4}, rows);
  if (sample_rows == 0 || columns == 0) return false;
  const auto& midpoints = first.finite_midpoints();
  const auto& halves = first.finite_half_persistences();
  const auto& sorted = second.sorted_finite_midpoints();
  std::size_t candidates = 0;
  for (std::size_t sample = 0; sample < sample_rows; ++sample) {
    const auto row = sample * rows / sample_rows;
    const double radius = options.metric == bottleneck::WassersteinMetric::w1_linf
        ? 2.0 * halves[row]
        : std::sqrt(2.0 * halves[row] * second.max_finite_half_persistence());
    const auto begin = std::lower_bound(sorted.begin(), sorted.end(), midpoints[row] - radius);
    const auto end = std::upper_bound(begin, sorted.end(), midpoints[row] + radius);
    candidates += static_cast<std::size_t>(end - begin);
  }
  if (static_cast<double>(candidates) / static_cast<double>(sample_rows * columns) < 0.75)
    return false;
  options.candidates = bottleneck::WassersteinCandidateStrategy::dense_blocked;
  return true;
}

std::uint64_t integer(std::istream& stream) {
  unsigned char bytes[8];
  if (!stream.read(reinterpret_cast<char*>(bytes), 8))
    throw std::runtime_error("truncated fixture");
  std::uint64_t value = 0;
  for (int i = 0; i < 8; ++i) value |= std::uint64_t(bytes[i]) << (8 * i);
  return value;
}

double real(std::istream& stream) {
  const auto bits = integer(stream);
  double value;
  static_assert(sizeof(value) == sizeof(bits));
  std::memcpy(&value, &bits, sizeof(value));
  return value;
}

long memory(const std::string& field) {
  std::ifstream stream("/proc/self/status");
  std::string line;
  while (std::getline(stream, line))
    if (line.compare(0, field.size(), field) == 0) return std::stol(line.substr(field.size()));
  return -1;
}

void nullable(long value) {
  if (value < 0) std::cout << "null";
  else std::cout << value;
}
}  // namespace

int main(int argc, char** argv) {
  if (argc != 4) return 2;
  const std::string metric = argv[2], variant = argv[3];
  try {
    std::ifstream stream(argv[1], std::ios::binary | std::ios::ate);
    const auto length = stream.tellg();
    stream.seekg(0);
    char magic[8];
    if (!stream.read(magic, 8) || std::memcmp(magic, "COCDST1\0", 8))
      throw std::runtime_error("expected COCDST1 distance fixture");
    const auto n = integer(stream), m = integer(stream);
    if (length < 24 || n > (std::uint64_t(length) - 24) / 16 ||
        m > (std::uint64_t(length) - 24) / 16 ||
        n + m != (std::uint64_t(length) - 24) / 16 || (std::uint64_t(length) - 24) % 16)
      throw std::runtime_error("invalid fixture length");
    bottleneck::Diagram first, second;
    for (auto* diagram : {&first, &second}) {
      const auto count = diagram == &first ? n : m;
      diagram->reserve(count);
      for (std::uint64_t i = 0; i < count; ++i) {
        const auto birth = real(stream), death = real(stream);
        diagram->push_back({birth, death});
      }
    }
    const long rss = memory("VmRSS:"), hwm = memory("VmHWM:");
    bottleneck::SolverStats bs;
    bottleneck::WassersteinStats ws;
    bool serial_candidate_override = false;
    const auto start = std::chrono::steady_clock::now();
    double value;
    if (metric == "bottleneck") {
      bottleneck::SolverConfig options;
      if (variant == "binary") options.threshold = bottleneck::ThresholdStrategy::binary;
      else if (variant == "refinement") options.threshold = bottleneck::ThresholdStrategy::geometric_refinement;
      else if (variant == "quickselect") options.threshold = bottleneck::ThresholdStrategy::quickselect;
      else if (variant == "quickselect_no_clip") {
        options.threshold = bottleneck::ThresholdStrategy::quickselect;
        options.candidates = bottleneck::CandidateStrategy::sort_unique;
      }
      else if (variant != "baseline" && variant != "default") throw std::invalid_argument("unsupported Topp bottleneck variant");
      value = bottleneck::bottleneck_distance(first, second, options, &bs);
    } else if (metric == "w1" || metric == "w2") {
      bottleneck::WassersteinConfig options;
      options.metric = metric == "w1" ? bottleneck::WassersteinMetric::w1_linf
                                      : bottleneck::WassersteinMetric::w2_l2;
      if (variant == "vectors" || variant == "arena") {
        options.matcher = variant == "arena" ? bottleneck::WassersteinMatcherStrategy::sparse_sap_arena
                                             : bottleneck::WassersteinMatcherStrategy::sparse_sap;
        options.components = bottleneck::WassersteinComponentStrategy::none;
        options.warm_start = bottleneck::WassersteinWarmStart::none;
        options.duplicates = bottleneck::WassersteinDuplicateStrategy::none;
      } else if (variant != "baseline" && variant != "default") throw std::invalid_argument("unsupported Topp Wasserstein variant");
      if (variant == "default") {
        value = bottleneck::wasserstein_distance(first, second, options, &ws);
      } else {
        const bottleneck::PreparedDiagram prepared_first(first), prepared_second(second);
        serial_candidate_override = configure_serial(prepared_first, prepared_second, options);
        value = bottleneck::wasserstein_distance(prepared_first, prepared_second, options, &ws);
      }
    } else throw std::invalid_argument("unknown metric");
    const auto elapsed = std::chrono::duration<double, std::milli>(
        std::chrono::steady_clock::now() - start).count();
    const long peak = memory("VmHWM:");
    if (std::isnan(value) || value < 0) throw std::runtime_error("invalid solver result");
    std::cout << std::setprecision(17)
              << "{\"protocol_id\":\"cocycle-distance-v1\",\"status\":\"completed\","
              << "\"backend\":\"topp\",\"metric\":\"" << metric << "\",\"variant\":\"" << variant
              << "\",\"value_kind\":\"" << (std::isinf(value) ? "infinite" : "finite") << "\",\"value\":";
    if (std::isinf(value)) std::cout << "null";
    else std::cout << value;
    std::cout << ",\"elapsed_ms\":" << elapsed << ",\"rss_before_kib\":";
    nullable(rss);
    std::cout << ",\"hwm_before_kib\":"; nullable(hwm);
    std::cout << ",\"peak_rss_kib\":"; nullable(peak);
    std::cout << ",\"cpp_threads\":" << (variant == "default" ? "null" : "1")
              << ",\"serial_candidate_override\":" << (serial_candidate_override ? "true" : "false")
              << ",\"observed_threads_after\":";
    nullable(memory("Threads:"));
    std::cout << ",\"stats\":{\"threshold_decisions\":" << bs.threshold_decisions
              << ",\"kd_nodes_visited\":" << bs.kd_nodes_visited
              << ",\"candidate_pairs\":" << ws.candidate_pairs
              << ",\"positive_edges\":" << ws.positive_edges
              << ",\"augmentations\":" << ws.augmentations
              << ",\"retained_candidates\":" << bs.retained_candidates
              << ",\"clipped_candidates\":" << bs.clipped_candidates
              << ",\"sparse_arena_builds\":" << ws.sparse_arena_builds
              << ",\"sparse_scratch_reuses\":" << ws.sparse_scratch_reuses
              << ",\"peak_sparse_arena_bytes\":" << ws.peak_sparse_arena_bytes
              << ",\"peak_graph_bytes\":" << ws.peak_graph_bytes
              << ",\"router_multiplicity_routes\":" << bs.router_multiplicity_routes
              << ",\"router_mandatory_routes\":" << bs.router_mandatory_routes
              << ",\"router_refinement_routes\":" << bs.router_refinement_routes
              << ",\"router_quickselect_routes\":" << bs.router_quickselect_routes << "}}\n";
    return 0;
  } catch (const std::exception& error) {
    // A nonzero status retains diagnostics without attempting to escape arbitrary
    // upstream exception strings into JSON. The controller labels process_error.
    std::cerr << error.what() << '\n';
    return 1;
  }
}
