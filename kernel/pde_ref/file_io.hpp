// kernel/pde_ref/file_io.hpp
#ifndef VEILIRIS_FILE_IO_HPP
#define VEILIRIS_FILE_IO_HPP

#include <vector>
#include <string>
#include <fstream>
#include <sstream>
#include <iomanip>

namespace veiliris {
namespace io {

template<typename T>
class FieldWriter {
public:
    static bool write_binary(const std::string& filename, 
                             const std::vector<std::vector<T>>& field) {
        std::ofstream file(filename, std::ios::binary);
        if (!file) return false;
        
        size_t rows = field.size();
        size_t cols = field.empty() ? 0 : field[0].size();
        
        file.write(reinterpret_cast<const char*>(&rows), sizeof(size_t));
        file.write(reinterpret_cast<const char*>(&cols), sizeof(size_t));
        
        for (const auto& row : field) {
            file.write(reinterpret_cast<const char*>(row.data()), 
                      row.size() * sizeof(T));
        }
        return true;
    }
    
    static bool read_binary(const std::string& filename,
                           std::vector<std::vector<T>>& field) {
        std::ifstream file(filename, std::ios::binary);
        if (!file) return false;
        
        size_t rows, cols;
        file.read(reinterpret_cast<char*>(&rows), sizeof(size_t));
        file.read(reinterpret_cast<char*>(&cols), sizeof(size_t));
        
        field.resize(rows, std::vector<T>(cols));
        for (auto& row : field) {
            file.read(reinterpret_cast<char*>(row.data()), 
                     row.size() * sizeof(T));
        }
        return true;
    }
    
    static bool write_csv(const std::string& filename,
                         const std::vector<std::vector<T>>& field) {
        std::ofstream file(filename);
        if (!file) return false;
        
        file << std::fixed << std::setprecision(6);
        for (const auto& row : field) {
            for (size_t i = 0; i < row.size(); ++i) {
                if (i > 0) file << ",";
                file << row[i];
            }
            file << "\n";
        }
        return true;
    }
    
    static bool read_csv(const std::string& filename,
                        std::vector<std::vector<T>>& field) {
        std::ifstream file(filename);
        if (!file) return false;
        
        field.clear();
        std::string line;
        while (std::getline(file, line)) {
            std::vector<T> row;
            std::stringstream ss(line);
            std::string cell;
            while (std::getline(ss, cell, ',')) {
                row.push_back(static_cast<T>(std::stod(cell)));
            }
            if (!row.empty()) field.push_back(row);
        }
        return true;
    }
};

} // namespace io
} // namespace veiliris

#endif // VEILIRIS_FILE_IO_HPP