// kernel/pde_ref/parallel_executor.hpp
#ifndef VEILIRIS_PARALLEL_EXECUTOR_HPP
#define VEILIRIS_PARALLEL_EXECUTOR_HPP

#include <vector>
#include <thread>
#include <future>
#include <algorithm>
#include <execution>

namespace veiliris {
namespace pde {

template<typename Solver>
class ParallelExecutor {
public:
    ParallelExecutor(size_t num_threads = std::thread::hardware_concurrency())
        : num_threads_(num_threads) {}
    
    template<typename Func>
    void for_each_parallel(size_t start, size_t end, Func func) {
        if (end - start < 1000 || num_threads_ == 1) {
            for (size_t i = start; i < end; ++i) {
                func(i);
            }
            return;
        }
        
        size_t chunk_size = (end - start) / num_threads_;
        std::vector<std::future<void>> futures;
        
        for (size_t t = 0; t < num_threads_; ++t) {
            size_t chunk_start = start + t * chunk_size;
            size_t chunk_end = (t == num_threads_ - 1) ? end : chunk_start + chunk_size;
            
            futures.push_back(std::async(std::launch::async, [=]() {
                for (size_t i = chunk_start; i < chunk_end; ++i) {
                    func(i);
                }
            }));
        }
        
        for (auto& f : futures) {
            f.wait();
        }
    }
    
    template<typename Iterator, typename Func>
    void transform_parallel(Iterator first, Iterator last, Func func) {
        std::for_each(std::execution::par_unseq, first, last, func);
    }
    
private:
    size_t num_threads_;
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_PARALLEL_EXECUTOR_HPP