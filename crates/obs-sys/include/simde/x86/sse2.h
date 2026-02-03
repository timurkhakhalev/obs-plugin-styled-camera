#pragma once

// OBS headers reference SIMDe for SSE intrinsics on non-x86 targets (e.g. Apple Silicon).
// The full SIMDe library is not vendored in obs-studio sources, so bindgen would fail
// without an external installation.
//
// This header is a best-effort compatibility shim:
// - If a real SIMDe is available later in the include search path, it is used via
//   `#include_next`.
// - Otherwise, it provides minimal types and function declarations so clang can parse
//   the OBS public headers during bindgen.

#if defined(__has_include_next)
#if __has_include_next(<simde/x86/sse2.h>)
#include_next <simde/x86/sse2.h>
#else

#ifdef __cplusplus
extern "C" {
#endif

typedef struct __obs_sys_m128 {
  float f32[4];
} __m128;

typedef struct __obs_sys_m128d {
  double f64[2];
} __m128d;

typedef struct __obs_sys_m128i {
  unsigned long long u64[2];
} __m128i;

#ifndef _MM_SHUFFLE
#define _MM_SHUFFLE(z, y, x, w) (((z) << 6) | ((y) << 4) | ((x) << 2) | (w))
#endif

__m128 _mm_setzero_ps(void);
__m128 _mm_set_ps(float w, float z, float y, float x);
__m128 _mm_set1_ps(float a);

__m128 _mm_add_ps(__m128 a, __m128 b);
__m128 _mm_sub_ps(__m128 a, __m128 b);
__m128 _mm_mul_ps(__m128 a, __m128 b);
__m128 _mm_div_ps(__m128 a, __m128 b);

__m128 _mm_min_ps(__m128 a, __m128 b);
__m128 _mm_max_ps(__m128 a, __m128 b);

__m128 _mm_movehl_ps(__m128 a, __m128 b);
__m128 _mm_movelh_ps(__m128 a, __m128 b);
__m128 _mm_shuffle_ps(__m128 a, __m128 b, const int imm8);

__m128 _mm_unpacklo_ps(__m128 a, __m128 b);
__m128 _mm_unpackhi_ps(__m128 a, __m128 b);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // __has_include_next(<simde/x86/sse2.h>)
#else
#error "obs-sys: compiler missing __has_include_next; install SIMDe or adjust include paths"
#endif

