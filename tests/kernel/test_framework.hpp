#pragma once

#include <cmath>
#include <functional>
#include <stdexcept>
#include <string>
#include <vector>

struct TestCase {
  std::string name;
  std::function<void()> fn;
};

std::vector<TestCase>& TestRegistry();

struct Registrar {
  Registrar(const char* name, std::function<void()> fn);
};

#define TEST(name)                                    void name();                                        static Registrar reg_##name(#name, name);          void name()

#define ASSERT_TRUE(cond)                                                     do {                                                                          if (!(cond)) throw std::runtime_error(std::string("ASSERT_TRUE failed: ") + #cond);   } while (0)

#define ASSERT_EQ(a, b)                                                       do {                                                                          if (!((a) == (b))) throw std::runtime_error("ASSERT_EQ failed");         } while (0)

#define ASSERT_NEAR(a, b, eps)                                                do {                                                                          const auto da = (a);                                                        const auto db = (b);                                                        if (std::abs(da - db) > (eps)) throw std::runtime_error("ASSERT_NEAR failed");   } while (0)
