// kernel/pde_ref/file_io.cpp
// File I/O implementation

#include "file_io.hpp"
#include <fstream>
#include <sstream>
#include <stdexcept>

namespace veiliris {
namespace io {

template<typename T>
void FieldWriter<T>::write_csv(const std::string& filename, 
                                const std::vector<std::vector<T>>& field) {
    std::ofstream file(filename);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open file for writing: " + filename);
    }
    
    for (const auto& row : field) {
        for (size_t i = 0; i < row.size(); ++i) {
            file << row[i];
            if (i < row.size() - 1) file << ",";
        }
        file << "\n";
    }
}

template<typename T>
void FieldWriter<T>::write_binary(const std::string& filename,
                                   const std::vector<std::vector<T>>& field) {
    std::ofstream file(filename, std::ios::binary);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open file for binary writing: " + filename);
    }
    
    size_t rows = field.size();
    size_t cols = field[0].size();
    file.write(reinterpret_cast<const char*>(&rows), sizeof(size_t));
    file.write(reinterpret_cast<const char*>(&cols), sizeof(size_t));
    
    for (const auto& row : field) {
        file.write(reinterpret_cast<const char*>(row.data()), row.size() * sizeof(T));
    }
}

// Explicit template instantiations
template class FieldWriter<float>;
template class FieldWriter<double>;

template<typename T>
std::vector<std::vector<T>> FieldReader<T>::read_csv(const std::string& filename) {
    std::ifstream file(filename);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open file for reading: " + filename);
    }
    
    std::vector<std::vector<T>> field;
    std::string line;
    while (std::getline(file, line)) {
        std::vector<T> row;
        std::stringstream ss(line);
        std::string cell;
        while (std::getline(ss, cell, ',')) {
            row.push_back(static_cast<T>(std::stod(cell)));
        }
        field.push_back(row);
    }
    return field;
}

template class FieldReader<float>;
template class FieldReader<double>;

} // namespace io
} // namespace veiliris