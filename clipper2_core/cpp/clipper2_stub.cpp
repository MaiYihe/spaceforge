#include <cstdint>
#include <cstdlib>

extern "C" {
int clipper2_nfp(
    const int64_t* subject_xy, std::size_t subject_len,
    const int64_t* subject_offsets, std::size_t subject_offset_len,
    const int64_t* clip_xy, std::size_t clip_len,
    const int64_t* clip_offsets, std::size_t clip_offset_len,
    int64_t** out_xy, std::size_t* out_len,
    int64_t** out_offsets, std::size_t* out_offset_len
) {
    (void)subject_xy; (void)subject_len; (void)subject_offsets; (void)subject_offset_len;
    (void)clip_xy; (void)clip_len; (void)clip_offsets; (void)clip_offset_len;
    *out_xy = nullptr;
    *out_len = 0;
    *out_offsets = nullptr;
    *out_offset_len = 0;
    return 1;
}

int clipper2_ifp(
    const int64_t* subject_xy, std::size_t subject_len,
    const int64_t* subject_offsets, std::size_t subject_offset_len,
    const int64_t* container_xy, std::size_t container_len,
    const int64_t* container_offsets, std::size_t container_offset_len,
    int64_t** out_xy, std::size_t* out_len,
    int64_t** out_offsets, std::size_t* out_offset_len
) {
    (void)subject_xy; (void)subject_len; (void)subject_offsets; (void)subject_offset_len;
    (void)container_xy; (void)container_len; (void)container_offsets; (void)container_offset_len;
    *out_xy = nullptr;
    *out_len = 0;
    *out_offsets = nullptr;
    *out_offset_len = 0;
    return 1;
}

void clipper2_free(int64_t* ptr) { std::free(ptr); }
}
