// kernel/pde_ref/solver_registry.hpp
#ifndef VEILIRIS_SOLVER_REGISTRY_HPP
#define VEILIRIS_SOLVER_REGISTRY_HPP

#include <functional>
#include <unordered_map>
#include <memory>
#include <string>

namespace veiliris {
namespace pde {

template<typename Solver>
class SolverRegistry {
public:
    using SolverFactory = std::function<std::unique_ptr<Solver>(const std::unordered_map<std::string, std::string>&)>;
    
    static SolverRegistry& instance() {
        static SolverRegistry registry;
        return registry;
    }
    
    void register_solver(const std::string& name, SolverFactory factory) {
        factories_[name] = factory;
    }
    
    std::unique_ptr<Solver> create(const std::string& name, 
                                   const std::unordered_map<std::string, std::string>& params) {
        auto it = factories_.find(name);
        if (it == factories_.end()) {
            throw std::runtime_error("Unknown solver: " + name);
        }
        return it->second(params);
    }
    
    std::vector<std::string> list_solvers() const {
        std::vector<std::string> names;
        for (const auto& pair : factories_) {
            names.push_back(pair.first);
        }
        return names;
    }
    
private:
    SolverRegistry() = default;
    std::unordered_map<std::string, SolverFactory> factories_;
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_SOLVER_REGISTRY_HPP