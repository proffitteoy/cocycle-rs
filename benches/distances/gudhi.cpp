// Isolated correctness oracle, pinned to the merged GUDHI matching-shortcut fix.
#include <gudhi/Bottleneck.h>

#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace {
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
}  // namespace

int main(int argc, char** argv) {
  if (argc != 4 || std::string(argv[2]) != "bottleneck" || std::string(argv[3]) != "baseline")
    return 2;
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
    std::vector<std::pair<double, double>> first, second;
    for (auto* diagram : {&first, &second}) {
      const auto count = diagram == &first ? n : m;
      diagram->reserve(count);
      for (std::uint64_t i = 0; i < count; ++i) {
        const auto birth = real(stream), death = real(stream);
        if (!std::isfinite(birth) || std::isnan(death) || death < birth)
          throw std::runtime_error("invalid endpoint");
        diagram->emplace_back(birth, death);
      }
    }
    const auto start = std::chrono::steady_clock::now();
    const double value = Gudhi::persistence_diagram::bottleneck_distance(first, second, 0.0);
    const auto elapsed = std::chrono::duration<double, std::milli>(
        std::chrono::steady_clock::now() - start).count();
    if (std::isnan(value) || value < 0) throw std::runtime_error("invalid oracle value");
    std::cout << std::setprecision(17)
              << "{\"protocol_id\":\"cocycle-distance-v1\",\"status\":\"completed\","
              << "\"backend\":\"gudhi_cpp\",\"metric\":\"bottleneck\",\"variant\":\"baseline\","
              << "\"correctness_reference_only\":true,\"settings\":{\"e\":0},"
              << "\"value_kind\":\"" << (std::isinf(value) ? "infinite" : "finite") << "\",\"value\":";
    if (std::isinf(value)) std::cout << "null";
    else std::cout << value;
    std::cout << ",\"elapsed_ms\":" << elapsed << "}\n";
    return 0;
  } catch (const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
