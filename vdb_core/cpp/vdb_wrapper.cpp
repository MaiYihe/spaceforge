#include <openvdb/openvdb.h>
#include <openvdb/tools/LevelSetSphere.h>
#include <openvdb/tools/MeshToVolume.h>
#include <openvdb/tools/VolumeToMesh.h>

#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <cstdlib>
#include <limits>

extern "C" {

void vdb_init() {
    openvdb::initialize();
}

static bool parse_obj(
    const char* path,
    std::vector<openvdb::Vec3s>& points,
    std::vector<openvdb::Vec3I>& triangles,
    float scale)
{
    std::ifstream in(path);
    if (!in.is_open()) return false;

    std::string line;
    while (std::getline(in, line)) {
        if (line.size() < 2) continue;

        if (line.rfind("v ", 0) == 0) {
            std::istringstream ss(line.substr(2));
            float x = 0.0f, y = 0.0f, z = 0.0f;
            if (ss >> x >> y >> z) {
                points.emplace_back(x * scale, y * scale, z * scale);
            }
        } else if (line.rfind("f ", 0) == 0) {
            std::istringstream ss(line.substr(2));
            std::string token;
            std::vector<int> face;
            while (ss >> token) {
                std::istringstream ts(token);
                std::string idx_str;
                if (!std::getline(ts, idx_str, '/')) continue;
                if (idx_str.empty()) continue;

                int idx = std::stoi(idx_str);
                int v = 0;
                if (idx > 0) {
                    v = idx - 1;
                } else if (idx < 0) {
                    v = static_cast<int>(points.size()) + idx;
                } else {
                    continue;
                }
                face.push_back(v);
            }

            if (face.size() >= 3) {
                for (size_t i = 1; i + 1 < face.size(); ++i) {
                    triangles.emplace_back(face[0], face[i], face[i + 1]);
                }
            }
        }
    }

    return !points.empty() && !triangles.empty();
}

// 从 OBJ 体素化为 SDF
openvdb::FloatGrid* create_from_obj(
    const char* path, float voxel_size, float scale)
{
    std::vector<openvdb::Vec3s> points;
    std::vector<openvdb::Vec3I> triangles;
    if (!parse_obj(path, points, triangles, scale)) {
        return nullptr;
    }

    auto transform = openvdb::math::Transform::createLinearTransform(voxel_size);
    const float exterior_band = 3.0f;

    auto grid_ptr = openvdb::tools::meshToLevelSet<openvdb::FloatGrid>(
        *transform, points, triangles, exterior_band);

    return new openvdb::FloatGrid(*grid_ptr);
}

int vdb_mesh_from_grid(openvdb::FloatGrid* grid,
                       float isovalue,
                       float adaptivity,
                       float** out_vertices,
                       int* out_vertex_count,
                       int** out_indices,
                       int* out_index_count)
{
    if (!grid || !out_vertices || !out_vertex_count || !out_indices || !out_index_count) {
        return 0;
    }

    std::vector<openvdb::Vec3s> points;
    std::vector<openvdb::Vec3I> triangles;
    std::vector<openvdb::Vec4I> quads;

    openvdb::tools::volumeToMesh(*grid, points, triangles, quads, isovalue, adaptivity);

    const size_t vertex_count = points.size();
    const size_t tri_count = triangles.size();
    const size_t quad_count = quads.size();

    const size_t index_count = (tri_count + quad_count * 2) * 3;
    if (vertex_count == 0 || index_count == 0) {
        return 0;
    }

    float* vertices = static_cast<float*>(std::malloc(vertex_count * 3 * sizeof(float)));
    int* indices = static_cast<int*>(std::malloc(index_count * sizeof(int)));
    if (!vertices || !indices) {
        std::free(vertices);
        std::free(indices);
        return 0;
    }

    for (size_t i = 0; i < vertex_count; ++i) {
        const auto& p = points[i];
        vertices[i * 3 + 0] = p.x();
        vertices[i * 3 + 1] = p.y();
        vertices[i * 3 + 2] = p.z();
    }

    size_t idx = 0;
    for (const auto& t : triangles) {
        indices[idx++] = t.x();
        indices[idx++] = t.y();
        indices[idx++] = t.z();
    }
    for (const auto& q : quads) {
        indices[idx++] = q.x();
        indices[idx++] = q.y();
        indices[idx++] = q.z();
        indices[idx++] = q.x();
        indices[idx++] = q.z();
        indices[idx++] = q.w();
    }

    *out_vertices = vertices;
    *out_vertex_count = static_cast<int>(vertex_count);
    *out_indices = indices;
    *out_index_count = static_cast<int>(index_count);
    return 1;
}

void vdb_mesh_free(float* vertices, int* indices)
{
    std::free(vertices);
    std::free(indices);
}

float vdb_voxel_size(openvdb::FloatGrid* grid)
{
    if (!grid) return 0.0f;
    const openvdb::Vec3d vs = grid->voxelSize();
    return static_cast<float>(vs.x());
}

int vdb_active_voxel_centers(openvdb::FloatGrid* grid,
                             float** out_positions,
                             int* out_count)
{
    if (!grid || !out_positions || !out_count) {
        return 0;
    }

    const size_t count = grid->activeVoxelCount();
    if (count == 0 || count > static_cast<size_t>(std::numeric_limits<int>::max())) {
        return 0;
    }

    float* positions = static_cast<float*>(std::malloc(count * 3 * sizeof(float)));
    if (!positions) {
        return 0;
    }

    size_t i = 0;
    for (auto iter = grid->cbeginValueOn(); iter; ++iter) {
        const openvdb::Coord ijk = iter.getCoord();
        const openvdb::Vec3d world = grid->indexToWorld(ijk);
        positions[i * 3 + 0] = static_cast<float>(world.x());
        positions[i * 3 + 1] = static_cast<float>(world.y());
        positions[i * 3 + 2] = static_cast<float>(world.z());
        i++;
    }

    *out_positions = positions;
    *out_count = static_cast<int>(count);
    return 1;
}

void vdb_active_voxel_centers_free(float* positions)
{
    std::free(positions);
}

int vdb_active_voxel_coords(openvdb::FloatGrid* grid,
                            int** out_coords,
                            int* out_count)
{
    if (!grid || !out_coords || !out_count) {
        return 0;
    }

    const size_t count = grid->activeVoxelCount();
    if (count == 0 || count > static_cast<size_t>(std::numeric_limits<int>::max())) {
        return 0;
    }

    int* coords = static_cast<int*>(std::malloc(count * 3 * sizeof(int)));
    if (!coords) {
        return 0;
    }

    size_t i = 0;
    for (auto iter = grid->cbeginValueOn(); iter; ++iter) {
        const openvdb::Coord ijk = iter.getCoord();
        coords[i * 3 + 0] = ijk.x();
        coords[i * 3 + 1] = ijk.y();
        coords[i * 3 + 2] = ijk.z();
        i++;
    }

    *out_coords = coords;
    *out_count = static_cast<int>(count);
    return 1;
}

void vdb_active_voxel_coords_free(int* coords)
{
    std::free(coords);
}

}
