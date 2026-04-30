#include <cstdlib>
#include <exception>
#include <iostream>

#include "test_framework.hpp"

std::vector<TestCase>& TestRegistry() {
  static std::vector<TestCase> tests;
  return tests;
}

Registrar::Registrar(const char* name, std::function<void()> fn) {
  TestRegistry().push_back({name, std::move(fn)});
}

int main() {
  std::size_t failed = 0;
  for (const auto& t : TestRegistry()) {
    try {
      t.fn();
      std::cout << "[PASS] " << t.name << "\n";
    } catch (const std::exception& e) {
      std::cerr << "[FAIL] " << t.name << ": " << e.what() << "\n";
      ++failed;
    }
  }

  return failed == 0 ? EXIT_SUCCESS : EXIT_FAILURE;
}
