#include <cstdint>
#include <cstdlib>

#ifdef CLIPPER2_AVAILABLE
#include "clipper2/clipper.h"
using namespace Clipper2Lib;
#endif

extern "C" {

// Polygons are passed as a flat array of i64 [x0,y0,x1,y1,...]
// with offsets marking each path start. Offsets length = path_count.
// Returns 0 on success, non-zero on failure.
int clipper2_nfp(
    const int64_t* subject_xy, std::size_t subject_len,
    const int64_t* subject_offsets, std::size_t subject_offset_len,
    const int64_t* clip_xy, std::size_t clip_len,
    const int64_t* clip_offsets, std::size_t clip_offset_len,
    int64_t** out_xy, std::size_t* out_len,
    int64_t** out_offsets, std::size_t* out_offset_len
) {
#ifdef CLIPPER2_AVAILABLE
    // NFP = MinkowskiSum(clip, -subject)
    auto build_paths = [](const int64_t* xy, std::size_t len,
                          const int64_t* offsets, std::size_t off_len) {
        Paths64 paths;
        if (!xy || len < 2 || !offsets || off_len == 0) return paths;
        for (size_t i = 0; i < off_len; ++i) {
            size_t start = static_cast<size_t>(offsets[i]);
            size_t end = (i + 1 < off_len) ? static_cast<size_t>(offsets[i + 1]) : len;
            if (start >= len || end > len || end - start < 4) continue;
            Path64 path;
            for (size_t j = start; j + 1 < end; j += 2) {
                path.emplace_back(Point64(xy[j], xy[j + 1]));
            }
            if (path.size() >= 3 && path.front() == path.back()) {
                path.pop_back();
            }
            paths.emplace_back(std::move(path));
        }
        return paths;
    };

    Paths64 subject_paths = build_paths(subject_xy, subject_len, subject_offsets, subject_offset_len);
    Paths64 clip_paths = build_paths(clip_xy, clip_len, clip_offsets, clip_offset_len);
    Paths64 result;

    for (const auto& subj : subject_paths) {
        Path64 neg_subj;
        neg_subj.reserve(subj.size());
        for (const auto& p : subj) {
            neg_subj.emplace_back(Point64(-p.x, -p.y));
        }
        for (const auto& clip : clip_paths) {
            Paths64 tmp = MinkowskiSum(clip, neg_subj, true);
            result.insert(result.end(), tmp.begin(), tmp.end());
        }
    }

    // Union to clean degeneracies
    Paths64 unified;
    Clipper64 c;
    c.AddSubject(result);
    c.Execute(ClipType::Union, FillRule::NonZero, unified);

    // Flatten output
    std::size_t total = 0;
    for (const auto& path : unified) total += path.size() * 2;
    if (total == 0) {
        *out_xy = nullptr; *out_len = 0;
        *out_offsets = nullptr; *out_offset_len = 0;
        return 0;
    }
    int64_t* flat = static_cast<int64_t*>(std::malloc(sizeof(int64_t) * total));
    int64_t* offs = static_cast<int64_t*>(std::malloc(sizeof(int64_t) * unified.size()));
    if (!flat || !offs) {
        std::free(flat); std::free(offs);
        return 3;
    }
    std::size_t cursor = 0;
    for (size_t i = 0; i < unified.size(); ++i) {
        offs[i] = static_cast<int64_t>(cursor);
        const auto& path = unified[i];
        for (const auto& p : path) {
            flat[cursor++] = p.x;
            flat[cursor++] = p.y;
        }
    }
    *out_xy = flat;
    *out_len = total;
    *out_offsets = offs;
    *out_offset_len = unified.size();
    return 0;
#else
    (void)subject_xy; (void)subject_len; (void)subject_offsets; (void)subject_offset_len;
    (void)clip_xy; (void)clip_len; (void)clip_offsets; (void)clip_offset_len;
    *out_xy = nullptr;
    *out_len = 0;
    *out_offsets = nullptr;
    *out_offset_len = 0;
    return 1;
#endif
}

int clipper2_ifp(
    const int64_t* subject_xy, std::size_t subject_len,
    const int64_t* subject_offsets, std::size_t subject_offset_len,
    const int64_t* container_xy, std::size_t container_len,
    const int64_t* container_offsets, std::size_t container_offset_len,
    int64_t** out_xy, std::size_t* out_len,
    int64_t** out_offsets, std::size_t* out_offset_len
) {
#ifdef CLIPPER2_AVAILABLE
    auto build_paths = [](const int64_t* xy, std::size_t len,
                          const int64_t* offsets, std::size_t off_len) {
        Paths64 paths;
        if (!xy || len < 2 || !offsets || off_len == 0) return paths;
        for (size_t i = 0; i < off_len; ++i) {
            size_t start = static_cast<size_t>(offsets[i]);
            size_t end = (i + 1 < off_len) ? static_cast<size_t>(offsets[i + 1]) : len;
            if (start >= len || end > len || end - start < 4) continue;
            Path64 path;
            for (size_t j = start; j + 1 < end; j += 2) {
                path.emplace_back(Point64(xy[j], xy[j + 1]));
            }
            if (path.size() >= 3 && path.front() == path.back()) {
                path.pop_back();
            }
            paths.emplace_back(std::move(path));
        }
        return paths;
    };

    Paths64 subject_paths = build_paths(subject_xy, subject_len, subject_offsets, subject_offset_len);
    Paths64 container_paths = build_paths(container_xy, container_len, container_offsets, container_offset_len);

    if (subject_paths.empty() || container_paths.empty()) {
        *out_xy = nullptr; *out_len = 0;
        *out_offsets = nullptr; *out_offset_len = 0;
        return 0;
    }

    // IFP = intersection over all subject vertices of (container - vertex)
    // (Exact for convex subject + convex container; good fallback for now.)
    Paths64 current = container_paths;
    const Path64& subj0 = subject_paths.front();
    for (const auto& p : subj0) {
        Paths64 shifted;
        shifted.reserve(container_paths.size());
        for (const auto& cont : container_paths) {
            Path64 moved;
            moved.reserve(cont.size());
            for (const auto& cpt : cont) {
                moved.emplace_back(Point64(cpt.x - p.x, cpt.y - p.y));
            }
            shifted.emplace_back(std::move(moved));
        }
        Paths64 intersected;
        Clipper64 c;
        c.AddSubject(current);
        c.AddClip(shifted);
        c.Execute(ClipType::Intersection, FillRule::NonZero, intersected);
        current.swap(intersected);
        if (current.empty()) break;
    }

    Paths64 unified = current;

    std::size_t total = 0;
    for (const auto& path : unified) total += path.size() * 2;
    if (total == 0) {
        *out_xy = nullptr; *out_len = 0;
        *out_offsets = nullptr; *out_offset_len = 0;
        return 0;
    }
    int64_t* flat = static_cast<int64_t*>(std::malloc(sizeof(int64_t) * total));
    int64_t* offs = static_cast<int64_t*>(std::malloc(sizeof(int64_t) * unified.size()));
    if (!flat || !offs) {
        std::free(flat); std::free(offs);
        return 3;
    }
    std::size_t cursor = 0;
    for (size_t i = 0; i < unified.size(); ++i) {
        offs[i] = static_cast<int64_t>(cursor);
        const auto& path = unified[i];
        for (const auto& p : path) {
            flat[cursor++] = p.x;
            flat[cursor++] = p.y;
        }
    }
    *out_xy = flat;
    *out_len = total;
    *out_offsets = offs;
    *out_offset_len = unified.size();
    return 0;
#else
    (void)subject_xy; (void)subject_len; (void)subject_offsets; (void)subject_offset_len;
    (void)container_xy; (void)container_len; (void)container_offsets; (void)container_offset_len;
    *out_xy = nullptr;
    *out_len = 0;
    *out_offsets = nullptr;
    *out_offset_len = 0;
    return 1;
#endif
}

void clipper2_free(int64_t* ptr) { std::free(ptr); }

}
