// kernel/pde_ref/serialization.hpp
#ifndef VEILIRIS_SERIALIZATION_HPP
#define VEILIRIS_SERIALIZATION_HPP

#include <vector>
#include <cstdint>
#include <cstring>
#include <stdexcept>

namespace veiliris {
namespace pde {

template<typename T>
class Serializer {
public:
    static std::vector<uint8_t> serialize(const std::vector<std::vector<T>>& field) {
        size_t rows = field.size();
        size_t cols = field.empty() ? 0 : field[0].size();
        size_t float_count = rows * cols;
        
        std::vector<uint8_t> buffer(
            sizeof(uint32_t) * 2 +  // rows, cols
            sizeof(T) * float_count   // data
        );
        
        uint8_t* ptr = buffer.data();
        memcpy(ptr, &rows, sizeof(uint32_t));
        ptr += sizeof(uint32_t);
        memcpy(ptr, &cols, sizeof(uint32_t));
        ptr += sizeof(uint32_t);
        
        for (const auto& row : field) {
            memcpy(ptr, row.data(), row.size() * sizeof(T));
            ptr += row.size() * sizeof(T);
        }
        
        return buffer;
    }
    
    static std::vector<std::vector<T>> deserialize(const std::vector<uint8_t>& buffer) {
        if (buffer.size() < sizeof(uint32_t) * 2) {
            throw std::runtime_error("Buffer too small for deserialization");
        }
        
        const uint8_t* ptr = buffer.data();
        uint32_t rows, cols;
        memcpy(&rows, ptr, sizeof(uint32_t));
        ptr += sizeof(uint32_t);
        memcpy(&cols, ptr, sizeof(uint32_t));
        ptr += sizeof(uint32_t);
        
        std::vector<std::vector<T>> field(rows, std::vector<T>(cols));
        
        for (size_t i = 0; i < rows; ++i) {
            memcpy(field[i].data(), ptr, cols * sizeof(T));
            ptr += cols * sizeof(T);
        }
        
        return field;
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_SERIALIZATION_HPP